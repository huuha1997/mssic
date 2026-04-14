use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use crate::types::Track;

fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("mssic")
}

fn library_path() -> PathBuf {
    config_dir().join("library.json")
}

pub fn load() -> Vec<Track> {
    let path = library_path();
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub fn save(tracks: &[Track]) -> Result<()> {
    let dir = config_dir();
    fs::create_dir_all(&dir)?;

    let json = serde_json::to_string_pretty(tracks)?;
    fs::write(library_path(), json)?;
    Ok(())
}
