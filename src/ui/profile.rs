use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::models::feed::FeedState;
use crate::models::profile::ProfileViewModel;
use crate::ui::post_widget;

pub fn draw_profile(
    frame: &mut Frame,
    area: Rect,
    profile: Option<&ProfileViewModel>,
    feed: &FeedState,
) {
    let profile = match profile {
        Some(p) => p,
        None => {
            let loading = Paragraph::new("One moment...")
                .style(Style::default().yellow())
                .centered();

            frame.render_widget(loading, area);
            return;
        }
    };

    let container = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(1)])
        .split(area);

    draw_profile_header(frame, container[0], profile);
    draw_author_feed(frame, container[1], feed);
}

fn draw_profile_header(frame: &mut Frame, area: Rect, profile: &ProfileViewModel) {
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().dark_gray());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let header_div = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    // Display name
    frame.render_widget(
        Paragraph::new(profile.display_name.as_str()).style(Style::default().white().bold()),
        header_div[0],
    );

    // Handle
    frame.render_widget(
        Paragraph::new(format!("@{}", profile.handle)).style(Style::default().dark_gray()),
        header_div[1],
    );

    // Bio
    if !profile.description.is_empty() {
        frame.render_widget(
            Paragraph::new(profile.description.as_str())
                .style(Style::default().gray())
                .wrap(Wrap { trim: true }),
            header_div[2],
        );
    }

    // Stats
    let stats = Line::from(vec![
        Span::styled(
            format!("{}", profile.followers_count),
            Style::default().white().bold(),
        ),
        Span::styled(" followers  ", Style::default().dark_gray()),
        Span::styled(
            format!("{}", profile.follows_count),
            Style::default().white().bold(),
        ),
        Span::styled(" following  ", Style::default().dark_gray()),
        Span::styled(
            format!("{}", profile.posts_count),
            Style::default().white().bold(),
        ),
        Span::styled(" posts", Style::default().dark_gray()),
    ]);
    frame.render_widget(Paragraph::new(stats), header_div[3]);
}

fn draw_author_feed(frame: &mut Frame, area: Rect, feed: &FeedState) {
    if feed.loading && feed.posts.is_empty() {
        let loading = Paragraph::new("Loading...")
            .style(Style::default().yellow())
            .centered();

        frame.render_widget(loading, area);
        return;
    }

    if feed.posts.is_empty() {
        let empty = Paragraph::new("No posts")
            .style(Style::default().dark_gray())
            .centered();

        frame.render_widget(empty, area);
        return;
    }

    let dynamic_tower: Vec<usize> = feed
        .posts
        .iter()
        .map(|p| post_widget::post_height(p, area.width, None) as usize)
        .collect();

    let highlight = feed.selected_index;

    let penthouse: usize = dynamic_tower[..highlight].iter().sum();
    let tower = dynamic_tower[highlight];

    let lookahead: usize = dynamic_tower.iter().skip(highlight + 1).take(2).sum();

    let start_lookback_from = highlight.saturating_sub(2);
    let lookback: usize = dynamic_tower[start_lookback_from..highlight]
        .iter()
        .sum();

    let mut scroll = feed.scroll_offset;
    if penthouse < scroll.saturating_sub(lookback) {
        scroll = penthouse.saturating_sub(lookback);
    } else if penthouse + tower + lookahead > scroll + area.height as usize {
        scroll = (penthouse + tower + lookahead).saturating_sub(area.height as usize);
    }

    let mut y = area.y;
    let mut passed: usize = 0;
    for (i, post) in feed.posts.iter().enumerate() {
        let h = dynamic_tower[i];

        if passed + h <= scroll {
            passed += h;
            continue;
        }

        if y >= area.bottom() {
            break;
        }

        let above_the_fold = (area.bottom() - y).min(h as u16);
        let interior = Rect::new(area.x, y, area.width, above_the_fold);
        post_widget::draw_post(frame, interior, post, i == feed.selected_index, None);
        y += above_the_fold;
        passed += h;
    }
}
