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

/// Errors raised while updating the shared filter.
#[derive(Debug, Error)]
pub enum FilterError {
    #[error("filter data lock is poisoned")]
    DataPoisoned,

    #[error("biquad filter lock is poisoned")]
    FilterPoisoned,

    #[error("invalid filter coefficients: {0:?}")]
    Coefficients(biquad::Errors),
}
