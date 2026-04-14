use anyhow::{Context, Result};
use tokio::process::Command;

use crate::types::{parse_duration, Track};

/// Search YouTube - use --flat-playlist for speed (no extra metadata fetching)
pub async fn search(query: &str) -> Result<Vec<Track>> {
    let output = Command::new("yt-dlp")
        .arg("--flat-playlist")
        .arg("--print")
        .arg("%(id)s\t%(title)s\t%(duration_string)s")
        .arg(format!("ytsearch10:{}", query))
        .output()
        .await
        .context("yt-dlp not found. Install: brew install yt-dlp")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("yt-dlp search failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut tracks = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.splitn(3, '\t').collect();
        if parts.len() == 3 {
            let duration_str = parts[2].to_string();
            let duration_secs = parse_duration(&duration_str);
            tracks.push(Track {
                id: parts[0].to_string(),
                title: parts[1].to_string(),
                duration_str,
                duration_secs,
            });
        }
    }

    Ok(tracks)
}

/// Get direct stream URL (fast, no download)
pub async fn get_stream_url(video_id: &str) -> Result<String> {
    let output = Command::new("yt-dlp")
        .arg("-f")
        .arg("bestaudio")
        .arg("-g")
        .arg("--no-playlist")
        .arg(format!("https://www.youtube.com/watch?v={}", video_id))
        .output()
        .await
        .context("Failed to get stream URL")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("yt-dlp URL error: {}", stderr);
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if url.is_empty() {
        anyhow::bail!("Empty stream URL");
    }
    Ok(url)
}
