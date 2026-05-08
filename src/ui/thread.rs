use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::models::thread::ThreadViewModel;
use crate::ui::post_widget;
use crate::app::ImageState;

pub fn draw_thread(
    frame: &mut Frame,
    area: Rect,
    thread: Option<&ThreadViewModel>,
    image_protocols: &mut std::collections::HashMap<String, ImageState>,
) {
    let thread = match thread {
        Some(t) => t,
        None => {
            let loading = Paragraph::new("Loading thread...")
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center);
            frame.render_widget(loading, area);
            return;
        }
    };
    let mut y = area.y;
    let max_y = area.bottom();

    for parent in &thread.parents {
        if y >= max_y {
            break;
        }
        let image_rows = image_protocols.get(&parent.uri).map(|s| s.rows);
        let h = post_widget::post_height(parent, area.width, image_rows).min(max_y - y);
        let post_area = Rect::new(area.x, y, area.width, h);
        let protocol = image_protocols.get_mut(&parent.uri);
        post_widget::draw_post(frame, post_area, parent, false, protocol);
        y += h;
        if y < max_y {
            let connector = Paragraph::new("│").style(Style::default().fg(Color::DarkGray));
            frame.render_widget(connector, Rect::new(area.x + 1, y, 1, 1));
            y += 1;
        }
    }

    if y < max_y {
        let image_rows = image_protocols.get(&thread.focal.uri).map(|s| s.rows);
        let h = post_widget::post_height(&thread.focal, area.width, image_rows).min(max_y - y);
        let post_area = Rect::new(area.x, y, area.width, h);
        let protocol = image_protocols.get_mut(&thread.focal.uri);
        post_widget::draw_post(frame, post_area, &thread.focal, true, protocol);
        y += h;
    }

    if y < max_y {
        let sep = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray));
        frame.render_widget(sep, Rect::new(area.x, y, area.width, 1));
        y += 1;
    }

    if y < max_y && !thread.replies.is_empty() {
        let header = Paragraph::new(format!(
            " {} {}",
            thread.replies.len(),
            if thread.replies.len() == 1 { "reply" } else { "replies" }
        ))
        .style(Style::default().fg(Color::Gray));
        frame.render_widget(header, Rect::new(area.x, y, area.width, 1));
        y += 1;
    }

    for reply in &thread.replies {
        if y >= max_y {
            break;
        }
        let indented_x = area.x + 2;
        let indented_w = area.width.saturating_sub(2);
        let image_rows = image_protocols.get(&reply.uri).map(|s| s.rows);
        let h = post_widget::post_height(reply, indented_w, image_rows).min(max_y - y);
        let reply_area = Rect::new(indented_x, y, indented_w, h);
        let protocol = image_protocols.get_mut(&reply.uri);
        post_widget::draw_post(frame, reply_area, reply, false, protocol);
        y += h;
    }
}
