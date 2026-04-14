use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;
use crate::types::{InputMode, Tab};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let (style, title) = match app.input_mode {
        InputMode::Normal => (
            Style::default().fg(Color::White),
            " An 'i' de tim nhac ",
        ),
        InputMode::Editing => (
            Style::default().fg(Color::Yellow),
            " Dang go... (Enter: tim, Esc: huy) ",
        ),
    };

    let input = Paragraph::new(app.search_query.as_str()).style(style).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(input, area);

    // Show cursor when editing
    if app.input_mode == InputMode::Editing {
        f.set_cursor_position((area.x + app.search_query.len() as u16 + 1, area.y + 1));
    }
}

pub fn render_list(f: &mut Frame, app: &mut App, area: Rect) {
    let list_data = app.current_list();
    let playing_id = app
        .player
        .current_track
        .as_ref()
        .map(|t| t.id.as_str());

    let items: Vec<ListItem> = list_data
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let is_playing = playing_id == Some(t.id.as_str());
            let prefix = if is_playing { "♫" } else { " " };
            let text = format!("{} {:>2}. {} [{}]", prefix, i + 1, t.title, t.duration_str);

            let style = if is_playing {
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(text).style(style)
        })
        .collect();

    let title = match app.current_tab {
        Tab::Search => format!(" Search Results ({}) ", app.search_results.len()),
        Tab::Library => format!(" Favorites ({}) ", app.library.len()),
        Tab::Queue => format!(" Queue ({}) ", app.queue.len()),
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" ▶ ");

    f.render_stateful_widget(list, area, &mut app.list_state);
}
