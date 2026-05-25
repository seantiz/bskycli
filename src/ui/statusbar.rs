use std::borrow::Cow;

use ratatui::{Frame, layout::Rect, style::Style, widgets::Paragraph};

use crate::app::Screen;

pub fn draw_statusbar(
    frame: &mut Frame,
    area: Rect,
    screen: &Screen,
    in_composer: bool,
    error: Option<&str>,
    show_quoted_hint: bool,
    show_hints: bool,
) {
    if let Some(err) = error {
        let error_bar =
            Paragraph::new(format!(" There was a problem: {}", err)).style(Style::default().red());
        frame.render_widget(error_bar, area);
        return;
    }

    if !show_hints {
        let bar = Paragraph::new(" ").style(Style::default().dark_gray());
        frame.render_widget(bar, area);
        return;
    }

    let hints: Cow<'_, str> = if in_composer {
        Cow::Borrowed("Enter: post | Esc: cancel")
    } else {
        match screen {
            Screen::Timeline => {
                Cow::Borrowed("n: new post | r: reply | rr: repost | l: like | R: refresh")
            }
            Screen::Thread => {
                let base = "r: reply | rr: repost | l: like | u: profile";
                if show_quoted_hint {
                    Cow::Owned(format!("{} | o: open quoted post", base))
                } else {
                    Cow::Borrowed(base)
                }
            }
            _ => Cow::Borrowed(""),
        }
    };

    let bar = Paragraph::new(format!(" {}", hints)).style(Style::default().dark_gray());
    frame.render_widget(bar, area);
}
