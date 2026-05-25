use ratatui::layout::Rect;
use ratatui::widgets::{Block, Padding, Tabs as RatatuiTabs};
use ratatui::Frame;
use ratatui::style::Style;

pub fn draw_tabs(frame: &mut Frame, area: Rect, active: usize) {
    // NOTE: This now adds nerd font as a requirement on user end for icons
    
    let titles = vec!["Timeline", "Search", "Profile", "\u{f013}", "\u{f0e0}"];
    let tabs = RatatuiTabs::new(titles)
        .block(
            Block::default()
                .title_top(" Bluesky ")
                .padding(Padding::new(0, 0, 1, 0)),
        )
        .select(active)
        .style(Style::default().dark_gray())
        .highlight_style(
            Style::default()
                .cyan()
                .bold()
        );
    frame.render_widget(tabs, area);
}
