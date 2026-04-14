use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub title: String,
    pub duration_str: String,
    #[serde(default)]
    pub duration_secs: Option<f64>,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Tab {
    Search,
    Library,
    Queue,
}

impl Tab {
    pub fn index(self) -> usize {
        match self {
            Tab::Search => 0,
            Tab::Library => 1,
            Tab::Queue => 2,
        }
    }
}

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

/// Events sent from background tasks to the main loop
pub enum AppEvent {
    SearchResults(Vec<Track>),
    StreamUrl {
        track: Track,
        url: String,
    },
    Error(String),
}

/// Actions dispatched by keybindings
#[derive(Debug, Clone, Copy)]
pub enum AppAction {
    Quit,
    EnterEdit,
    ExitEdit,
    Search,
    PlaySelected,
    TogglePause,
    NextTab,
    NextItem,
    PrevItem,
    VolumeUp,
    VolumeDown,
    ToggleFavorite,
    AddToQueue,
    SeekForward,
    SeekBackward,
    Stop,
    ShowHelp,
    ToggleEQ,
    EqNextBand,
    EqPrevBand,
    EqBandUp,
    EqBandDown,
    EqNextPreset,
    EqPrevPreset,
    EqToggleEnabled,
    TypeChar(char),
    Backspace,
}

/// Parse duration string like "3:42" or "1:02:30" to seconds
pub fn parse_duration(s: &str) -> Option<f64> {
    let parts: Vec<&str> = s.split(':').collect();
    match parts.len() {
        2 => {
            let mins = parts[0].parse::<f64>().ok()?;
            let secs = parts[1].parse::<f64>().ok()?;
            Some(mins * 60.0 + secs)
        }
        3 => {
            let hours = parts[0].parse::<f64>().ok()?;
            let mins = parts[1].parse::<f64>().ok()?;
            let secs = parts[2].parse::<f64>().ok()?;
            Some(hours * 3600.0 + mins * 60.0 + secs)
        }
        _ => None,
    }
}
