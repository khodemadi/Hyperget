use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

pub type DownloadId = Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadState {
    Created,
    Resolving,
    Queued,
    Connecting,
    Downloading,
    Pausing,
    Paused,
    RetryWaiting,
    Merging,
    Verifying,
    Completed,
    Failed,
    Cancelled,
}

impl DownloadState {
    pub fn can_transition(self, to: Self) -> bool {
        use DownloadState::*;
        matches!(
            (self, to),
            (Created, Resolving | Queued | Cancelled)
                | (Resolving, Queued | Connecting | Failed | Cancelled)
                | (Queued, Connecting | Paused | Cancelled)
                | (
                    Connecting,
                    Downloading | RetryWaiting | Failed | Pausing | Cancelled
                )
                | (
                    Downloading,
                    Pausing | RetryWaiting | Merging | Verifying | Completed | Failed | Cancelled
                )
                | (Pausing, Paused | Failed | Cancelled)
                | (Paused, Queued | Connecting | Cancelled)
                | (RetryWaiting, Connecting | Pausing | Failed | Cancelled)
                | (Merging, Verifying | Completed | Failed | Cancelled)
                | (Verifying, Completed | Failed | Cancelled)
                | (Failed, Queued | Connecting | Cancelled)
                | (Cancelled, Queued)
        )
    }
    pub fn is_active(self) -> bool {
        matches!(
            self,
            Self::Resolving
                | Self::Connecting
                | Self::Downloading
                | Self::Pausing
                | Self::RetryWaiting
                | Self::Merging
                | Self::Verifying
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddDownloadRequest {
    pub url: String,
    pub output: Option<PathBuf>,
    pub connections: u8,
    pub start_immediately: bool,
    pub checksum_sha256: Option<String>,
}
impl Default for AddDownloadRequest {
    fn default() -> Self {
        Self {
            url: String::new(),
            output: None,
            connections: 4,
            start_immediately: false,
            checksum_sha256: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadSnapshot {
    pub id: DownloadId,
    pub url: String,
    pub final_url: Option<String>,
    pub filename: String,
    pub destination: PathBuf,
    pub temporary_directory: PathBuf,
    pub state: DownloadState,
    pub queue_position: i64,
    pub total_bytes: Option<u64>,
    pub downloaded_bytes: u64,
    pub connection_count: u8,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub checksum_sha256: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub error: Option<String>,
}
impl DownloadSnapshot {
    pub fn percentage(&self) -> Option<f64> {
        self.total_bytes
            .filter(|n| *n > 0)
            .map(|n| self.downloaded_bytes as f64 * 100.0 / n as f64)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DownloadFilter {
    pub state: Option<DownloadState>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalStatus {
    pub downloaded_bytes: u64,
    pub known_total_bytes: u64,
    pub percentage: Option<f64>,
    pub combined_speed: u64,
    pub active: u32,
    pub queued: u32,
    pub paused: u32,
    pub completed: u32,
    pub failed: u32,
}
