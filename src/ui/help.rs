use ratatui::Frame;
use ratatui::text::{Text, Line};
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap};
use ratatui::layout::Rect;
use ratatui::style::Stylize;

pub fn draw_help(frame: &mut Frame, area: Rect) {
    let popup_width = 48;
    let popup_height = 17;

    let horizontal = (area.width.saturating_sub(popup_width)) / 2;
    let vertical = (area.height.saturating_sub(popup_height)) / 2;

    let rect = Rect::new(
        area.x + horizontal,
        area.y + vertical,
        popup_width,
        popup_height,
    );

    let content = vec![
        Line::from(" Switch Views ").bold().centered(),
        Line::from(""),
        Line::from("  1        Timeline"),
        Line::from("  2        Search"),
        Line::from("  3        Profile"),
        Line::from("  4        Preferences"),
        Line::from("  5        Notifications"),
        Line::from(""),
        Line::from(" Press any number to go to that screen ").dim().centered(),
        Line::from(""),
        Line::from(" Misc ").bold().centered(),
        Line::from(""),
        Line::from("  Ctrl+L   Logout"),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Help ")
        .padding(Padding::new(1, 1, 1, 1));

    let paragraph = Paragraph::new(Text::from(content))
        .left_aligned()
        .wrap(Wrap { trim: true });

    frame.render_widget(Clear, rect);
    frame.render_widget(&block, rect);
    frame.render_widget(paragraph, block.inner(rect));
}
