use chrono::{DateTime, Utc};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::models::notifications::NotificationViewModel;
use crate::utils::time::relative_time;

fn render(reason: &str) -> &'static str {
    match reason {
        "like" => "liked your post",
        "repost" => "reposted your post",
        "follow" => "followed you",
        "mention" => "mentioned you",
        "reply" => "replied to your post",
        "quote" => "quoted your post",
        "starterpack-joined" => "joined your starter pack",
        _ => "interacted",
    }
}

fn apartment_height(n: &NotificationViewModel, width: u16) -> u16 {
    let mut h = 3u16;
    if n.record.is_some() {
        h += 1;
    }
    h.min(width.saturating_sub(4).max(1))
}

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

    let apartments: Vec<u16> = notifications
        .iter()
        .map(|n| apartment_height(n, area.width))
        .collect();

    let viewport = area.height as usize;

    let roof: usize = apartments[..scrolled_past].iter().map(|h| *h as usize).sum();
    let tower_block = apartments[scrolled_past] as usize;

    let start_lookback = scrolled_past.saturating_sub(3);
    let lookback: usize = apartments[start_lookback..scrolled_past]
        .iter()
        .map(|h| *h as usize)
        .sum();

    let lookahead: usize = apartments
        .iter()
        .skip(scrolled_past + 1)
        .take(3)
        .map(|h| *h as usize)
        .sum();

    let mut scroll = 0usize;
    if roof < scroll.saturating_sub(lookback) {
        scroll = roof.saturating_sub(lookback);
    } else if roof + tower_block + lookahead > scroll + viewport {
        scroll = (roof + tower_block + lookahead).saturating_sub(viewport);
    }

    let mut y = area.y;
    let mut passed = 0usize;
    for (i, notif) in notifications.iter().enumerate() {
        let h = apartments[i] as usize;

        if passed + h <= scroll {
            passed += h;
            continue;
        }

        if y >= area.bottom() {
            break;
        }

        let is_selected = i == scrolled_past;
        let border_style = if is_selected {
            Style::default().cyan()
        } else {
            Style::default().dark_gray()
        };

        let block = Block::default()
            .borders(Borders::LEFT)
            .border_style(border_style);

        let card_area = Rect::new(area.x, y, area.width, h as u16);
        let inner = block.inner(card_area);
        frame.render_widget(block, card_area);

        let inner_y = inner.y;
        let inner_x = inner.x + 1;
        let inner_w = inner.width.saturating_sub(1);
        let inner_bottom = inner.bottom();

        let text_style = if notif.is_read {
            Style::default().dark_gray()
        } else {
            Style::default()
        };

        let mut ly = inner_y;

        if ly < inner_bottom {
            let time = DateTime::parse_from_rfc3339(&notif.indexed_at)
                .map(|dt| relative_time(&dt.with_timezone(&Utc)))
                .unwrap_or_default();

            let author_line = ratatui::text::Line::from(vec![
                ratatui::text::Span::styled(
                    &notif.display_name,
                    Style::default().white().bold(),
                ),
                ratatui::text::Span::styled(
                    format!("  @{}", notif.handle),
                    Style::default().dark_gray(),
                ),
                ratatui::text::Span::styled(format!("  {}", time), Style::default().dark_gray()),
            ]);
            frame.render_widget(Paragraph::new(author_line), Rect::new(inner_x, ly, inner_w, 1));
            ly += 1;
        }

        if ly < inner_bottom {
            let reason_text = format!("{}", render(&notif.reason));
            frame.render_widget(
                Paragraph::new(reason_text).style(text_style),
                Rect::new(inner_x, ly, inner_w, 1),
            );
            ly += 1;
        }

        if let Some(ref record) = notif.record {
            if ly < inner_bottom {
                let snippet: String = record.chars().take(60).collect();
                frame.render_widget(
                    Paragraph::new(snippet).style(Style::default().dark_gray()),
                    Rect::new(inner_x, ly, inner_w, 1),
                );
            }
        }

        y += h as u16;
        passed += h;
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
