use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame) {
    let area = centered_rect(55, 70, f.area());

    let help_text = "\
  MSSIC - Terminal Music Player + DJ Control

  Navigation:
    ↑/↓  j/k       Di chuyen trong danh sach
    Tab             Chuyen tab (Search/Library/Queue)
    i               Nhap tu khoa tim kiem
    Enter           Phat bai hat duoc chon
    Esc             Thoat che do nhap

  DJ Control:
    Space           Play / Pause
    ←/→  h/l        Seek -10s / +10s
    +/=             Tang am luong (+5%)
    -               Giam am luong (-5%)
    s               Stop

  Library & Queue:
    f               Them/Xoa khoi Favorites
    a               Them vao Queue

  Other:
    ?               Hien/An Help nay
    q               Thoat MSSIC

  An '?' de dong Help.";

    f.render_widget(Clear, area);

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(help, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
