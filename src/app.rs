use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::prelude::*;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::error;

use crate::action::Action;
use crate::api::auth;
use crate::api::client::{BlueskyClient, ReplyRef};
use crate::api::session;
use crate::event::{self, EventHandler};
use crate::models::feed::FeedState;
use crate::models::profile::ProfileViewModel;
use crate::models::thread::ThreadViewModel;
use crate::tui::Tui;
use crate::ui::Component;
use crate::ui::composer::Composer;
use crate::ui::login::LoginForm;
use crate::utils::meta::ImageLibrary;
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
}

pub struct App {
    should_quit: bool,
    screen: Screen,
    screen_stack: Vec<Screen>,
    active_tab: usize,
    client: Arc<BlueskyClient>,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    handle: Option<String>,

    // State
    timeline: FeedState,
    thread: Option<ThreadViewModel>,
    profile: Option<ProfileViewModel>,
    profile_feed: FeedState,
    error_message: Option<String>,

    // Active data-loading task (aborted when a new load starts or on navigation)
    active_load: Option<JoinHandle<()>>,

    // Modals
    login_form: LoginForm,
    composer: Composer,
    show_composer: bool,

    image_library: ImageLibrary,
    image_protocols: HashMap<String, ImageState>,
}

impl App {
    pub fn new(
        handle: Option<String>,
        _prefer_app_password: bool,
        client: Arc<BlueskyClient>,
    ) -> Self {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let default_handle = handle.clone().or_else(|| session::get_last_handle());

        App {
            should_quit: false,
            screen: Screen::Login,
            screen_stack: Vec::new(),
            active_tab: 0,
            client,
            action_tx,
            action_rx,
            handle: handle.clone(),
            timeline: FeedState::new(),
            thread: None,
            profile: None,
            profile_feed: FeedState::new(),
            error_message: None,
            active_load: None,
            login_form: LoginForm::new(default_handle),
            composer: Composer::new(),
            show_composer: false,
            image_library: ImageLibrary::new(),
            image_protocols: std::collections::HashMap::new(),
        }
    }

