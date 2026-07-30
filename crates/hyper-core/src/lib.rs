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

pub async fn discover_wildcard_urls(pattern: &str, padding: usize, maximum: usize) -> Result<Vec<String>> {
    use futures_util::{StreamExt, stream};
    if pattern.matches('*').count() != 1 {
        return Err(Error::Task("pattern must contain exactly one wildcard".into()));
    }
    let maximum = maximum.clamp(1, 10_000);
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .timeout(std::time::Duration::from_secs(12))
        .build()?;
    let mut found = Vec::new();
    let mut seen_any = false;
    let mut empty_windows = 0_u8;
    for window_start in (0..maximum).step_by(32) {
        let window_end = (window_start + 32).min(maximum);
        let checks = (window_start..window_end).map(|number| {
            let url = pattern.replacen('*', &format!("{number:0padding$}"), 1);
            let client = client.clone();
            async move {
                let head = client.head(&url).send().await;
                let exists = match head {
                    Ok(response) if response.status().is_success() => true,
                    Ok(response) if response.status() == reqwest::StatusCode::METHOD_NOT_ALLOWED => client
                        .get(&url)
                        .header(reqwest::header::RANGE, "bytes=0-0")
                        .send()
                        .await
                        .is_ok_and(|r| r.status().is_success()),
                    _ => false,
                };
                (number, url, exists)
            }
        });
        let mut results = stream::iter(checks)
            .buffer_unordered(12)
            .collect::<Vec<_>>()
            .await;
        results.sort_by_key(|item| item.0);
        let window_found = results
            .into_iter()
            .filter_map(|(_, url, exists)| exists.then_some(url))
            .collect::<Vec<_>>();
        if window_found.is_empty() && seen_any {
            break;
        }
        if window_found.is_empty() {
            empty_windows += 1;
            if empty_windows >= 4 {
                break;
            }
        } else {
            empty_windows = 0;
        }
        seen_any |= !window_found.is_empty();
        found.extend(window_found);
    }
    if found.is_empty() {
        return Err(Error::Task(
            "no downloadable files were found for this pattern".into(),
        ));
    }
    Ok(found)
}
