use crate::{DownloadId, DownloadState};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("download {0} was not found")]
    NotFound(DownloadId),
    #[error("invalid transition from {from:?} to {to:?}")]
    InvalidTransition { from: DownloadState, to: DownloadState },
    #[error("invalid URL: {0}")]
    InvalidUrl(String),
    #[error("remote content changed; refusing unsafe resume")]
    RemoteChanged,
    #[error("server returned an invalid byte range: {0}")]
    InvalidRange(String),
    #[error("database: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("network: {0}")]
    Network(#[from] reqwest::Error),
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("verification failed: {0}")]
    Verification(String),
    #[error("task already active")]
    AlreadyActive,
    #[error("internal task failed: {0}")]
    Task(String),
}
pub type Result<T> = std::result::Result<T, Error>;
