use anyhow::Result;
use ratatui::widgets::ListState;
use tokio::sync::mpsc;

use crate::equalizer::Equalizer;
use crate::library;
use crate::player::Player;
use crate::queue::PlayQueue;
use crate::types::{AppAction, AppEvent, InputMode, Tab, Track};
use crate::ytdlp;

pub struct App {
    pub search_query: String,
    pub search_results: Vec<Track>,
    pub library: Vec<Track>,
    pub queue: PlayQueue,
    pub list_state: ListState,
    pub current_tab: Tab,
    pub input_mode: InputMode,
    pub player: Player,
    pub eq: Equalizer,
    pub message: String,
    pub show_help: bool,
    event_tx: mpsc::UnboundedSender<AppEvent>,
}

impl App {
    pub fn new(event_tx: mpsc::UnboundedSender<AppEvent>) -> Result<Self> {
        let player = Player::new()?;
        let lib = library::load();

        Ok(Self {
            search_query: String::new(),
            search_results: Vec::new(),
            library: lib,
            queue: PlayQueue::new(),
            list_state: ListState::default(),
            current_tab: Tab::Search,
            input_mode: InputMode::Normal,
            player,
            eq: Equalizer::new(),
            message: "Nhan 'i' de tim nhac, 'e' mo Equalizer, '?' de xem Help".to_string(),
            show_help: false,
            event_tx,
        })
    }

    pub fn current_list(&self) -> Vec<&Track> {
        match self.current_tab {
            Tab::Search => self.search_results.iter().collect(),
            Tab::Library => self.library.iter().collect(),
            Tab::Queue => self.queue.tracks.iter().collect(),
        }
    }

