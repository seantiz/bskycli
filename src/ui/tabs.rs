use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Tabs as RatatuiTabs};
use ratatui::Frame;
use ratatui::style::Style;

pub fn draw_tabs(frame: &mut Frame, area: Rect, active: usize) {
    // TODO: This now adds a hard requirement that the user have nerd font installed. 
    
    let titles = vec!["[1] Timeline", "[2] Profile", "[3] Preferences", "[4] Search", "\u{f0e0}"];
    let tabs = RatatuiTabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .title(" Bluesky "),
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
