use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::app::Screen;

pub fn draw_statusbar(
    frame: &mut Frame,
    area: Rect,
    screen: &Screen,
    in_composer: bool,
    error: Option<&str>,
) {
    if let Some(err) = error {
        let error_bar = Paragraph::new(format!(" There was a problem: {}", err))
            .style(Style::default().red());
        frame.render_widget(error_bar, area);
        return;
    }

    let hints = if in_composer {
        "Enter: post | Esc: cancel"
    } else {
        match screen {
            Screen::Login => "Tab to cycle and Escape to close",
            Screen::Timeline => {
                "n: new post | r: reply | rr: repost | l: like | R: refresh"
            }
            Screen::Thread => {
                "r: reply | rr: repost | l: like | u: profile"
            }
            Screen::Profile => {""}
            Screen::Preferences => {""},
            Screen::Search => {"/ to begin searching Bluesky"},
            Screen::Notifications => {""}
        }
    };

    let bar = Paragraph::new(format!(" {}", hints))
        .style(Style::default().dark_gray());
    frame.render_widget(bar, area);
}
