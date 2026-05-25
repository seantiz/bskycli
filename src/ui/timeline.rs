use crate::models::feed::FeedState;
use crate::ui::post_widget;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn draw_timeline(frame: &mut Frame, mut area: Rect, feed: &FeedState) {
    if feed.loading && feed.posts.is_empty() {
        frame.render_widget(
            Paragraph::new("Loading...")
                .style(Style::default().yellow())
                .centered()
                .block(Block::default().borders(Borders::NONE)),
            area,
        );
        return;
    }

    if feed.posts.is_empty() {
        frame.render_widget(
            Paragraph::new("Press R to refresh.")
                .style(Style::default().dark_gray())
                .centered(),
            area,
        );
        return;
    }

    let heights: Vec<usize> = feed
        .posts
        .iter()
        .map(|p| post_widget::post_height(p, area.width, None) as usize)
        .collect();

    let hovered_post = feed.selected_index;
    let viewport = area.height as usize;

    let top_of_hp: usize = heights[..hovered_post].iter().sum();
    let height_of_hp = heights[hovered_post];

    let lookahead: usize = heights.iter().skip(hovered_post + 1).take(2).sum();

    let start_lookback_from = hovered_post.saturating_sub(2);
    let lookback: usize = heights[start_lookback_from..hovered_post].iter().sum();

    let mut scroll = feed.scroll_offset;
    if top_of_hp < scroll.saturating_sub(lookback) {
        scroll = top_of_hp.saturating_sub(lookback);
    } else if top_of_hp + height_of_hp + lookahead > scroll + viewport {
        scroll = (top_of_hp + height_of_hp + lookahead).saturating_sub(viewport);
    }

    let mut passed: usize = 0;
    for (i, meta) in feed.posts.iter().enumerate() {
        let h = heights[i];

        if passed + h <= scroll {
            passed += h;
            continue;
        }

        if area.y >= area.bottom() {
            break;
        }

        let above_fold = (area.bottom() - area.y).min(h as u16);
        let post = Rect::new(area.x, area.y, area.width, above_fold);
        post_widget::draw_post(frame, post, meta, i == hovered_post, None);
        area.y += above_fold;
        passed += h;
    }

    if feed.loading && area.y < area.bottom() {
        frame.render_widget(
            Paragraph::new("Loading more posts...")
                .style(Style::default().yellow()).centered(),
            Rect::new(area.x, area.y, area.width, 1),
        );
    }
}
