use crate::models::notifications::NotificationViewModel;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Paragraph;

pub fn draw_notifications(
    frame: &mut Frame,
    area: Rect,
    notifications: &[NotificationViewModel],
    scrolled_past: usize,
    loading: bool,
) {
    if loading && notifications.is_empty() {
        frame.render_widget(
            Paragraph::new("Loading...")
                .style(Style::default().yellow())
                .centered(),
            area,
        );
        return;
    }

    if notifications.is_empty() {
        frame.render_widget(
            Paragraph::new("Nothing to see here!")
                .style(Style::default().dark_gray())
                .centered(),
            area,
        );
        return;
    }

    let viewport = area.height as usize;
    let scroll = if scrolled_past >= viewport {
        scrolled_past - viewport + 1
    } else {
        0
    };

    let mut y = area.y;
    for (i, notif) in notifications.iter().enumerate().skip(scroll) {
        if y >= area.bottom() {
            break;
        }

        let line = format!("{} {}", notif.handle, render(&notif.reason));

        let is_selected = i == scrolled_past;
        let style = if is_selected {
            Style::default().cyan()
        } else if notif.is_read {
            Style::default().dark_gray()
        } else {
            Style::default()
        };

        frame.render_widget(
            Paragraph::new(line).style(style),
            Rect::new(area.x, y, area.width, 1),
        );
        y += 1;
    }

    if loading {
        frame.render_widget(
            Paragraph::new("Loading more...")
                .style(Style::default().yellow())
                .centered(),
            Rect::new(area.x, y, area.width, 1),
        );
    }
}

fn render(reason: &str) -> &'static str {
    match reason {
        "like" => "liked your post",
        "repost" => "reposted you",
        "follow" => "followed you",
        "mention" => "mentioned you",
        "reply" => "replied to your post",
        "quote" => "quoted your post",
        "starterpack-joined" => "joined your starter pack",
        // NOTE: Hopefully impossible
        _ => "did stuff",
    }
}
