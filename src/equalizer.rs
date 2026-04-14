use serde_json::json;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::time::Duration;

const IPC_SOCKET: &str = "/tmp/mssic-mpv.sock";

pub const NUM_BANDS: usize = 10;

pub const BAND_LABELS: [&str; NUM_BANDS] = [
    "Sub", "Bass", "Low", "LMid", "Mid", "UMid", "High", "HiHi", "Air", "Bril",
];

pub const BAND_FREQS: [u32; NUM_BANDS] = [
    31, 62, 125, 250, 500, 1000, 2000, 4000, 8000, 16000,
];

pub const BAND_WIDTHS: [u32; NUM_BANDS] = [
    20, 40, 80, 160, 300, 600, 1200, 2400, 4800, 4000,
];

#[derive(Clone)]
pub struct Preset {
    pub name: &'static str,
    pub gains: [f64; NUM_BANDS],
}

pub const PRESETS: [Preset; 8] = [
    Preset {
        name: "Flat",
        gains: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    Preset {
        name: "Bass Boost",
        gains: [8.0, 7.0, 5.0, 3.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0],
    },
    Preset {
        name: "Treble Boost",
        gains: [0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 4.0, 6.0, 7.0, 7.0],
    },
    Preset {
        name: "Vocal",
        gains: [-3.0, -2.0, 0.0, 2.0, 5.0, 5.0, 3.0, 1.0, 0.0, -1.0],
    },
    Preset {
        name: "Electronic",
        gains: [7.0, 6.0, 4.0, 0.0, -2.0, 0.0, 2.0, 5.0, 6.0, 5.0],
    },
    Preset {
        name: "Deep Bass",
        gains: [10.0, 8.0, 6.0, 3.0, 0.0, -1.0, -2.0, -1.0, 0.0, 0.0],
    },
    Preset {
        name: "Rock",
        gains: [5.0, 4.0, 2.0, 0.0, -1.0, 1.0, 3.0, 5.0, 6.0, 5.0],
    },
    Preset {
        name: "Jazz",
        gains: [3.0, 2.0, 0.0, 2.0, 4.0, 4.0, 2.0, 1.0, 3.0, 4.0],
    },
];

pub struct Equalizer {
    pub gains: [f64; NUM_BANDS],
    pub selected_band: usize,
    pub active_preset: usize,
    pub enabled: bool,
    pub show_panel: bool,
}

impl Equalizer {
    pub fn new() -> Self {
        Self {
            gains: [0.0; NUM_BANDS],
            selected_band: 0,
            active_preset: 0,
            enabled: true,
            show_panel: false,
        }
    }

    pub fn toggle_panel(&mut self) {
        self.show_panel = !self.show_panel;
    }

    pub fn select_next_band(&mut self) {
        self.selected_band = (self.selected_band + 1) % NUM_BANDS;
    }

    pub fn select_prev_band(&mut self) {
        self.selected_band = if self.selected_band == 0 {
            NUM_BANDS - 1
        } else {
            self.selected_band - 1
        };
    }

    pub fn increase_band(&mut self) {
        self.gains[self.selected_band] = (self.gains[self.selected_band] + 1.0).min(12.0);
        self.active_preset = usize::MAX;
        self.apply();
    }

    pub fn decrease_band(&mut self) {
        self.gains[self.selected_band] = (self.gains[self.selected_band] - 1.0).max(-12.0);
        self.active_preset = usize::MAX;
        self.apply();
    }

    pub fn next_preset(&mut self) {
        self.active_preset = (self.active_preset.wrapping_add(1)) % PRESETS.len();
        self.gains = PRESETS[self.active_preset].gains;
        self.apply();
    }

    pub fn prev_preset(&mut self) {
        if self.active_preset == 0 || self.active_preset == usize::MAX {
            self.active_preset = PRESETS.len() - 1;
        } else {
            self.active_preset -= 1;
        }
        self.gains = PRESETS[self.active_preset].gains;
        self.apply();
    }

    pub fn toggle_enabled(&mut self) {
        self.enabled = !self.enabled;
        if self.enabled {
            self.apply();
        } else {
            self.clear();
        }
    }

    pub fn preset_name(&self) -> &str {
        if self.active_preset < PRESETS.len() {
            PRESETS[self.active_preset].name
        } else {
            "Custom"
        }
    }

    pub fn apply(&self) {
        if !self.enabled {
            return;
        }

        let filters: Vec<String> = (0..NUM_BANDS)
            .map(|i| {
                format!(
                    "equalizer=f={}:t=h:w={}:g={}",
                    BAND_FREQS[i], BAND_WIDTHS[i], self.gains[i]
                )
            })
            .collect();

        let filter_str = format!("lavfi=[{}]", filters.join(","));
        let cmd = json!({"command": ["af", "set", filter_str]});
        self.send_command(&cmd);
    }

    fn clear(&self) {
        let cmd = json!({"command": ["af", "set", ""]});
        self.send_command(&cmd);
    }

    fn send_command(&self, cmd: &serde_json::Value) {
        if let Ok(mut stream) = UnixStream::connect(IPC_SOCKET) {
            let _ = stream.set_write_timeout(Some(Duration::from_millis(100)));
            let mut msg = cmd.to_string();
            msg.push('\n');
            let _ = stream.write_all(msg.as_bytes());
        }
    }
}
