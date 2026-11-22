use rodio::decoder::DecoderError;
use thiserror::Error;

/// Errors raised while loading a track into a `Player`.
#[derive(Debug, Error)]
pub enum PlaybackError {
    #[error("failed to open `{path}`: {source}")]
    Open {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to decode `{path}`: {source}")]
    Decode {
        path: String,
        #[source]
        source: DecoderError,
    },

    #[error("track index {index} is out of range (playlist has {len} tracks)")]
    TrackOutOfRange { index: usize, len: usize },
    #[error("Seek failed: {0}")]
    Seek(#[source] rodio::source::SeekError),
}
