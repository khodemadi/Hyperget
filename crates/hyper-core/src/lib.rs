mod batch;
mod error;
mod events;
mod http;
mod persistence;
mod segment;
mod service;
mod types;
mod verify;

pub use batch::{BatchPreviewRequest, WildcardRange, expand_wildcards};
pub use error::{Error, Result};
pub use events::*;
pub use segment::{Segment, split_segments};
pub use service::{DownloadManager, DownloadService};
pub use types::*;
pub async fn probe_url(url: &str) -> Result<serde_json::Value> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()?;
    let metadata = http::probe(&client, url).await?;
    serde_json::to_value(metadata).map_err(|e| Error::Task(e.to_string()))
}
