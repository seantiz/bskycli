use ratatui::Frame;
use ratatui::style::Style;
use ratatui::text::{Text, Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap};
use ratatui::layout::Rect;

use crate::models::preferences::PreferencesViewModel;


// NOTE: Navigation through this view is directly clamped by Screen::Preferences arm in app.rs
pub fn draw_settings(frame: &mut Frame, area: Rect, preferences: &PreferencesViewModel, selected_index: usize) {
    let popup_width = 54;
    let popup_height = 24;

    let horizontal = (area.width.saturating_sub(popup_width)) / 2;
    let vertical = (area.height.saturating_sub(popup_height)) / 2;

    let rect = Rect::new(
        area.x + horizontal,
        area.y + vertical,
        popup_width,
        popup_height,
    );

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(""));

    let section_style = Style::default().cyan().bold();
    lines.push(Line::from(Span::styled(" Timeline ", section_style)));

    let timeline_items: [(usize, &str, bool); 4] = [
        (1, "Hide replies", preferences.hide_replies),
        (2, "Hide replies by people you don't follow", preferences.hide_replies_by_unfollowed),
        (3, "Hide reposts", preferences.hide_reposts),
        (4, "Hide quote posts", preferences.hide_quote_posts),
    ];
    for (idx, label, checked) in &timeline_items {
        let checkbox = if *checked { "x" } else { " " };
        let text = format!("  [{}] {}", checkbox, label);
        let style = if *idx == selected_index {
            Style::default().cyan()
        } else {
            Style::default()
        };
        lines.push(Line::from(Span::styled(text, style)));
    }

    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(" Notifications ", section_style)));

    let notify_items: [(usize, &str, bool); 7] = [
        (5, "Notify me when my posts are liked", preferences.notify_likes),
        (6, "Notify me of reposts", preferences.notify_reposts),
        (7, "Notify me when someone follows me", preferences.notify_follows),
        (8, "Notify me when someone mentions me", preferences.notify_mentions),
        (9, "Notify me of replies", preferences.notify_replies),
        (10, "Notify me when someone quotes my post", preferences.notify_quotes),
        (11, "Notify me when someone uses my starter pack", preferences.notify_starterpack_joins),
    ];
    for (idx, label, checked) in &notify_items {
        let checkbox = if *checked { "x" } else { " " };
        let text = format!("  [{}] {}", checkbox, label);
        let style = if *idx == selected_index {
            Style::default().cyan()
        } else {
            Style::default()
        };
        lines.push(Line::from(Span::styled(text, style)));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(" Other ", section_style)));

    let interface_items: [(usize, &str, bool); 1] = [
        (12, "Show action hints", preferences.show_hints),
    ];
    for (idx, label, checked) in &interface_items {
        let checkbox = if *checked { "x" } else { " " };
        let text = format!("  [{}] {}", checkbox, label);
        let style = if *idx == selected_index {
            Style::default().cyan()
        } else {
            Style::default()
        };
        lines.push(Line::from(Span::styled(text, style)));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " Press spacebar to toggle any setting ",
        Style::default().dim(),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Preferences ")
        .padding(Padding::new(1, 1, 1, 1));

    let paragraph = Paragraph::new(Text::from(lines))
        .left_aligned()
        .wrap(Wrap { trim: true });

    frame.render_widget(Clear, rect);
    frame.render_widget(&block, rect);
    frame.render_widget(paragraph, block.inner(rect));
}
