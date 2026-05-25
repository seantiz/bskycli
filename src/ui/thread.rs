use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::ImageState;
use crate::models::thread::ThreadViewModel;
use crate::models::post::PostViewModel;
use crate::ui::post_widget;

pub fn draw_thread(
    frame: &mut Frame,
    area: Rect,
    thread: Option<&ThreadViewModel>,
    thread_cursor: usize,
    scroll_offset: usize,
    image_protocols: &mut std::collections::HashMap<String, ImageState>,
) {
    let thread = match thread {
        Some(t) => t,
        None => {
            let loading = Paragraph::new("One moment...")
                .style(Style::default().yellow())
                .centered();
            frame.render_widget(loading, area);
            return;
        }
    };

    let absolute_max = (area.height as f32 * 0.4) as u16;

    let cap = |rows: Option<u16>| -> Option<u16> { rows.map(|r| r.min(absolute_max)) };

    let mut pixels_drawn = area.y;
    let ground_floor = area.bottom();

    let all_posts: Vec<&PostViewModel> = thread
        .parents
        .iter()
        .chain(std::iter::once(&thread.focal))
        .chain(thread.replies.iter())
        .collect();

    let focal_index = thread.parents.len();

    let dynamic_tower: Vec<usize> = all_posts
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let image_rows = image_protocols.get(&p.uri).map(|s| s.rows);
            let width = if i > focal_index {
                area.width.saturating_sub(2)
            } else {
                area.width
            };
            post_widget::post_height(p, width, cap(image_rows)) as usize
        })
        .collect();

    let tower_summit: usize = dynamic_tower[..thread_cursor].iter().sum();
    let tower_height = dynamic_tower[thread_cursor];

    let lookahead: usize = dynamic_tower.iter().skip(thread_cursor + 1).take(2).sum();
    let start_lookback_from = thread_cursor.saturating_sub(2);
    let lookback: usize = dynamic_tower[start_lookback_from..thread_cursor]
        .iter()
        .sum();

    let mut scroll = scroll_offset;
    if tower_summit < scroll.saturating_sub(lookback) {
        scroll = tower_summit.saturating_sub(lookback);
    } else if tower_summit + tower_height + lookahead > scroll + area.height as usize {
        scroll = (tower_summit + tower_height + lookahead).saturating_sub(area.height as usize);
    }

    let mut passed: usize = 0;
    for (i, post) in all_posts.iter().enumerate() {
        let h = dynamic_tower[i];

        if passed + h <= scroll {
            passed += h;
            continue;
        }

        if pixels_drawn >= ground_floor {
            break;
        }

        let is_reply = i > focal_index;
        let is_focal = i == focal_index;

        let highlight = i == thread_cursor;
        let indent: u16 = if is_reply { 2 } else { 0 };
        let cell_width = area.width.saturating_sub(indent);
        let apartment = (h as u16).min(ground_floor - pixels_drawn);
        let interior = Rect::new(area.x + indent, pixels_drawn, cell_width, apartment);

        let mut image_state = image_protocols.get_mut(&post.uri);
        if let Some(ref mut s) = image_state {
            s.rows = s.rows.min(absolute_max);
        }

        post_widget::draw_post(frame, interior, post, highlight, image_state);
        pixels_drawn += apartment;
        passed += h;

        if !is_reply && !is_focal && pixels_drawn < ground_floor {
            let connector = Paragraph::new("│").style(Style::default().dark_gray());
            frame.render_widget(connector, Rect::new(area.x + 1, pixels_drawn, 1, 1));
            pixels_drawn += 1;
            passed += 1;
        }

        if is_focal && pixels_drawn < ground_floor {
            let sep = Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().dark_gray());
            frame.render_widget(sep, Rect::new(area.x, pixels_drawn, area.width, 1));
            pixels_drawn += 1;
            passed += 1;

            if pixels_drawn < ground_floor && !thread.replies.is_empty() {
                let header = Paragraph::new(format!(
                    " {} {}",
                    thread.replies.len(),
                    if thread.replies.len() == 1 {
                        "reply"
                    } else {
                        "replies"
                    }
                ))
                .style(Style::default().gray());
                frame.render_widget(header, Rect::new(area.x, pixels_drawn, area.width, 1));
                pixels_drawn += 1;
                passed += 1;
            }
        }
    }
}
