use ratatui::{
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::equalizer::{Equalizer, BAND_FREQS, BAND_LABELS, NUM_BANDS};

pub fn render(f: &mut Frame, eq: &Equalizer) {
    let area = centered_rect(80, 80, f.area());
    f.render_widget(Clear, area);

    let enabled_str = if eq.enabled { "ON" } else { "OFF" };
    let title = format!(" Equalizer [{}]  Preset: {} ", enabled_str, eq.preset_name());

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height < 8 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // +12dB label
            Constraint::Min(6),   // EQ bars
            Constraint::Length(1), // -12dB label
            Constraint::Length(1), // band labels
            Constraint::Length(1), // freq labels
            Constraint::Length(1), // gain values
            Constraint::Length(1), // spacer
            Constraint::Length(1), // controls 1
            Constraint::Length(1), // controls 2
        ])
        .split(inner);

    // --- dB scale labels ---
    let top_label = Paragraph::new(" +12dB").style(Style::default().fg(Color::DarkGray));
    f.render_widget(top_label, chunks[0]);

    let bot_label = Paragraph::new(" -12dB").style(Style::default().fg(Color::DarkGray));
    f.render_widget(bot_label, chunks[2]);

    // --- EQ Bars ---
    let bar_constraints: Vec<Constraint> = (0..NUM_BANDS).map(|_| Constraint::Ratio(1, NUM_BANDS as u32)).collect();

    let bar_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(bar_constraints.clone())
        .split(chunks[1]);

    let bar_height = chunks[1].height as i32;
    for (i, col) in bar_cols.iter().enumerate() {
        render_band_bar(f, *col, eq.gains[i], bar_height, i == eq.selected_band);
    }

    // --- Band name labels ---
    let label_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(bar_constraints.clone())
        .split(chunks[3]);

    for (i, col) in label_cols.iter().enumerate() {
        let style = if i == eq.selected_band {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let label = Paragraph::new(BAND_LABELS[i]).style(style).alignment(Alignment::Center);
        f.render_widget(label, *col);
    }

    // --- Frequency labels ---
    let freq_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(bar_constraints.clone())
        .split(chunks[4]);

    for (i, col) in freq_cols.iter().enumerate() {
        let freq = BAND_FREQS[i];
        let label = if freq >= 1000 {
            format!("{}k", freq / 1000)
        } else {
            format!("{}", freq)
        };
        let style = if i == eq.selected_band {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let widget = Paragraph::new(label).style(style).alignment(Alignment::Center);
        f.render_widget(widget, *col);
    }

    // --- Gain values ---
    let val_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(bar_constraints)
        .split(chunks[5]);

    for (i, col) in val_cols.iter().enumerate() {
        let val = format!("{:+.0}", eq.gains[i]);
        let style = if i == eq.selected_band {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let widget = Paragraph::new(val).style(style).alignment(Alignment::Center);
        f.render_widget(widget, *col);
    }

    // --- Controls ---
    let help1 = Paragraph::new("  ←→: Chon band   ↑↓: +/-1dB   p/P: Presets   t: On/Off")
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(help1, chunks[7]);

    let help2 = Paragraph::new("  e/Esc: Dong EQ   Space: Play/Pause")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help2, chunks[8]);
}

fn render_band_bar(f: &mut Frame, area: Rect, gain: f64, height: i32, selected: bool) {
    if area.width < 2 || height < 2 {
        return;
    }

    let mid = height / 2;
    let bar_w = area.width.min(4);
    let x_start = area.x + (area.width - bar_w) / 2;

    for y_off in 0..height {
        let y = area.y + y_off as u16;
        // dB value at this row: top = +12, middle = 0, bottom = -12
        let db_at_y = (mid - y_off) as f64 * 12.0 / mid.max(1) as f64;

        // Center line
        if y_off == mid {
            let line: String = "─".repeat(bar_w as usize);
            let widget = Paragraph::new(line).style(Style::default().fg(Color::DarkGray));
            f.render_widget(widget, Rect::new(x_start, y, bar_w, 1));
            continue;
        }

        let show = if gain >= 0.0 {
            db_at_y > 0.0 && db_at_y <= gain
        } else {
            db_at_y < 0.0 && db_at_y >= gain
        };

        if show {
            let color = if selected {
                band_color_bright(gain)
            } else {
                band_color(gain)
            };

            let ch = if selected { '█' } else { '▓' };
            let bar: String = std::iter::repeat(ch).take(bar_w as usize).collect();
            let mut style = Style::default().fg(color);
            if selected {
                style = style.add_modifier(Modifier::BOLD);
            }
            let widget = Paragraph::new(bar).style(style);
            f.render_widget(widget, Rect::new(x_start, y, bar_w, 1));
        }
    }

    // Selection indicator at bottom
    if selected {
        let indicator = "▲";
        let ix = x_start + bar_w / 2;
        let iy = area.y + area.height;
        if iy < area.y + area.height + 2 {
            let widget = Paragraph::new(indicator).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
            f.render_widget(widget, Rect::new(ix, iy.saturating_sub(1), 1, 1));
        }
    }
}

fn band_color(gain: f64) -> Color {
    let abs = gain.abs();
    if abs > 8.0 {
        Color::Red
    } else if abs > 4.0 {
        Color::Yellow
    } else if gain < 0.0 {
        Color::Blue
    } else {
        Color::Green
    }
}

fn band_color_bright(gain: f64) -> Color {
    let abs = gain.abs();
    if abs > 8.0 {
        Color::LightRed
    } else if abs > 4.0 {
        Color::LightYellow
    } else if gain < 0.0 {
        Color::LightBlue
    } else {
        Color::LightGreen
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
