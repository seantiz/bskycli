use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};

use crate::app::ImageState;
use crate::models::post::{EmbedKind, PostViewModel, QuotedPost};
use crate::utils::text::{styled_text, wrapped_line_count};
use crate::utils::time::relative_time;

pub fn post_height(post: &PostViewModel, width: u16, image_rows: Option<u16>) -> u16 {
    let text_width = width.saturating_sub(4);
    let text_lines = wrapped_line_count(&post.text, text_width);

    let mut height = 1 + text_lines + 1 + 1;

    if post.replied_by.is_some() {
        height += 1;
    }
    if post.reposted_by.is_some() {
        height += 1;
    }
    if let Some(ref embed) = post.meta {
        match &embed.kind {
            EmbedKind::Images(_) => {
                height += image_rows.unwrap_or(1);
            }
            EmbedKind::Record(quoted) => {
                height += quoted_card_height(quoted, text_width);
            }
            EmbedKind::RecordWithMedia(quoted) => {
                height += quoted_card_height(quoted, text_width) + 1;
            }
            _ => {
                height += 1;
            }
        }
    }

    height
}

fn quoted_card_height(quoted: &QuotedPost, width: u16) -> u16 {
    let mut h = 3u16;
    if !quoted.text.is_empty() {
        h += wrapped_line_count(&quoted.text, width.saturating_sub(2)).min(3);
    }
    if quoted.meta.is_some() {
        h += 1;
    }
    h
}

pub fn draw_post(
    frame: &mut Frame,
    area: Rect,
    post: &PostViewModel,
    highlighted: bool,
    image_state: Option<&mut ImageState>,
) {
    let border_style = if highlighted {
        Style::default().cyan().on_white()
    } else {
        Style::default().gray()
    };

    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(border_style)
        .border_type(BorderType::Thick);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let mut y = inner.y;
    let x = inner.x + 1;
    let w = inner.width.saturating_sub(1);
    let bottom = inner.bottom();

    // Repost indicator
    if let Some(ref reposted_by) = post.reposted_by {
        if y >= bottom {
            return;
        }
        let repost_line = Line::from(vec![
            Span::styled("⟳ ", Style::default().green()),
            Span::styled(
                format!("Reposted by {}", reposted_by),
                Style::default().dark_gray(),
            ),
        ]);
        frame.render_widget(Paragraph::new(repost_line), Rect::new(x, y, w, 1));
        y += 1;
    }

    // Reply indicator
    if let Some(ref parent_author) = post.replied_by {
        if y >= bottom {
            return;
        }
        let reply_line = Line::from(vec![
            Span::styled("↩ ", Style::default().blue()),
            Span::styled(
                format!("Reply to {}", parent_author),
                Style::default().dark_gray(),
            ),
        ]);
        frame.render_widget(Paragraph::new(reply_line), Rect::new(x, y, w, 1));
        y += 1;
    }

    // Author line
    if y >= bottom {
        return;
    }
    let time_str = relative_time(&post.created_at);
    let author_line = Line::from(vec![
        Span::styled(&post.display_name, Style::default().bold()),
        Span::styled(
            format!("  @{}", post.handle),
            Style::default().dark_gray(),
        ),
        Span::styled(format!("  {}", time_str), Style::default().dark_gray()),
    ]);
    frame.render_widget(Paragraph::new(author_line), Rect::new(x, y, w, 1));
    y += 1;

    if y >= bottom {
        return;
    }
    let text_lines = styled_text(&post.text, &post.facets);
    let remaining = bottom.saturating_sub(y);
    let text_height = remaining.saturating_sub(2).max(1).min(remaining);
    frame.render_widget(
        Paragraph::new(text_lines).wrap(Wrap { trim: false }),
        Rect::new(x, y, w, text_height),
    );
    let wrap_lines = wrapped_line_count(&post.text, w);
    y += wrap_lines.min(remaining);

    if let Some(ref embed) = post.meta {
        if y < bottom {
            match &embed.kind {
                EmbedKind::Images(n) => {
                    if let Some(state) = image_state {
                        let h = (bottom - y).min(state.rows);
                        let img_area = Rect::new(x, y, state.cols.min(w), h);
                        frame.render_stateful_widget(
                            ratatui_image::StatefulImage::default(),
                            img_area,
                            &mut state.protocol,
                        );
                        y += h;
                    } else {
                        draw_embed_images(frame, x, y, w, n.len());
                        y += 1;
                    }
                }
                EmbedKind::Record(quoted) => {
                    y = draw_quoted_card(frame, x, y, w, bottom, quoted);
                }
                EmbedKind::RecordWithMedia(quoted) => {
                    y = draw_quoted_card(frame, x, y, w, bottom, quoted);
                    if y < bottom {
                        frame.render_widget(
                            Paragraph::new("📎 [media]").style(Style::default().dark_gray()),
                            Rect::new(x, y, w, 1),
                        );
                        y += 1;
                    }
                }
                _ => {
                    let embed_text = match (&embed.title, &embed.description) {
                        (Some(t), _) => format!("📎 {}", t),
                        (_, Some(d)) => format!("📎 {}", d),
                        _ => format!(
                            "📎 [{}]",
                            match embed.kind {
                                EmbedKind::ExternalLink => "link",
                                EmbedKind::Video => "video",
                                _ => unreachable!(),
                            }
                        ),
                    };
                    frame.render_widget(
                        Paragraph::new(embed_text).style(Style::default().dark_gray()),
                        Rect::new(x, y, w, 1),
                    );
                    y += 1;
                }
            }
        }
    }

    // Stats line
    if y < bottom {
        let like_style = if post.is_liked {
            Style::default().red()
        } else {
            Style::default().dark_gray()
        };
        let repost_style = if post.is_reposted {
            Style::default().green()
        } else {
            Style::default().dark_gray()
        };

        let stats = Line::from(vec![
            Span::styled(if post.is_liked { "♥ " } else { "♡ " }, like_style),
            Span::styled(format!("{}", post.likes), like_style),
            Span::raw("  "),
            Span::styled(if post.is_reposted { "⟳ " } else { "⟳ " }, repost_style),
            Span::styled(format!("{}", post.reposts), repost_style),
            Span::raw("  "),
            Span::styled(
                format!("Replies {}", post.replies),
                Style::default().dark_gray(),
            ),
        ]);
        frame.render_widget(Paragraph::new(stats), Rect::new(x, y, w, 1));
    }
}

