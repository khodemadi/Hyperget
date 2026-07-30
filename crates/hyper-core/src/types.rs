use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

pub type DownloadId = Uuid;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    #[default]
    Normal,
    High,
    Critical,
}

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
    pub destination_directory: Option<PathBuf>,
    pub connections: u8,
    pub start_immediately: bool,
    pub checksum_sha256: Option<String>,
}
impl Default for AddDownloadRequest {
    fn default() -> Self {
        Self {
            url: String::new(),
            output: None,
            destination_directory: None,
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
    pub priority: Priority,
    pub start_immediately: bool,
    pub per_download_speed_limit: Option<u64>,
    pub total_bytes: Option<u64>,
    pub downloaded_bytes: u64,
    pub connection_count: u8,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub checksum_sha256: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
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
    pub total: u32,
    pub downloaded_bytes: u64,
    pub known_total_bytes: u64,
    pub percentage: Option<f64>,
    pub combined_speed: u64,
    pub active: u32,
    pub queued: u32,
    pub paused: u32,
    pub completed: u32,
    pub failed: u32,
    pub unknown_size: u32,
    pub active_connections: u32,
    pub eta_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub maximum_simultaneous_downloads: u8,
    pub default_connections_per_file: u8,
    pub default_retry_count: u8,
    pub initial_retry_delay_seconds: u64,
    pub maximum_retry_delay_seconds: u64,
    pub auto_retry: bool,
    pub auto_start_next: bool,
    pub restore_unfinished_downloads: bool,
    pub global_speed_limit_mode: String,
    pub global_speed_limit_bytes: u64,
    pub default_priority: Priority,
    pub confirm_before_delete: bool,
    pub theme: String,
    pub default_download_directory: String,
    pub ask_where_to_save: bool,
    pub remember_last_directory: bool,
    pub last_selected_directory: String,
    pub create_category_subfolders: bool,
    pub wildcard_batch_behavior: String,
    pub wildcard_auto_start: bool,
    pub quick_download_bar_expanded: bool,
    pub duplicate_filename_behavior: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            maximum_simultaneous_downloads: 3,
            default_connections_per_file: 8,
            default_retry_count: 5,
            initial_retry_delay_seconds: 1,
            maximum_retry_delay_seconds: 30,
            auto_retry: true,
            auto_start_next: true,
            restore_unfinished_downloads: true,
            global_speed_limit_mode: "unlimited".into(),
            global_speed_limit_bytes: 0,
            default_priority: Priority::Normal,
            confirm_before_delete: true,
            theme: "system".into(),
            default_download_directory: String::new(),
            ask_where_to_save: false,
            remember_last_directory: true,
            last_selected_directory: String::new(),
            create_category_subfolders: false,
            wildcard_batch_behavior: "preview".into(),
            wildcard_auto_start: false,
            quick_download_bar_expanded: true,
            duplicate_filename_behavior: "rename".into(),
        }
    }
}