    pub async fn run(&mut self, terminal: &mut Tui) -> Result<()> {
        // Try to restore session
        match auth::try_restore_session(&self.client).await {
            auth::AuthResult::Success(handle) => {
                self.screen = Screen::Timeline;
                self.handle = Some(handle);
                self.dispatch(Action::RefreshTimeline);
            }
            auth::AuthResult::NeedsLogin => {
                self.screen = Screen::Login;
            }
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

                // Global key handling
                if let Some(action) =
                    event::key_to_action(key, self.show_composer, self.screen == Screen::Login)
                {
                    // Special handling for 'r' to populate reply_to
                    let action = if matches!(key.code, KeyCode::Char('r'))
                        && key.modifiers == KeyModifiers::NONE
                    {
                        self.make_reply_action().unwrap_or(Action::OpenComposer {
                            reply_to: None,
                            reply_to_author: None,
                        })
                    } else {
                        action
                    };
                    self.dispatch(action);
                }
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
    }

    fn make_reply_action(&self) -> Option<Action> {
        let post = match self.screen {
            Screen::Timeline => self.timeline.selected_post(),
            Screen::Thread => self.thread.as_ref().map(|t| &t.focal),
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
            reply_to_author: Some(p.author_display_name.clone()),
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
            Screen::Thread => self.thread.as_ref().map(|t| &t.focal),
            _ => None,
        };

        let post = match post {
            Some(p) => p,
            None => return,
        };

        let embed = match &post.embed_summary {
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

        self.spawn_load(async move {
            match library.retrieve_or_download(&url).await {
                Ok(path) => {
                    if let Ok(data) = std::fs::read(&path) {
                        if let Ok(dyn_img) = image::load_from_memory(&data) {
                            let mut picker = ratatui_image::picker::Picker::from_query_stdio();
                            picker.as_mut()
                                .expect("REASON")
                                .set_protocol_type(ratatui_image::picker::ProtocolType::Kitty);
                            let font_size = picker.as_mut().expect("REASON").font_size();
                            let cols = (dyn_img.width() / font_size.0 as u32).max(1) as u16;
                            let rows = (dyn_img.height() / font_size.1 as u32).max(1) as u16;
                            let protocol = picker.as_mut().expect("REASON").new_resize_protocol(dyn_img);
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
                    match auth::login_with_app_password(&client, &handle, &password).await {
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
                let _ = auth::logout();
                self.handle = None;
                self.screen = Screen::Login;
                self.timeline = FeedState::new();
            }

            Action::RefreshTimeline => {
                self.timeline.loading = true;
                let client = self.client.clone();
                let tx = self.action_tx.clone();
                self.spawn_load(async move {
                    match client.get_timeline(None, Some(50u8)).await {
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
                self.spawn_load(async move {
                    match client.get_timeline(cursor, Some(50u8)).await {
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
                self.load_selected_post_images();
            }

            Action::SelectNext => match self.screen {
                Screen::Timeline => {
                    self.timeline.select_next();
                    self.load_selected_post_images();
                    if self.timeline.near_bottom(20) {
                        self.dispatch(Action::LoadMoreTimeline);
                    }
                }
                Screen::Profile => {
                    self.profile_feed.select_next();
                    self.load_selected_post_images();
                }
                Screen::Thread => {
                    // Thread navigation is handled at draw level
                }
                _ => {}
            },

            Action::SelectPrev => match self.screen {
                Screen::Timeline => {
                    self.timeline.select_prev();
                    self.load_selected_post_images();
                }
                Screen::Profile => {
                    self.profile_feed.select_prev();
                    self.load_selected_post_images();
                }
                _ => {}
            },

            Action::ScrollToTop => match self.screen {
                Screen::Timeline => self.timeline.select_first(),
                Screen::Profile => self.profile_feed.select_first(),
                _ => {}
            },

            Action::ScrollToBottom => match self.screen {
                Screen::Timeline => self.timeline.select_last(),
                Screen::Profile => self.profile_feed.select_last(),
                _ => {}
            },

            Action::OpenThread => {
                let uri = match self.screen {
                    Screen::Timeline => self.timeline.selected_post().map(|p| p.uri.clone()),
                    Screen::Profile => self.profile_feed.selected_post().map(|p| p.uri.clone()),
                    _ => None,
                };

                if let Some(uri) = uri {
                    let client = self.client.clone();
                    let tx = self.action_tx.clone();
                    self.screen_stack.push(self.screen.clone());
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
                        if self.timeline.posts.is_empty() {
                            self.dispatch(Action::RefreshTimeline);
                        }
                    }
                    1 => {
                        if let Some(handle) = self.handle.clone() {
                            self.dispatch(Action::LoadProfile(handle));
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
                    Screen::Thread => self.thread.as_ref().map(|t| t.focal.clone()),
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
                    p.like_count += 1;
                });
            }

            Action::UnlikeSuccess { post_uri } => {
                self.update_post(&post_uri, |p| {
                    p.is_liked = false;
                    p.like_uri = None;
                    p.like_count = (p.like_count - 1).max(0);
                });
            }

            Action::ToggleRepost => {
                let post = match self.screen {
                    Screen::Timeline => self.timeline.selected_post().cloned(),
                    Screen::Thread => self.thread.as_ref().map(|t| t.focal.clone()),
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
                    p.repost_count += 1;
                });
            }

            Action::UnrepostSuccess { post_uri } => {
                self.update_post(&post_uri, |p| {
                    p.is_reposted = false;
                    p.repost_uri = None;
                    p.repost_count = (p.repost_count - 1).max(0);
                });
            }

            Action::ViewAuthorProfile => {
                let did = match self.screen {
                    Screen::Timeline => self.timeline.selected_post().map(|p| p.author_did.clone()),
                    Screen::Thread => self.thread.as_ref().map(|t| t.focal.author_did.clone()),
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
                crate::ui::timeline::draw_timeline(
                    frame,
                    chunks[1],
                    &self.timeline,
                    &mut self.image_protocols,
                );
            }
            Screen::Thread => {
                crate::ui::thread::draw_thread(
                    frame,
                    chunks[1],
                    self.thread.as_ref(),
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
    }
}