    pub fn handle_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit => {} // handled in main loop
            AppAction::EnterEdit => {
                self.input_mode = InputMode::Editing;
                self.message = "Dang nhap... (Enter: tim, Esc: huy)".to_string();
            }
            AppAction::ExitEdit => {
                self.input_mode = InputMode::Normal;
                self.message = "Da thoat che do nhap.".to_string();
            }
            AppAction::Search => self.spawn_search(),
            AppAction::PlaySelected => self.play_selected(),
            AppAction::TogglePause => {
                if self.player.current_track.is_some() {
                    self.player.toggle_pause();
                    self.message = if self.player.is_paused() {
                        "Paused.".to_string()
                    } else {
                        "Playing...".to_string()
                    };
                } else if self.player.is_loading {
                    self.message = "Dang tai nhac, vui long cho...".to_string();
                } else {
                    self.message = "Chua co bai nao. An Enter de phat.".to_string();
                }
            }
            AppAction::NextTab => {
                self.current_tab = match self.current_tab {
                    Tab::Search => Tab::Library,
                    Tab::Library => Tab::Queue,
                    Tab::Queue => Tab::Search,
                };
                self.list_state.select(Some(0));
            }
            AppAction::NextItem => self.navigate(1),
            AppAction::PrevItem => self.navigate(-1),
            AppAction::VolumeUp => {
                self.player.volume_up();
                self.message = format!("Volume: {}%", self.player.volume as u32);
            }
            AppAction::VolumeDown => {
                self.player.volume_down();
                self.message = format!("Volume: {}%", self.player.volume as u32);
            }
            AppAction::SeekForward => {
                self.player.seek(10.0);
                self.message = ">> +10s".to_string();
            }
            AppAction::SeekBackward => {
                self.player.seek(-10.0);
                self.message = "<< -10s".to_string();
            }
            AppAction::Stop => {
                self.player.stop();
                self.message = "Stopped.".to_string();
            }
            AppAction::ToggleFavorite => self.toggle_favorite(),
            AppAction::AddToQueue => self.add_to_queue(),
            AppAction::ShowHelp => {
                self.show_help = !self.show_help;
            }
            AppAction::ToggleEQ => {
                self.eq.toggle_panel();
            }
            AppAction::EqNextBand => self.eq.select_next_band(),
            AppAction::EqPrevBand => self.eq.select_prev_band(),
            AppAction::EqBandUp => {
                self.eq.increase_band();
                self.message = format!(
                    "EQ {} ({}Hz): {:+.0}dB",
                    crate::equalizer::BAND_LABELS[self.eq.selected_band],
                    crate::equalizer::BAND_FREQS[self.eq.selected_band],
                    self.eq.gains[self.eq.selected_band]
                );
            }
            AppAction::EqBandDown => {
                self.eq.decrease_band();
                self.message = format!(
                    "EQ {} ({}Hz): {:+.0}dB",
                    crate::equalizer::BAND_LABELS[self.eq.selected_band],
                    crate::equalizer::BAND_FREQS[self.eq.selected_band],
                    self.eq.gains[self.eq.selected_band]
                );
            }
            AppAction::EqNextPreset => {
                self.eq.next_preset();
                self.message = format!("EQ Preset: {}", self.eq.preset_name());
            }
            AppAction::EqPrevPreset => {
                self.eq.prev_preset();
                self.message = format!("EQ Preset: {}", self.eq.preset_name());
            }
            AppAction::EqToggleEnabled => {
                self.eq.toggle_enabled();
                self.message = format!(
                    "EQ: {}",
                    if self.eq.enabled { "ON" } else { "OFF" }
                );
            }
            AppAction::TypeChar(c) => {
                self.search_query.push(c);
            }
            AppAction::Backspace => {
                self.search_query.pop();
            }
        }
    }

    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::SearchResults(tracks) => {
                let count = tracks.len();
                self.search_results = tracks;
                self.input_mode = InputMode::Normal;
                if count > 0 {
                    self.list_state.select(Some(0));
                    self.message = format!("Tim thay {} bai hat.", count);
                } else {
                    self.message = "Khong thay ket qua.".to_string();
                }
            }
            AppEvent::StreamUrl { track, url } => {
                let title = track.title.clone();
                if let Err(e) = self.player.play_url(&url, track) {
                    self.message = format!("Loi phat nhac: {}", e);
                } else {
                    self.message = format!("Playing: {}", title);
                }
            }
            AppEvent::Error(msg) => {
                self.player.is_loading = false;
                self.message = format!("Loi: {}", msg);
            }
        }
    }

    pub fn check_auto_next(&mut self) {
        if self.player.is_finished() && !self.queue.is_empty() {
            if let Some(next_track) = self.queue.pop_next() {
                self.start_play(next_track);
            }
        }
    }

    pub fn save_library(&self) {
        let _ = library::save(&self.library);
    }

    fn spawn_search(&mut self) {
        if self.search_query.is_empty() {
            return;
        }
        self.message = format!("Dang tim '{}'...", self.search_query);
        let query = self.search_query.clone();
        let tx = self.event_tx.clone();

        tokio::spawn(async move {
            match ytdlp::search(&query).await {
                Ok(tracks) => {
                    let _ = tx.send(AppEvent::SearchResults(tracks));
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::Error(e.to_string()));
                }
            }
        });
    }

    fn play_selected(&mut self) {
        let list = self.current_list();
        if let Some(index) = self.list_state.selected() {
            if let Some(track) = list.get(index) {
                self.start_play((*track).clone());
            }
        }
    }

    fn start_play(&mut self, track: Track) {
        self.player.is_loading = true;
        self.message = format!("Dang lay link: {}...", track.title);
        let tx = self.event_tx.clone();
        let id = track.id.clone();
        let track_clone = track.clone();

        tokio::spawn(async move {
            match ytdlp::get_stream_url(&id).await {
                Ok(url) => {
                    let _ = tx.send(AppEvent::StreamUrl {
                        track: track_clone,
                        url,
                    });
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::Error(e.to_string()));
                }
            }
        });
    }

    fn toggle_favorite(&mut self) {
        if let Some(index) = self.list_state.selected() {
            match self.current_tab {
                Tab::Search => {
                    if let Some(track) = self.search_results.get(index) {
                        if !self.library.iter().any(|t| t.id == track.id) {
                            let title = track.title.clone();
                            self.library.push(track.clone());
                            self.save_library();
                            self.message = format!("Saved: {}", title);
                        } else {
                            self.message = "Da co trong Library.".to_string();
                        }
                    }
                }
                Tab::Library => {
                    if index < self.library.len() {
                        self.library.remove(index);
                        self.save_library();
                        self.message = "Da xoa khoi Library.".to_string();
                        // Adjust selection
                        if !self.library.is_empty() {
                            let new_idx = index.min(self.library.len() - 1);
                            self.list_state.select(Some(new_idx));
                        } else {
                            self.list_state.select(None);
                        }
                    }
                }
                Tab::Queue => {}
            }
        }
    }

    fn add_to_queue(&mut self) {
        let list = self.current_list();
        if let Some(index) = self.list_state.selected() {
            if let Some(track) = list.get(index) {
                let title = track.title.clone();
                self.queue.push((*track).clone());
                self.message = format!("Queue +1: {} ({})", title, self.queue.len());
            }
        }
    }

    fn navigate(&mut self, direction: i32) {
        let len = self.current_list().len();
        if len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if direction > 0 {
                    if i >= len - 1 { 0 } else { i + 1 }
                } else if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}
