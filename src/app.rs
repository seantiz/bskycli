use std::collections::HashMap;
use std::time::Duration;
use std::sync::Arc;

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::prelude::*;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::error;

use crate::action::Action;
use crate::api::login_form;
use crate::api::wrapper::{AgentWrapper, ReplyRef};
use crate::event::{self, EventHandler, DoubleTap};
use crate::models::feed::FeedState;
use crate::models::post::PostViewModel;
use crate::models::preferences::PreferencesViewModel;
use crate::models::profile::ProfileViewModel;
use crate::models::thread::ThreadViewModel;
use crate::tui::Tui;
use crate::ui::composer::Composer;
use crate::ui::login::LoginForm;
use crate::ui::{Component, Dialog};
use crate::utils::meta::ImageLibrary;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;

pub struct ImageState {
    pub protocol: StatefulProtocol,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Login,
    Timeline,
    Thread,
    Profile,
    Preferences,
    Search,
    Notifications,
}

pub struct App {
    should_quit: bool,
    screen: Screen,
    screen_stack: Vec<Screen>,
    active_tab: usize,
    client: Arc<AgentWrapper>,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    handle: Option<String>,

    // State
    timeline: FeedState,
    thread: Option<ThreadViewModel>,
    thread_cursor: usize,
    thread_scroll_offset: usize,
    profile: Option<ProfileViewModel>,
    profile_feed: FeedState,
    preferences: PreferencesViewModel,
    preferences_selected_index: usize,
    search_feed: FeedState,
    search_query: String,
    search_focused: bool,
    notifications: Vec<crate::models::notifications::NotificationViewModel>,
    notifications_cursor: Option<String>,
    notifications_loading: bool,
    current_notification: usize,
    error_message: Option<String>,

    // Active data-loading task (aborted when a new load starts or on navigation)
    active_load: Option<JoinHandle<()>>,

    // Better keymaps
    key_tracker: DoubleTap,
    pending_action: Option<Action>,

    // Modals
    login_form: LoginForm,
    composer: Composer,
    show_composer: bool,
    logout_confirmation: Option<Dialog>,
    show_help: bool,

    image_library: ImageLibrary,
    image_protocols: HashMap<String, ImageState>,
    picker: Option<Picker>,
}

impl App {
    pub fn new(client: Arc<AgentWrapper>) -> Self {
        let (action_tx, action_rx) = mpsc::unbounded_channel();

        App {
            should_quit: false,
            screen: Screen::Login,
            screen_stack: Vec::new(),
            active_tab: 0,
            client,
            action_tx,
            action_rx,
            handle: None,
            timeline: FeedState::new(),
            thread: None,
            thread_cursor: 0,
            thread_scroll_offset: 0,
            profile: None,
            profile_feed: FeedState::new(),
            preferences: PreferencesViewModel::load(),
            preferences_selected_index: 1,
            search_feed: FeedState::new(),
            search_query: String::new(),
            search_focused: false,
            notifications: Vec::new(),
            notifications_cursor: None,
            notifications_loading: false,
            current_notification: 0,
            error_message: None,
            active_load: None,
            key_tracker: DoubleTap::new(),
            pending_action: None,
            login_form: LoginForm::new(None),
            composer: Composer::new(),
            show_composer: false,
            logout_confirmation: None,
            show_help: false,
            image_library: ImageLibrary::new(),
            image_protocols: std::collections::HashMap::new(),
            picker: None,
        }
    }

