use crate::models::feed::FeedState;
use crate::ui::post_widget;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};
use ratatui::style::Style;
use ratatui::widgets::Paragraph;

pub fn draw_search(frame: &mut Frame, feed: &FeedState, query: &str, focused: bool) {
    let container = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(frame.area());

    let search_bar = Paragraph::new(format!("Search: {}", query)).style(Style::default().cyan());
    frame.render_widget(search_bar, container[0]);
    if focused {
        // NOTE: This may be a guess, but search is around 8 characters in the head
        let cursor_x = container[0].x + 8 + query.len() as u16;
        let cursor_y = container[0].y;
        frame.set_cursor_position(Position::new(cursor_x, cursor_y));
    }

    let mut results_list = container[1];

    if query.is_empty() && feed.posts.is_empty() {
        frame.render_widget(
            Paragraph::new("Press Enter to search")
                .style(Style::default().dark_gray())
                .centered(),
            results_list,
        );
        return;
    }

    if feed.loading {
        frame.render_widget(
            Paragraph::new("Searching...")
                .style(Style::default().yellow())
                .centered(),
            results_list,
        );
        return;
    }

    if feed.posts.is_empty() && !query.is_empty() {
        frame.render_widget(
            Paragraph::new("We couldn't find anything. Try again?")
                .style(Style::default().dark_gray())
                .centered(),
            results_list,
        );
        return;
    }

    if feed.posts.is_empty() {
        return;
    }

    let dynamic_tower: Vec<usize> = feed
        .posts
        .iter()
        .map(|p| post_widget::post_height(p, results_list.width, None) as usize)
        .collect();

    let hovered_post = feed.selected_index;
    let viewport = results_list.height as usize;

    let tower_summit: usize = dynamic_tower[..hovered_post].iter().sum();
    let tower_height = dynamic_tower[hovered_post];

    let lookahead: usize = dynamic_tower.iter().skip(hovered_post + 1).take(2).sum();

    let start_lookback_from = hovered_post.saturating_sub(2);
    let lookback: usize = dynamic_tower[start_lookback_from..hovered_post]
        .iter()
        .sum();

    let mut scroll = feed.scroll_offset;
    if tower_summit < scroll.saturating_sub(lookback) {
        scroll = tower_summit.saturating_sub(lookback);
    } else if tower_summit + tower_height + lookahead > scroll + viewport {
        scroll = (tower_summit + tower_height + lookahead).saturating_sub(viewport);
    }

    let mut passed: usize = 0;
    for (i, meta) in feed.posts.iter().enumerate() {
        let h = dynamic_tower[i];

        if passed + h <= scroll {
            passed += h;
            continue;
        }

        if results_list.y >= results_list.bottom() {
            break;
        }

        let above_the_fold = (results_list.bottom() - results_list.y).min(h as u16);

        let post = Rect::new(
            results_list.x,
            results_list.y,
            results_list.width,
            above_the_fold,
        );
        post_widget::draw_post(frame, post, meta, i == hovered_post, None);
        results_list.y += above_the_fold;
        passed += h;
    }

    if feed.loading && results_list.y < results_list.bottom() {
        frame.render_widget(
            Paragraph::new("Loading more results...")
                .style(Style::default().yellow())
                .centered(),
            Rect::new(results_list.x, results_list.y, results_list.width, 1),
        );
    }
}
