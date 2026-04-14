mod equalizer;
mod help;
mod now_playing;
mod search;
mod tabs;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // tabs
            Constraint::Length(3), // search input
            Constraint::Min(5),   // track list
            Constraint::Length(7), // DJ control panel
            Constraint::Length(1), // keybinds footer
        ])
        .split(f.area());

    tabs::render(f, app, chunks[0]);
    search::render(f, app, chunks[1]);
    search::render_list(f, app, chunks[2]);
    now_playing::render(f, app, chunks[3]);
    render_footer(f, app, chunks[4]);

    if app.eq.show_panel {
        equalizer::render(f, &app.eq);
    }

    if app.show_help {
        help::render(f);
    }
}

fn render_footer(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    use ratatui::{
        style::{Color, Style},
        widgets::Paragraph,
    };

    let text = format!(
        " {} | Space:⏯  ↑↓:Nav  Tab:Switch  f:Fav  a:Queue  ←→:Seek  +/-:Vol  e:EQ  ?:Help  q:Quit",
        app.message
    );
    let footer = Paragraph::new(text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, area);
}