    pub async fn run(&mut self, terminal: &mut Tui) -> Result<()> {
        if let Some(did) = self.client.agent.did().await
            && self
                .client
                .agent
                .api
                .com
                .atproto
                .server
                .refresh_session()
                .await
                .is_ok()
        {
            self.handle = Some(did.to_string());
            self.screen = Screen::Timeline;
            self.dispatch(Action::RefreshTimeline);
        }

        if let Ok(mut picker) = Picker::from_query_stdio() {
            picker.set_protocol_type(ratatui_image::picker::ProtocolType::Kitty);
            self.picker = Some(picker);
        }

        let mut events = EventHandler::new();

        loop {
            terminal.draw(|frame| self.draw(frame))?;

            tokio::select! {
                Some(event) = events.next() => {
                    self.handle_event(event);
                }
                Some(action) = self.action_rx.recv() => {
                    self.update(action).await;
                }
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => {
                // Let modals handle keys first
                if self.show_composer {
                    if let Some(action) = self.composer.handle_key_event(key) {
                        self.dispatch(action);
                    }
                    return;
                }

                if self.screen == Screen::Login {
                    if let Some(action) = self.login_form.handle_key_event(key) {
                        self.dispatch(action);
                    }
                    return;
                }

                if let Some(ref mut dialog) = self.logout_confirmation {
                    if let Some(action) = dialog.handle_key_event(key) {
                        self.dispatch(action);
                    }
                    return;
                }

                if self.show_help {
                    self.show_help = false;
                    return;
                }

                if self.screen == Screen::Preferences {
                    match (key.modifiers, key.code) {
                        (KeyModifiers::NONE, KeyCode::Char('q'))
                        | (KeyModifiers::NONE, KeyCode::Esc) => {
                            self.dispatch(Action::SwitchTab(0));
                            return;
                        }
                        _ => {}
                    }
                }

                if self.screen == Screen::Search {
                    if self.search_focused {
                        match key.code {
                            KeyCode::Char(pressed) if key.modifiers == KeyModifiers::NONE => {
                                self.search_query.push(pressed);
                                return;
                            }
                            KeyCode::Backspace => {
                                self.search_query.pop();
                                return;
                            }
                            KeyCode::Enter => {
                                let query = self.search_query.clone();
                                if !query.is_empty() {
                                    self.search_focused = false;
                                    self.dispatch(Action::Search(query));
                                }
                                return;
                            }
                            KeyCode::Esc => {
                                self.search_focused = false;
                                return;
                            }
                            _ => {}
                        }
                    } else if matches!(key.code, KeyCode::Char('/'))
                        && key.modifiers == KeyModifiers::NONE
                    {
                        self.search_focused = true;
                        return;
                    }
                }

                if matches!(key.code, KeyCode::Char('r')) && key.modifiers == KeyModifiers::NONE {
                    if self.key_tracker.press('r') {
                        self.pending_action = None;
                        self.dispatch(Action::ToggleRepost);
                    } else {
                        let reply_action = self.make_reply_action().unwrap_or(Action::OpenComposer {
                            reply_to: None,
                            reply_to_author: None,
                        });
                        self.pending_action = Some(reply_action);
                        let tx = self.action_tx.clone();
                        tokio::spawn(async move {
                            tokio::time::sleep(Duration::from_millis(600)).await;
                            let _ = tx.send(Action::ReplyTimeout);
                        });
                    }
                    return;
                }

                // Flush pending action
                if let Some(action) = self.pending_action.take() {
                    self.dispatch(action);
                }

                // Global key handling
                if let Some(action) =
                    event::key_to_action(key, self.show_composer, self.screen == Screen::Login)
                {
                    self.dispatch(action);
                }
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
    }

    // NOTE: Moving around
    // 0:   parent[0] is eldest
    // 1:   parent[1]
    // ...
    // N-1: parent[N-1]   is the immediate parent
    // N:   focal         is our view entry point
    // N+1: reply[0]
    // N+2: reply[1]
    fn move_around_thread(&self) -> Option<&PostViewModel> {
        let thread = self.thread.as_ref()?;
        if self.thread_cursor < thread.parents.len() {
            thread.parents.get(self.thread_cursor)
        } else if self.thread_cursor == thread.parents.len() {
            Some(&thread.focal)
        } else {
            thread
                .replies
                .get(self.thread_cursor - thread.parents.len() - 1)
        }
    }

    fn make_reply_action(&self) -> Option<Action> {
        let post = match self.screen {
            Screen::Timeline => self.timeline.selected_post(),
            Screen::Thread => self.move_around_thread(),
            Screen::Profile => self.profile_feed.selected_post(),
            _ => None,
        };

        post.map(|p| Action::OpenComposer {
            reply_to: Some(ReplyRef {
                parent_uri: p.uri.clone(),
                parent_cid: p.cid.clone(),
                root_uri: p.uri.clone(),
                root_cid: p.cid.clone(),
            }),
            reply_to_author: Some(p.display_name.clone()),
        })
    }

    fn dispatch(&self, action: Action) {
        let _ = self.action_tx.send(action);
    }

    fn spawn_load(&mut self, future: impl std::future::Future<Output = ()> + Send + 'static) {
        self.cancel_load();
        self.active_load = Some(tokio::spawn(future));
    }

    fn cancel_load(&mut self) {
        if let Some(handle) = self.active_load.take() {
            handle.abort();
        }
    }

    fn load_selected_post_images(&mut self) {
        let post = match self.screen {
            Screen::Timeline => self.timeline.selected_post(),
            Screen::Profile => self.profile_feed.selected_post(),
            Screen::Search => self.search_feed.selected_post(),
            Screen::Thread => self.move_around_thread(),
            _ => None,
        };

        let post = match post {
            Some(p) => p,
            None => return,
        };

        let embed = match &post.meta {
            Some(e) => match &e.kind {
                crate::models::post::EmbedKind::Images(imgs) => imgs,
                _ => return,
            },
            None => return,
        };

        if embed.is_empty() {
            return;
        }

        if self.image_protocols.contains_key(&post.uri) {
            return;
        }

        let url = embed[0].size.clone();
        let post_uri = post.uri.clone();
        let library = self.image_library.clone();
        let tx = self.action_tx.clone();
        let picker = self.picker.clone();

        self.spawn_load(async move {
            match library.retrieve_or_download(&url).await {
                Ok(path) => {
                    if let Ok(data) = std::fs::read(&path)
                        && let Ok(dyn_img) = image::load_from_memory(&data)
                    {
                        if let Some(picker) = picker {
                            let font_size = picker.font_size();
                            let cols = (dyn_img.width() / font_size.0 as u32).max(1) as u16;
                            let rows = (dyn_img.height() / font_size.1 as u32).max(1) as u16;
                            let protocol = picker.new_resize_protocol(dyn_img);
                            let _ = tx.send(Action::ImageLoaded {
                                post_uri,
                                protocol,
                                cols,
                                rows,
                            });
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to load image: {}", e);
                }
            }
        });
    }

    async fn update(&mut self, action: Action) {
        match action {
            Action::Quit => {
                self.cancel_load();
                self.should_quit = true;
            }

            Action::ShowLogin => self.screen = Screen::Login,

            Action::SubmitLogin { handle, password } => {
                let client = self.client.clone();
                let tx = self.action_tx.clone();
                tokio::spawn(async move {
                    match login_form::login(&client, &handle, &password).await {
                        Ok(h) => {
                            let _ = tx.send(Action::LoginSuccess(h));
                        }
                        Err(e) => {
                            let _ = tx.send(Action::LoginFailed(e.to_string()));
                        }
                    }
                });
            }

            Action::LoginSuccess(handle) => {
                self.handle = Some(handle);
                self.screen = Screen::Timeline;
                self.login_form.clear_error();
                self.dispatch(Action::RefreshTimeline);
            }

            Action::LoginFailed(msg) => {
                self.login_form.set_error(msg);
            }

            Action::Logout => {
                if let Err(e) = login_form::logout(&self.client).await {
                    error!(
                        "Something went wrong and the session may not have been cleared: {}",
                        e
                    )
                }
                self.handle = None;
                self.screen = Screen::Login;
                self.timeline = FeedState::new();
            }

            Action::LogoutConfirm => {
                self.logout_confirmation =
                    Some(Dialog::new("Press Enter to logout or Esc to cancel."));
            }

            Action::DefinitelyLogout => {
                if let Err(e) = login_form::logout(&self.client).await {
                    error!(
                        "Something went wrong and the session may not have been cleared: {}",
                        e
                    )
                }
                self.handle = None;
                self.screen = Screen::Login;
                self.timeline = FeedState::new();
                self.logout_confirmation = None;
            }

            Action::LogoutCancelled => {
                self.logout_confirmation = None;
            }

            Action::ShowHelp => {
                self.show_help = !self.show_help;
            }

            Action::RefreshTimeline => {
                self.timeline.loading = true;
                let client = self.client.clone();
                let tx = self.action_tx.clone();
                let preferences = self.preferences.clone();
                self.spawn_load(async move {
                    match client.get_timeline(None, Some(50u8), preferences).await {
                        Ok((posts, cursor)) => {
                            let _ = tx.send(Action::TimelineLoaded {
                                posts,
                                cursor,
                                append: false,
                            });
                        }
                        Err(e) => {
                            let _ = tx.send(Action::Error(e.to_string()));
                        }
                    }
                });
            }

            Action::LoadMoreTimeline => {
                if self.timeline.loading || self.timeline.cursor.is_none() {
                    return;
                }
                self.timeline.loading = true;
                let client = self.client.clone();
                let cursor = self.timeline.cursor.clone();
                let tx = self.action_tx.clone();
                let preferences = self.preferences.clone();
                self.spawn_load(async move {
                    match client.get_timeline(cursor, Some(50u8), preferences).await {
                        Ok((posts, cursor)) => {
                            let _ = tx.send(Action::TimelineLoaded {
                                posts,
                                cursor,
                                append: true,
                            });
                        }
                        Err(e) => {
                            let _ = tx.send(Action::Error(e.to_string()));
                        }
                    }
                });
            }

            Action::TimelineLoaded {
                posts,
                cursor,
                append,
            } => {
                if append {
                    self.timeline.append_posts(posts, cursor);
                } else {
                    self.timeline.replace_posts(posts, cursor);
                }
            }

            Action::SelectNext => match self.screen {
                Screen::Timeline => {
                    self.timeline.select_next();
                    if self.timeline.near_bottom(20) {
                        self.dispatch(Action::LoadMoreTimeline);
                    }
                }
                Screen::Profile => {
                    self.profile_feed.select_next();
                }
                Screen::Thread => {
                    if let Some(ref thread) = self.thread {
                        let total = thread.parents.len() + 1 + thread.replies.len();
                        if total > 0 && self.thread_cursor + 1 < total {
                            self.thread_cursor += 1;
                            self.load_selected_post_images();
                        }
                    }
                }
                Screen::Search => {
                    self.search_feed.select_next();
                    if self.search_feed.near_bottom(20) {
                        self.dispatch(Action::LoadMoreResults);
                    }
                }
                Screen::Notifications => {
                    if !self.notifications.is_empty()
                        && self.current_notification < self.notifications.len() - 1
                    {
                        self.current_notification += 1;
                    }
                    if self.current_notification + 3 >= self.notifications.len()
                        && self.notifications_cursor.is_some()
                    {
                        self.dispatch(Action::LoadMoreNotifications);
                    }
                }
                Screen::Preferences => {
                    // NOTE: Off by one for ui padding
                    self.preferences_selected_index = (self.preferences_selected_index + 1).min(11);
                }
                _ => {}
            },

            Action::SelectPrev => match self.screen {
                Screen::Timeline => {
                    self.timeline.select_prev();
                }
                Screen::Profile => {
                    self.profile_feed.select_prev();
                }
                Screen::Search => {
                    self.search_feed.select_prev();
                }
                Screen::Notifications => {
                    self.current_notification = self.current_notification.saturating_sub(1);
                }
                Screen::Thread => {
                    if self.thread_cursor > 0 {
                        self.thread_cursor -= 1;
                        self.load_selected_post_images();
                    }
                }
                Screen::Preferences => {
                    // NOTE: Off by one for ui padding
                    self.preferences_selected_index =
                        self.preferences_selected_index.saturating_sub(1).max(1);
                }
                _ => {}
            },

            Action::ScrollToTop => match self.screen {
                Screen::Timeline => self.timeline.select_first(),
                Screen::Profile => self.profile_feed.select_first(),
                Screen::Search => self.search_feed.select_first(),
                Screen::Thread => {
                    self.thread_cursor = 0;
                    self.thread_scroll_offset = 0;
                }
                Screen::Notifications => self.current_notification = 0,
                _ => {}
            },

            Action::ScrollToBottom => match self.screen {
                Screen::Timeline => self.timeline.select_last(),
                Screen::Profile => self.profile_feed.select_last(),
                Screen::Search => self.search_feed.select_last(),
                Screen::Thread => {
                    if let Some(ref thread) = self.thread {
                        let total = thread.parents.len() + 1 + thread.replies.len();
                        if total > 0 {
                            self.thread_cursor = total - 1;
                        }
                    }
                }
                Screen::Notifications => {
                    self.current_notification = self.notifications.len().saturating_sub(1);
                }
                _ => {}
            },

            Action::OpenThread => {
                let uri = match self.screen {
                    Screen::Timeline => self.timeline.selected_post().map(|p| p.uri.clone()),
                    Screen::Profile => self.profile_feed.selected_post().map(|p| p.uri.clone()),
                    Screen::Search => self.search_feed.selected_post().map(|p| p.uri.clone()),
                    Screen::Notifications => self
                        .notifications
                        .get(self.current_notification)
                        .and_then(|n| n.subject.clone()),
                    Screen::Thread => self.move_around_thread().map(|p| p.uri.clone()),
                    _ => None,
                };

                if let Some(uri) = uri {
                    let client = self.client.clone();
                    let tx = self.action_tx.clone();
                    if self.screen != Screen::Thread {
                        self.screen_stack.push(self.screen.clone());
                    }
                    self.screen = Screen::Thread;
                    self.spawn_load(async move {
                        match client.get_thread(&uri).await {
                            Ok(thread) => {
                                let _ = tx.send(Action::ThreadLoaded(Box::new(thread)));
                            }
                            Err(e) => {
                                let _ = tx.send(Action::Error(e.to_string()));
                            }
                        }
                    });
                }
            }

            Action::ThreadLoaded(thread) => {
                self.thread = *thread;
                if let Some(ref t) = self.thread {
                    self.thread_cursor = t.parents.len();
                    self.thread_scroll_offset = 0;
                }
                self.load_selected_post_images();
            }

            Action::GoBack => {
                self.cancel_load();
                if let Some(prev) = self.screen_stack.pop() {
                    self.screen = prev;
                    self.thread = None;
                }
            }

            Action::SwitchTab(idx) => {
                self.active_tab = idx;
                match idx {
                    0 => {
                        self.screen = Screen::Timeline;
                        self.dispatch(Action::RefreshTimeline);
                    }
                    1 => {
                        self.screen = Screen::Search;
                    }
                    2 => {
                        if let Some(handle) = self.handle.clone() {
                            self.dispatch(Action::LoadProfile(handle));
                        }
                    }
                    3 => {
                        self.screen = Screen::Preferences;
                    }
                    4 => {
                        self.screen = Screen::Notifications;
                        if self.notifications.is_empty() {
                            self.dispatch(Action::RefreshNotifications);
                        }
                    }
                    _ => {}
                }
            }

            Action::OpenComposer {
                reply_to,
                reply_to_author,
            } => {
                self.composer = Composer::new();
                self.composer.set_reply(reply_to, reply_to_author);
                self.show_composer = true;
            }

            Action::CloseComposer => {
                self.show_composer = false;
            }

            Action::SubmitPost { text, reply_to } => {
                self.show_composer = false;
                let client = self.client.clone();
                let tx = self.action_tx.clone();
                tokio::spawn(async move {
                    match client.create_post(text, reply_to).await {
                        Ok(uri) => {
                            let _ = tx.send(Action::PostCreated(uri));
                        }
                        Err(e) => {
                            let _ = tx.send(Action::Error(e.to_string()));
                        }
                    }
                });
            }

            Action::PostCreated(_uri) => {
                self.dispatch(Action::RefreshTimeline);
            }

            Action::ToggleLike => {
                let post = match self.screen {
                    Screen::Timeline => self.timeline.selected_post().cloned(),
                    Screen::Thread => self.move_around_thread().cloned(),
                    Screen::Profile => self.profile_feed.selected_post().cloned(),
                    _ => None,
                };

                if let Some(post) = post {
                    let client = self.client.clone();
                    let tx = self.action_tx.clone();
                    if post.is_liked {
                        if let Some(like_uri) = post.like_uri.clone() {
                            let post_uri = post.uri.clone();
                            tokio::spawn(async move {
                                match client.unlike(&like_uri).await {
                                    Ok(_) => {
                                        let _ = tx.send(Action::UnlikeSuccess { post_uri });
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Action::Error(e.to_string()));
                                    }
                                }
                            });
                        }
                    } else {
                        let uri = post.uri.clone();
                        let cid = post.cid.clone();
                        tokio::spawn(async move {
                            match client.like(&uri, &cid).await {
                                Ok(like_uri) => {
                                    let _ = tx.send(Action::LikeSuccess {
                                        post_uri: uri,
                                        like_uri,
                                    });
                                }
                                Err(e) => {
                                    let _ = tx.send(Action::Error(e.to_string()));
                                }
                            }
                        });
                    }
                }
            }

            Action::LikeSuccess { post_uri, like_uri } => {
                self.update_post(&post_uri, |p| {
                    p.is_liked = true;
                    p.like_uri = Some(like_uri.clone());
                    p.likes += 1;
                });
            }

            Action::UnlikeSuccess { post_uri } => {
                self.update_post(&post_uri, |p| {
                    p.is_liked = false;
                    p.like_uri = None;
                    p.likes = (p.likes - 1).max(0);
                });
            }

            Action::ToggleRepost => {
                let post = match self.screen {
                    Screen::Timeline => self.timeline.selected_post().cloned(),
                    Screen::Thread => self.move_around_thread().cloned(),
                    Screen::Profile => self.profile_feed.selected_post().cloned(),
                    _ => None,
                };

                if let Some(post) = post {
                    let client = self.client.clone();
                    let tx = self.action_tx.clone();
                    if post.is_reposted {
                        if let Some(repost_uri) = post.repost_uri.clone() {
                            let post_uri = post.uri.clone();
                            tokio::spawn(async move {
                                match client.unrepost(&repost_uri).await {
                                    Ok(_) => {
                                        let _ = tx.send(Action::UnrepostSuccess { post_uri });
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Action::Error(e.to_string()));
                                    }
                                }
                            });
                        }
                    } else {
                        let uri = post.uri.clone();
                        let cid = post.cid.clone();
                        tokio::spawn(async move {
                            match client.repost(&uri, &cid).await {
                                Ok(repost_uri) => {
                                    let _ = tx.send(Action::RepostSuccess {
                                        post_uri: uri,
                                        repost_uri,
                                    });
                                }
                                Err(e) => {
                                    let _ = tx.send(Action::Error(e.to_string()));
                                }
                            }
                        });
                    }
                }
            }

            Action::RepostSuccess {
                post_uri,
                repost_uri,
            } => {
                self.update_post(&post_uri, |p| {
                    p.is_reposted = true;
                    p.repost_uri = Some(repost_uri.clone());
                    p.reposts += 1;
                });
            }

            Action::UnrepostSuccess { post_uri } => {
                self.update_post(&post_uri, |p| {
                    p.is_reposted = false;
                    p.repost_uri = None;
                    p.reposts = (p.reposts - 1).max(0);
                });
            }

            Action::ViewAuthorProfile => {
                let did = match self.screen {
                    Screen::Timeline => self.timeline.selected_post().map(|p| p.did.clone()),
                    Screen::Thread => self.move_around_thread().map(|p| p.did.clone()),
                    _ => None,
                };
                if let Some(did) = did {
                    self.screen_stack.push(self.screen.clone());
                    self.dispatch(Action::LoadProfile(did));
                }
            }

            Action::LoadProfile(actor) => {
                self.screen = Screen::Profile;
                self.profile = None;
                self.profile_feed = FeedState::new();
                self.profile_feed.loading = true;
                let client = self.client.clone();
                let tx = self.action_tx.clone();
                self.spawn_load(async move {
                    let profile_result = client.get_profile(&actor).await;
                    let feed_result = client.get_author_feed(&actor, None).await;
                    match (profile_result, feed_result) {
                        (Ok(profile), Ok((posts, cursor))) => {
                            let _ = tx.send(Action::ProfileLoaded {
                                profile,
                                posts,
                                cursor,
                            });
                        }
                        (Err(e), _) | (_, Err(e)) => {
                            let _ = tx.send(Action::Error(e.to_string()));
                        }
                    }
                });
            }

            Action::ProfileLoaded {
                profile,
                posts,
                cursor,
            } => {
                self.profile = Some(profile);
                self.profile_feed.replace_posts(posts, cursor);
            }

            Action::SavePreferences(prefs) => {
                if let Err(e) = prefs.save() {
                    error!("Something went wrong: {}", e);
                } else {
                    self.preferences = prefs;
                    self.dispatch(Action::RefreshTimeline);
                }
            }

            Action::TogglePreferences => {
                match self.preferences_selected_index {
                    1 => self.preferences.hide_replies = !self.preferences.hide_replies,
                    2 => {
                        self.preferences.hide_replies_by_unfollowed =
                            !self.preferences.hide_replies_by_unfollowed
                    }
                    3 => self.preferences.hide_reposts = !self.preferences.hide_reposts,
                    4 => self.preferences.hide_quote_posts = !self.preferences.hide_quote_posts,
                    5 => self.preferences.notify_likes = !self.preferences.notify_likes,
                    6 => self.preferences.notify_reposts = !self.preferences.notify_reposts,
                    7 => self.preferences.notify_follows = !self.preferences.notify_follows,
                    8 => self.preferences.notify_mentions = !self.preferences.notify_mentions,
                    9 => self.preferences.notify_replies = !self.preferences.notify_replies,
                    10 => self.preferences.notify_quotes = !self.preferences.notify_quotes,
                    11 => {
                        self.preferences.notify_starterpack_joins =
                            !self.preferences.notify_starterpack_joins
                    }
                    _ => {}
                }
                if let Err(e) = self.preferences.save() {
                    error!("Failed to save preferences: {}", e);
                }
            }

            Action::FocusSearchInput => {
                self.search_focused = true;
            }

            Action::Search(query) => {
                self.search_query = query.clone();
                self.search_feed = FeedState::new();
                self.search_feed.loading = true;
                let client = self.client.clone();
                let tx = self.action_tx.clone();
                self.spawn_load(async move {
                    match client.search_firehose(query, None).await {
                        Ok((posts, cursor)) => {
                            let _ = tx.send(Action::SearchResults {
                                posts,
                                cursor,
                                append: false,
                            });
                        }
                        Err(e) => {
                            let _ = tx.send(Action::Error(e.to_string()));
                        }
                    }
                });
            }

            Action::LoadMoreResults => {
                if self.search_feed.loading || self.search_feed.cursor.is_none() {
                    return;
                }
                self.search_feed.loading = true;
                let client = self.client.clone();
                let cursor = self.search_feed.cursor.clone();
                let query = self.search_query.clone();
                let tx = self.action_tx.clone();
                self.spawn_load(async move {
                    match client.search_firehose(query, cursor).await {
                        Ok((posts, cursor)) => {
                            let _ = tx.send(Action::SearchResults {
                                posts,
                                cursor,
                                append: true,
                            });
                        }
                        Err(e) => {
                            let _ = tx.send(Action::Error(e.to_string()));
                        }
                    }
                });
            }

            Action::SearchResults {
                posts,
                cursor,
                append,
            } => {
                if append {
                    self.search_feed.append_posts(posts, cursor);
                } else {
                    self.search_feed.replace_posts(posts, cursor);
                }
            }

            Action::RefreshNotifications => {
                self.notifications_loading = true;
                let client = self.client.clone();
                let tx = self.action_tx.clone();
                self.spawn_load(async move {
                    match client.get_notifications(None).await {
                        Ok((notifications, cursor)) => {
                            let _ = tx.send(Action::NotificationsLoaded {
                                notifications,
                                cursor,
                                append: false,
                            });
                        }
                        Err(e) => {
                            let _ = tx.send(Action::Error(e.to_string()));
                        }
                    }
                });
            }

            Action::LoadMoreNotifications => {
                if self.notifications_loading || self.notifications_cursor.is_none() {
                    return;
                }
                self.notifications_loading = true;
                let client = self.client.clone();
                let cursor = self.notifications_cursor.clone();
                let tx = self.action_tx.clone();
                self.spawn_load(async move {
                    match client.get_notifications(cursor).await {
                        Ok((notifications, cursor)) => {
                            let _ = tx.send(Action::NotificationsLoaded {
                                notifications,
                                cursor,
                                append: true,
                            });
                        }
                        Err(e) => {
                            let _ = tx.send(Action::Error(e.to_string()));
                        }
                    }
                });
            }

            Action::NotificationsLoaded {
                notifications,
                cursor,
                append,
            } => {
                self.notifications_loading = false;
                if append {
                    self.notifications.extend(notifications);
                    self.notifications_cursor = cursor;
                } else {
                    self.notifications = notifications;
                    self.notifications_cursor = cursor;
                    self.current_notification = 0;
                }
            }

            Action::Error(msg) => {
                error!("Error: {}", msg);
                self.error_message = Some(msg);
            }

            Action::ClearError => {
                self.error_message = None;
            }

            Action::ImageLoaded {
                post_uri,
                protocol,
                cols,
                rows,
            } => {
                self.image_protocols.insert(
                    post_uri,
                    ImageState {
                        protocol,
                        cols,
                        rows,
                    },
                );
            }

            Action::ReplyTimeout => {
                if let Some(action) = self.pending_action.take() {
                    self.dispatch(action);
                }
            }

            _ => {}
        }
    }

    fn update_post(&mut self, uri: &str, f: impl Fn(&mut crate::models::post::PostViewModel)) {
        for post in &mut self.timeline.posts {
            if post.uri == uri {
                f(post);
            }
        }
        for post in &mut self.profile_feed.posts {
            if post.uri == uri {
                f(post);
            }
        }
        for post in &mut self.search_feed.posts {
            if post.uri == uri {
                f(post);
            }
        }
        if let Some(ref mut thread) = self.thread {
            if thread.focal.uri == uri {
                f(&mut thread.focal);
            }
            for post in &mut thread.parents {
                if post.uri == uri {
                    f(post);
                }
            }
            for post in &mut thread.replies {
                if post.uri == uri {
                    f(post);
                }
            }
        }
    }

    fn draw(&mut self, frame: &mut ratatui::Frame) {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(area);

        // Tab bar
        crate::ui::tabs::draw_tabs(frame, chunks[0], self.active_tab);

        // Main content
        match self.screen {
            Screen::Login => {
                self.login_form.draw(frame, chunks[1]);
            }
            Screen::Timeline => {
                crate::ui::timeline::draw_timeline(frame, chunks[1], &self.timeline);
            }
            Screen::Thread => {
                crate::ui::thread::draw_thread(
                    frame,
                    chunks[1],
                    self.thread.as_ref(),
                    self.thread_cursor,
                    self.thread_scroll_offset,
                    &mut self.image_protocols,
                );
            }
            Screen::Profile => {
                crate::ui::profile::draw_profile(
                    frame,
                    chunks[1],
                    self.profile.as_ref(),
                    &self.profile_feed,
                );
            }
            Screen::Preferences => {
                crate::ui::user_prefs::draw_settings(
                    frame,
                    chunks[1],
                    &self.preferences,
                    self.preferences_selected_index,
                );
            }
            Screen::Search => {
                crate::ui::search::draw_search(
                    frame,
                    chunks[1],
                    &self.search_feed,
                    &self.search_query,
                    self.search_focused,
                );
            }
            Screen::Notifications => {
                crate::ui::notifications::draw_notifications(
                    frame,
                    chunks[1],
                    &self.notifications,
                    self.current_notification,
                    self.notifications_loading,
                );
            }
        }

        // Status bar
        crate::ui::statusbar::draw_statusbar(
            frame,
            chunks[2],
            &self.screen,
            self.show_composer,
            self.error_message.as_deref(),
        );

        // Composer overlay
        if self.show_composer {
            self.composer.draw(frame, area);
        }

        // Confirm dialog overlay
        if let Some(ref dialog) = self.logout_confirmation {
            dialog.draw(frame, area);
        }

        // Help dialog overlay
        if self.show_help {
            crate::ui::help::draw_help(frame, area);
        }
    }
}
