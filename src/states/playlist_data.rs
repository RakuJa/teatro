use bon::bon;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct PlaylistData {
    pub tracks: Vec<Track>,
    pub current_track: u64,
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
    pub track_length: Duration,
    pub elapsed_time: Duration,
}

#[bon]
impl Track {
    #[builder]
    pub fn new(
        file_path: &str,
        track_length: Option<Duration>,
        current_position: Option<Duration>,
    ) -> Self {
        Self {
            file_path: file_path.to_string(),
            track_length: track_length.unwrap_or_default(),
            elapsed_time: current_position.unwrap_or_default(),
        }
    }
}
