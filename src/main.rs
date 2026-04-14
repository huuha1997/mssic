mod app;
mod equalizer;
mod keys;
mod library;
mod player;
mod queue;
mod types;
mod ui;
mod ytdlp;

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use crate::app::App;
use crate::keys::map_key;
use crate::types::{AppAction, AppEvent};

#[tokio::main]
async fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (event_tx, event_rx) = mpsc::unbounded_channel::<AppEvent>();
    let app = App::new(event_tx).context("Audio init error")?;

    let res = run_app(&mut terminal, app, event_rx).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut app: App,
    mut event_rx: mpsc::UnboundedReceiver<AppEvent>,
) -> Result<()> {
    let mut last_render = Instant::now();
    let mut last_ipc = Instant::now();
    let mut dirty = true;

    loop {
        // --- 1. Input: always fast (50ms poll) ---
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if let Some(action) = map_key(&app.input_mode, key.code, app.eq.show_panel) {
                    if matches!(action, AppAction::Quit) {
                        app.save_library();
                        return Ok(());
                    }
                    app.handle_action(action);
                    dirty = true;
                }
            }
        }

        // --- 2. Background events: always drain ---
        while let Ok(ev) = event_rx.try_recv() {
            app.handle_event(ev);
            dirty = true;
        }

        // --- 3. IPC + auto-next: every 500ms when playing ---
        let is_playing = app.player.current_track.is_some();
        if is_playing && last_ipc.elapsed() >= Duration::from_millis(500) {
            app.player.tick();
            app.check_auto_next();
            last_ipc = Instant::now();
            dirty = true;
        } else if !is_playing {
            app.player.tick();
            app.check_auto_next();
        }

        // --- 4. Render: only when dirty or periodic refresh ---
        let render_interval = if is_playing && !app.player.is_paused() {
            Duration::from_millis(250) // smooth progress bar
        } else {
            Duration::from_secs(1) // idle: barely redraw
        };

        if dirty || last_render.elapsed() >= render_interval {
            terminal
                .draw(|f| ui::draw(f, &mut app))
                .context("Draw error")?;
            last_render = Instant::now();
            dirty = false;
        }
    }
}
