use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" DJ Control ")
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height < 4 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // track title
            Constraint::Length(1), // progress bar
            Constraint::Length(1), // spacer
            Constraint::Length(1), // volume + controls
            Constraint::Length(1), // EQ visualizer
        ])
        .split(inner);

    match &app.player.current_track {
        Some(track) => {
            // --- Line 1: Track title + status ---
            let status_icon = if app.player.is_loading {
                "⏳"
            } else if app.player.is_paused() {
                "⏸ "
            } else {
                "▶ "
            };

            let title = format!("  {}  {}", status_icon, track.title);
            let title_widget = Paragraph::new(title).style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );
            f.render_widget(title_widget, chunks[0]);

            // --- Line 2: Progress bar ---
            let elapsed = app.player.elapsed_secs();
            let total = if app.player.duration > 0.0 {
                app.player.duration
            } else {
                track.duration_secs.unwrap_or(0.0)
            };
            let ratio = if total > 0.0 {
                (elapsed / total).min(1.0)
            } else {
                0.0
            };

            let elapsed_fmt = format_time(elapsed);
            let total_fmt = format_time(total);

            let gauge = Gauge::default()
                .gauge_style(Style::default().fg(Color::Green).bg(Color::DarkGray))
                .ratio(ratio)
                .label(format!("  {} / {}  ", elapsed_fmt, total_fmt));
            f.render_widget(gauge, chunks[1]);

            // --- Line 3: Volume fader + transport controls ---
            if chunks.len() > 3 {
                let ctrl_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                    .split(chunks[3]);

                // Volume fader
                let vol = app.player.volume;
                let vol_bars = (vol / 5.0) as usize;
                let vol_empty = 20_usize.saturating_sub(vol_bars);
                let vol_display = format!(
                    "  VOL  {}{}  {}%",
                    "█".repeat(vol_bars),
                    "░".repeat(vol_empty),
                    vol as u32
                );
                let vol_color = if vol > 80.0 {
                    Color::Red
                } else if vol > 50.0 {
                    Color::Yellow
                } else {
                    Color::Green
                };
                let vol_widget =
                    Paragraph::new(vol_display).style(Style::default().fg(vol_color));
                f.render_widget(vol_widget, ctrl_chunks[0]);

                // Transport controls
                let controls = format!(
                    "  ◀◀ (←)   {}   ▶▶ (→)   ⏹ (s)",
                    if app.player.is_paused() {
                        "▶  (Space)"
                    } else {
                        "⏸  (Space)"
                    }
                );
                let ctrl_widget =
                    Paragraph::new(controls).style(Style::default().fg(Color::Cyan));
                f.render_widget(ctrl_widget, ctrl_chunks[1]);
            }

            // --- Line 4: EQ visualizer (animated based on time) ---
            if chunks.len() > 4 && !app.player.is_paused() && !app.player.is_loading {
                render_eq(f, chunks[4], elapsed);
            }
        }
        None => {
            let msg = if app.player.is_loading {
                "  ⏳ Dang lay link nhac..."
            } else {
                "  Khong co bai nao. Chon bai va an Enter de phat nhac."
            };
            let empty = Paragraph::new(msg).style(Style::default().fg(Color::DarkGray));
            f.render_widget(empty, chunks[0]);
        }
    }
}

fn render_eq(f: &mut Frame, area: Rect, time: f64) {
    let bars = "▁▂▃▄▅▆▇█";
    let bar_chars: Vec<char> = bars.chars().collect();
    let width = area.width as usize;

    let mut eq_str = String::from("  EQ  ");
    for i in 0..width.saturating_sub(8) {
        // Pseudo-random animation based on time and position
        let phase = (time * 3.0 + i as f64 * 0.7).sin();
        let phase2 = (time * 5.0 + i as f64 * 1.3).cos();
        let combined = ((phase + phase2 + 2.0) / 4.0 * bar_chars.len() as f64) as usize;
        let idx = combined.min(bar_chars.len() - 1);
        eq_str.push(bar_chars[idx]);
    }

    let eq_widget = Paragraph::new(eq_str).style(
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::DIM),
    );
    f.render_widget(eq_widget, area);
}

fn format_time(secs: f64) -> String {
    let total_secs = secs as u64;
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    if h > 0 {
        format!("{}:{:02}:{:02}", h, m, s)
    } else {
        format!("{}:{:02}", m, s)
    }
}
