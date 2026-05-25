use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};

use futures::StreamExt;

use crate::action::Action;

pub struct EventHandler {
    stream: EventStream,
}

impl EventHandler {
    pub fn new() -> Self {
        EventHandler {
            stream: EventStream::new(),
        }
    }

    pub async fn next(&mut self) -> Option<Event> {
        loop {
            match self.stream.next().await {
                Some(Ok(event)) => return Some(event),
                Some(Err(_)) => continue,
                None => return None,
            }
        }
    }
}

pub struct DoubleTap {
    last_key: Option<char>,
    last_time: Option<Instant>,
}

impl DoubleTap {
    pub fn new() -> Self {
        Self {
            last_key: None,
            last_time: None,
        }
    }

    pub fn press(&mut self, key: char) -> bool {
        let now = Instant::now();
        let double_tap_window = Duration::from_millis(600);

        let is_double_tap = self.last_key == Some(key)
            && self
                .last_time
                .map(|t| now.duration_since(t) <= double_tap_window)
                .unwrap_or(false);

        self.last_key = Some(key);
        self.last_time = Some(now);

        is_double_tap
    }
}

pub fn key_to_action(key: KeyEvent, in_composer: bool, in_login: bool) -> Option<Action> {
    if in_composer || in_login {
        return None;
    }

    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => Some(Action::Quit),
        (KeyModifiers::NONE, KeyCode::Char('q')) => Some(Action::Quit),
        (KeyModifiers::NONE, KeyCode::Char('j')) | (KeyModifiers::NONE, KeyCode::Down) => {
            Some(Action::SelectNext)
        }
        (KeyModifiers::NONE, KeyCode::Char('k')) | (KeyModifiers::NONE, KeyCode::Up) => {
            Some(Action::SelectPrev)
        }
        (KeyModifiers::NONE, KeyCode::Char(' ')) => Some(Action::TogglePreferences),
        (KeyModifiers::NONE, KeyCode::Enter) => Some(Action::OpenThread),
        (KeyModifiers::NONE, KeyCode::Char(',')) => Some(Action::GoBack),
        (KeyModifiers::NONE, KeyCode::Char('n')) => Some(Action::OpenComposer {
            reply_to: None,
            reply_to_author: None,
        }),
        (KeyModifiers::NONE, KeyCode::Char('r')) => {
            // Reply - actual reply_to will be filled by the app
            Some(Action::OpenComposer {
                reply_to: None,
                reply_to_author: None,
            })
        }
        (KeyModifiers::CONTROL, KeyCode::Char('l')) => Some(Action::LogoutConfirm),
        (KeyModifiers::NONE, KeyCode::Char('l')) => Some(Action::ToggleLike),

        (KeyModifiers::NONE, KeyCode::Char('u')) => Some(Action::ViewAuthorProfile),
        (KeyModifiers::SHIFT, KeyCode::Char('R')) => Some(Action::RefreshTimeline),
        (KeyModifiers::NONE, KeyCode::Char('g')) => Some(Action::ScrollToTop),
        (KeyModifiers::SHIFT, KeyCode::Char('G')) => Some(Action::ScrollToBottom),
        (KeyModifiers::NONE, KeyCode::Char('1')) => Some(Action::SwitchTab(0)),
        (KeyModifiers::NONE, KeyCode::Char('2')) => Some(Action::SwitchTab(1)),
        (KeyModifiers::NONE, KeyCode::Char('3')) => Some(Action::SwitchTab(2)),
        (KeyModifiers::NONE, KeyCode::Char('4')) => Some(Action::SwitchTab(3)),
        (KeyModifiers::NONE, KeyCode::Char('5')) => Some(Action::SwitchTab(4)),
        (KeyModifiers::NONE, KeyCode::Char('?')) => Some(Action::ShowHelp),

        _ => None,
    }
}