fn draw_embed_images(frame: &mut Frame, x: u16, y: u16, w: u16, count: usize) {
    let text = format!("🖼 {} image{}", count, if count != 1 { "s" } else { "" });
    frame.render_widget(
        Paragraph::new(text).style(Style::default().dark_gray()),
        Rect::new(x, y, w, 1),
    );
}

fn draw_quoted_card(
    frame: &mut Frame,
    x: u16,
    y: u16,
    w: u16,
    bottom: u16,
    quoted: &QuotedPost,
) -> u16 {
    let mut cy = y;
    let dim = Style::default().dark_gray();

    if cy < bottom {
        frame.render_widget(
            Paragraph::new(Span::styled(" ─ ─ ─ ─ ─ ─ ─ ─ ─", dim)),
            Rect::new(x, cy, w, 1),
        );
        cy += 1;
    }

    if cy < bottom {
        let author = if quoted.handle.is_empty() {
            Line::from(Span::styled(&quoted.text, dim))
        } else {
            Line::from(vec![
                Span::styled(&quoted.display_name, dim),
                Span::styled(format!("  @{}", quoted.handle), dim),
            ])
        };
        frame.render_widget(Paragraph::new(author), Rect::new(x, cy, w, 1));
        cy += 1;
    }

    if cy < bottom && !quoted.text.is_empty() && !quoted.handle.is_empty() {
        let text_lines = styled_text(&quoted.text, &quoted.facets);
        let wrap_count = wrapped_line_count(&quoted.text, w.saturating_sub(2)).min(3);
        let text_height = (bottom - cy).min(wrap_count);
        frame.render_widget(
            Paragraph::new(text_lines).wrap(Wrap { trim: false }),
            Rect::new(x + 1, cy, w.saturating_sub(1), text_height),
        );
        cy += text_height;
    }

    if let Some(ref embed) = quoted.meta {
        if cy < bottom {
            let text = match &embed.kind {
                EmbedKind::ExternalLink => embed
                    .title
                    .as_deref()
                    .map(|t| format!("📎 {}", t))
                    .or_else(|| embed.description.as_deref().map(|d| format!("📎 {}", d)))
                    .unwrap_or_else(|| "📎 [link]".to_string()),
                EmbedKind::Images(imgs) => {
                    format!("🖼 {} image{}", imgs.len(), if imgs.len() != 1 { "s" } else { "" })
                }
                EmbedKind::Video => "📎 [video]".to_string(),
                EmbedKind::Record(_) => "📎 [quote]".to_string(),
                EmbedKind::RecordWithMedia(_) => "📎 [quote+media]".to_string(),
            };
            frame.render_widget(Paragraph::new(text).style(dim), Rect::new(x, cy, w, 1));
            cy += 1;
        }
    }

    if cy < bottom {
        frame.render_widget(
            Paragraph::new(Span::styled(" ─ ─ ─ ─ ─ ─ ─ ─ ─", dim)),
            Rect::new(x, cy, w, 1),
        );
        cy += 1;
    }

    cy
}
