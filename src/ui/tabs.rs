use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Padding, Tabs as RatatuiTabs};
use ratatui_image::StatefulImage;

use crate::app::ImageState;

// NOTE: This now adds nerd font as a requirement on user end for icons
pub fn draw_tabs(frame: &mut Frame, area: Rect, active: usize, logo: &mut Option<ImageState>) {
    let titles = vec!["Timeline", "Search", "Profile", "\u{f013}", "\u{f0e0}"];

    if let Some(state) = logo {
        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([Constraint::Length(state.cols), Constraint::Min(0)])
            .split(area);

        let img_area = Rect {
            x: chunks[0].x,
            y: chunks[0].y,
            width: state.cols,
            height: state.rows.min(chunks[0].height),
        };
        frame.render_stateful_widget(StatefulImage::default(), img_area, &mut state.protocol);

        let tabs = RatatuiTabs::new(titles)
            .block(Block::default().padding(Padding::new(0, 0, 1, 0)))
            .select(active)
            .style(Style::default().dark_gray())
            .highlight_style(Style::default().cyan().bold());
        frame.render_widget(tabs, chunks[1]);

        // NOTE: In the unlikely event that a TTY-only setup will completely fail
    } else {
        let tabs = RatatuiTabs::new(titles)
            .block(
                Block::default()
                    .title_top(" Bluesky ")
                    .padding(Padding::new(0, 0, 1, 0)),
            )
            .select(active)
            .style(Style::default().dark_gray())
            .highlight_style(Style::default().cyan().bold());
        frame.render_widget(tabs, area);
    }
}
