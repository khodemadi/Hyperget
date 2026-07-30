use crate::{DownloadId, DownloadState};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CoreEvent {
    DownloadAdded {
        id: DownloadId,
        timestamp: String,
        sequence: u64,
    },
    StateChanged {
        id: DownloadId,
        state: DownloadState,
        timestamp: String,
        sequence: u64,
    },
    Progress {
        id: DownloadId,
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
        speed: u64,
        eta_seconds: Option<u64>,
        timestamp: String,
        sequence: u64,
    },
    SegmentUpdated {
        id: DownloadId,
        index: u32,
        downloaded_bytes: u64,
        timestamp: String,
        sequence: u64,
    },
    Completed {
        id: DownloadId,
        timestamp: String,
        sequence: u64,
    },
    Failed {
        id: DownloadId,
        message: String,
        timestamp: String,
        sequence: u64,
    },
    Removed {
        id: DownloadId,
        timestamp: String,
        sequence: u64,
    },
}
