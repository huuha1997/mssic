use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Tabs as RatatuiTabs},
    Frame,
};

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let queue_title = format!(" [3] Queue ({}) ", app.queue.len());
    let titles = vec![" [1] Search ", " [2] Library ", queue_title.as_str()];

    let tabs = RatatuiTabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("  MSSIC  ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .select(app.current_tab.index())
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, area);
}
