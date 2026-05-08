use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::models::feed::FeedState;
use crate::ui::post_widget;
use crate::app::ImageState;
use std::collections::HashMap;

pub fn draw_timeline(
    frame: &mut Frame,
    area: Rect,
    feed: &FeedState,
    image_state: &mut HashMap<String, ImageState>,
) {
    if feed.loading && feed.posts.is_empty() {
        let loading = Paragraph::new("Loading timeline...")
            .style(Style::default().yellow())
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::NONE));
        frame.render_widget(loading, area);
        return;
    }
    if feed.posts.is_empty() {
        let empty = Paragraph::new("No posts yet. Press R to refresh.")
            .style(Style::default().dark_gray())
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    let mut y = area.y;
    let max_y = area.bottom();
    let mut offset = feed.scroll_offset;
    let visible_height = area.height as usize;

    let mut cumulative_height: usize = 0;
    let mut selected_start: usize = 0;
    let mut selected_height: usize = 0;
    for (i, post) in feed.posts.iter().enumerate() {
        let image_rows = image_state.get(&post.uri).map(|s| s.rows);
        let h = post_widget::post_height(post, area.width, image_rows) as usize;
        if i == feed.selected_index {
            selected_start = cumulative_height;
            selected_height = h;
            break;
        }
        cumulative_height += h;
    }

    if selected_start < offset {
        offset = selected_start;
    } else if selected_start + selected_height > offset + visible_height {
        offset = (selected_start + selected_height).saturating_sub(visible_height);
    }

    let mut running_height: usize = 0;
    for (i, post) in feed.posts.iter().enumerate() {
        let image_rows = image_state.get(&post.uri).map(|s| s.rows);
        let h = post_widget::post_height(post, area.width, image_rows);
        if running_height + h as usize <= offset {
            running_height += h as usize;
            continue;
        }
        if y >= max_y {
            break;
        }
        let available_h = (max_y - y).min(h);
        let post_area = Rect::new(area.x, y, area.width, available_h);
        let selected = i == feed.selected_index;
        let protocol = image_state.get_mut(&post.uri);
        post_widget::draw_post(frame, post_area, post, selected, protocol);
        y += available_h;
        running_height += h as usize;
    }

    if feed.loading {
        if y < max_y {
            frame.render_widget(
                Paragraph::new("Loading more...")
                    .style(Style::default().yellow())
                    .alignment(Alignment::Center),
                Rect::new(area.x, y, area.width, 1),
            );
        }
    }
}
