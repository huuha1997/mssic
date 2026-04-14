use std::collections::VecDeque;

use crate::types::Track;

pub struct PlayQueue {
    pub tracks: VecDeque<Track>,
}

impl PlayQueue {
    pub fn new() -> Self {
        Self {
            tracks: VecDeque::new(),
        }
    }

    pub fn push(&mut self, track: Track) {
        self.tracks.push_back(track);
    }

    pub fn pop_next(&mut self) -> Option<Track> {
        self.tracks.pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
    }
}
