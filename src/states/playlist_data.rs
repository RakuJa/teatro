use bon::bon;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct PlaylistData {
    pub tracks: Vec<Track>,
    pub current_track: u64,
}

impl PlaylistData {
    pub fn get_current_track(&self) -> Option<Track> {
        self.tracks.get(self.current_track as usize).cloned()
    }
}

#[bon]
impl PlaylistData {
    #[builder]
    pub fn new(tracks: Vec<Track>, current_track: Option<u64>) -> Self {
        Self {
            tracks,
            current_track: current_track.unwrap_or(0),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Track {
    pub file_path: String,
    pub track_length: u64,
    pub elapsed_seconds: u64,
}

#[bon]
impl Track {
    #[builder]
    pub fn new(
        file_path: &str,
        track_length: Option<Option<Duration>>,
        current_position: Option<u64>,
    ) -> Self {
        Self {
            file_path: file_path.to_string(),
            track_length: track_length.unwrap_or_default().map_or(0, |x| x.as_secs()),
            elapsed_seconds: current_position.unwrap_or(0),
        }
    }
}
