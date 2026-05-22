use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::models::preferences::PreferencesViewModel;

pub fn draw_settings(frame: &mut Frame, area: Rect, preferences: &PreferencesViewModel, selected_index: usize) {
    let lines = [
        String::new(),
        format!("[{}] Hide replies", if preferences.hide_replies { "x" } else { " " }),
        format!("[{}] Hide replies by people you don't follow", if preferences.hide_replies_by_unfollowed { "x" } else { " " }),
        format!("[{}] Hide reposts", if preferences.hide_reposts { "x" } else { " " }),
        format!("[{}] Hide quote posts", if preferences.hide_quote_posts { "x" } else { " " }),
        format!("[{}] Notify me when my posts are liked", if preferences.notify_likes { "x" } else { " " }),
        format!("[{}] Notify me of reposts", if preferences.notify_reposts { "x" } else { " " }),
        format!("[{}] Notify me when someone follows me", if preferences.notify_follows { "x" } else { " " }),
        format!("[{}] Notify me when someone mentions me", if preferences.notify_mentions { "x" } else { " " }),
        format!("[{}] Notify me of replies", if preferences.notify_replies { "x" } else { " " }),
        format!("[{}] Notify me when someone quotes my post", if preferences.notify_quotes { "x" } else { " " }),
        format!("[{}] Notify me when someone uses my starter pack", if preferences.notify_starterpack_joins { "x" } else { " " }),
    ];

    let mut text = Text::default();
    for (i, line) in lines.iter().enumerate() {
        if i == selected_index {
            text.lines.push(Line::from(Span::styled(line.clone(), Style::default().fg(Color::Cyan))));
        } else {
            text.lines.push(Line::from(line.clone()));
        }
    }

    let block = Block::default()
        .title(" Change your Timeline view ")
        .borders(Borders::ALL);

    let paragraph = Paragraph::new(text).block(block);

    frame.render_widget(paragraph, area);
}
