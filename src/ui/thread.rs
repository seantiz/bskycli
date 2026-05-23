use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::ImageState;
use crate::models::thread::ThreadViewModel;
use crate::ui::post_widget;

pub fn draw_thread(
    frame: &mut Frame,
    area: Rect,
    thread: Option<&ThreadViewModel>,
    thread_cursor: usize,
    origin: usize,
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
    // NOTE: area.y would be immutable otherwise
    let mut pixels_drawn = area.y;

    let ground_floor = area.bottom();

    // Draw any parents first
    for (i, parent) in thread.parents.iter().enumerate().skip(origin) {
        if pixels_drawn >= ground_floor {
            break;
        }

        let highlight = i + origin == thread_cursor;

        let image_rows = image_protocols.get(&parent.uri).map(|s| s.rows);
        let apartment = post_widget::post_height(parent, area.width, image_rows)
            .min(ground_floor - pixels_drawn);
        let interior = Rect::new(area.x, pixels_drawn, area.width, apartment);
        let images = image_protocols.get_mut(&parent.uri);

        post_widget::draw_post(frame, interior, parent, highlight, images);

        pixels_drawn += apartment;

        if pixels_drawn < ground_floor {
            let connector = Paragraph::new("│").style(Style::default().dark_gray());
            frame.render_widget(connector, Rect::new(area.x + 1, pixels_drawn, 1, 1));
            pixels_drawn += 1;
        }
    }

    // Parents are drawn, now the post we entered the thread view for
    if thread.parents.len() >= origin {
        if pixels_drawn < ground_floor {
            let highlight = thread.parents.len() == thread_cursor;

            let image_rows = image_protocols.get(&thread.focal.uri).map(|s| s.rows);
            let apartment = post_widget::post_height(&thread.focal, area.width, image_rows)
                .min(ground_floor - pixels_drawn);
            let interior = Rect::new(area.x, pixels_drawn, area.width, apartment);
            let protocol = image_protocols.get_mut(&thread.focal.uri);

            post_widget::draw_post(frame, interior, &thread.focal, highlight, protocol);

            pixels_drawn += apartment;
        }

        if pixels_drawn < ground_floor {
            let sep = Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().dark_gray());
            frame.render_widget(sep, Rect::new(area.x, pixels_drawn, area.width, 1));
            pixels_drawn += 1;
        }

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
        }
    }

    // Finally draw replies
    let replies_start = origin.saturating_sub(thread.parents.len() + 1);

    for (i, reply) in thread.replies.iter().enumerate().skip(replies_start) {

        if pixels_drawn >= ground_floor {
            break;
        }

        // Skip past the GLOBAL index and focal post
        let highlight = thread.parents.len() + 1 + i == thread_cursor;

        let image_rows = image_protocols.get(&reply.uri).map(|s| s.rows);

        // NOTE: Indent replies by 2
        let apartment = post_widget::post_height(reply, area.width.saturating_sub(2), image_rows)
            .min(ground_floor - pixels_drawn);
        let interior = Rect::new(area.x + 2, pixels_drawn, area.width.saturating_sub(2), apartment);
        let protocol = image_protocols.get_mut(&reply.uri);

        post_widget::draw_post(frame, interior, reply, highlight, protocol);

        pixels_drawn += apartment;
    }
}
