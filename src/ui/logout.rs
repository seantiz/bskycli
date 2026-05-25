use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap};

use crate::action::Action;
use crate::ui::Component;

pub struct Dialog {
    pub message: String,
}

impl Dialog {
    pub fn new(message: &str) -> Self {
        Dialog {
            message: message.to_string(),
        }
    }
}

impl Component for Dialog {
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Action> {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Esc) => Some(Action::LogoutCancelled),
            (KeyModifiers::NONE, KeyCode::Enter) => Some(Action::DefinitelyLogout),
            _ => None,
        }
    }

    fn draw(&self, frame: &mut Frame, area: Rect) {
        let popup_width = 40;
        let popup_height = 6;

        let horizontal = (area.width.saturating_sub(popup_width)) / 2;
        let vertical = (area.height.saturating_sub(popup_height)) / 2;

        let rect = Rect::new(
            area.x + horizontal,
            area.y + vertical,
            popup_width,
            popup_height,
        );

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Are you sure? ")
            .padding(Padding::new(1, 1, 1, 1));
        let content = Paragraph::new(self.message.as_str())
            .centered()
            .wrap(Wrap { trim: true });

        frame.render_widget(Clear, rect);
        frame.render_widget(&block, rect);
        frame.render_widget(content, block.inner(rect));
    }
}
