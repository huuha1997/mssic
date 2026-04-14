use anyhow::{Context, Result};
use serde_json::json;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use crate::types::Track;

const IPC_SOCKET: &str = "/tmp/mssic-mpv.sock";

pub struct Player {
    process: Option<Child>,
    pub current_track: Option<Track>,
    pub volume: f64,
    pub is_loading: bool,
    pub time_pos: f64,
    pub duration: f64,
    pub paused: bool,
}

impl Player {
    pub fn new() -> Result<Self> {
        let _ = std::fs::remove_file(IPC_SOCKET);

        Ok(Self {
            process: None,
            current_track: None,
            volume: 80.0,
            is_loading: false,
            time_pos: 0.0,
            duration: 0.0,
            paused: false,
        })
    }

    pub fn play_url(&mut self, url: &str, track: Track) -> Result<()> {
        self.kill_process();
        let _ = std::fs::remove_file(IPC_SOCKET);

        let child = Command::new("mpv")
            .arg("--no-video")
            .arg("--no-terminal")
            .arg(format!("--input-ipc-server={}", IPC_SOCKET))
            .arg(format!("--volume={}", self.volume as u32))
            .arg("--demuxer-max-bytes=50MiB")
            .arg("--demuxer-max-back-bytes=25MiB")
            .arg(url)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("mpv not found. Install: brew install mpv")?;

        self.process = Some(child);
        self.current_track = Some(track);
        self.is_loading = false;
        self.time_pos = 0.0;
        self.duration = 0.0;
        self.paused = false;

        Ok(())
    }

    pub fn toggle_pause(&mut self) {
        if self.current_track.is_none() {
            return;
        }
        self.send_command(&json!({"command": ["cycle", "pause"]}));
        self.paused = !self.paused;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn set_volume(&mut self, vol: f64) {
        self.volume = vol.clamp(0.0, 100.0);
        self.send_command(&json!({"command": ["set_property", "volume", self.volume]}));
    }

    pub fn volume_up(&mut self) {
        self.set_volume(self.volume + 5.0);
    }

    pub fn volume_down(&mut self) {
        self.set_volume(self.volume - 5.0);
    }

    pub fn seek(&mut self, secs: f64) {
        self.send_command(&json!({"command": ["seek", secs, "relative"]}));
    }

    pub fn elapsed_secs(&self) -> f64 {
        self.time_pos
    }

    pub fn is_finished(&self) -> bool {
        if self.is_loading {
            return false;
        }
        match &self.process {
            Some(_) => false,
            None => self.current_track.is_some(),
        }
    }

    /// Poll mpv state - called every ~500ms only when playing
    pub fn tick(&mut self) {
        if let Some(ref mut child) = self.process {
            match child.try_wait() {
                Ok(Some(_)) => {
                    self.process = None;
                    return;
                }
                _ => {}
            }
        }

        if self.process.is_none() || self.current_track.is_none() {
            return;
        }

        // Batch read: single socket connection for both properties
        if let Ok(mut stream) = UnixStream::connect(IPC_SOCKET) {
            let _ = stream.set_read_timeout(Some(Duration::from_millis(30)));
            let _ = stream.set_write_timeout(Some(Duration::from_millis(30)));

            // Send both queries
            let cmd1 = json!({"command": ["get_property", "time-pos"], "request_id": 1});
            let cmd2 = json!({"command": ["get_property", "duration"], "request_id": 2});
            let msg = format!("{}\n{}\n", cmd1, cmd2);

            if stream.write_all(msg.as_bytes()).is_ok() {
                let reader = BufReader::new(&stream);
                for line in reader.lines().take(4) {
                    // read up to 4 lines (responses + events)
                    let Ok(line) = line else { break };
                    let Ok(val) = serde_json::from_str::<serde_json::Value>(&line) else {
                        continue;
                    };
                    if val.get("error").and_then(|e| e.as_str()) != Some("success") {
                        continue;
                    }
                    let data = val.get("data").and_then(|d| d.as_f64());
                    match val.get("request_id").and_then(|r| r.as_u64()) {
                        Some(1) => {
                            if let Some(pos) = data {
                                self.time_pos = pos;
                            }
                        }
                        Some(2) => {
                            if let Some(dur) = data {
                                self.duration = dur;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn stop(&mut self) {
        self.kill_process();
        self.current_track = None;
        self.time_pos = 0.0;
        self.duration = 0.0;
    }

    fn send_command(&self, cmd: &serde_json::Value) {
        if let Ok(mut stream) = UnixStream::connect(IPC_SOCKET) {
            let _ = stream.set_write_timeout(Some(Duration::from_millis(50)));
            let mut msg = cmd.to_string();
            msg.push('\n');
            let _ = stream.write_all(msg.as_bytes());
        }
    }

    fn kill_process(&mut self) {
        self.send_command(&json!({"command": ["quit"]}));
        std::thread::sleep(Duration::from_millis(50));

        if let Some(ref mut child) = self.process {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.process = None;
        let _ = std::fs::remove_file(IPC_SOCKET);
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.kill_process();
    }
}
