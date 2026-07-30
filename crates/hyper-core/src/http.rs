use crate::{Error, Result};
use reqwest::header::{CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_RANGE, ETAG, LAST_MODIFIED, RANGE};
use url::Url;

#[derive(Debug, Clone)]
pub struct RemoteMetadata {
    pub final_url: String,
    pub _filename: String,
    pub total: Option<u64>,
    pub _ranges: bool,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

pub async fn probe(client: &reqwest::Client, raw: &str) -> Result<RemoteMetadata> {
    let url = Url::parse(raw).map_err(|e| Error::InvalidUrl(e.to_string()))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(Error::InvalidUrl("only http and https are supported".into()));
    }
    let head = client.head(url.clone()).send().await?;
    let mut final_url = head.url().clone();
    let mut headers = head.headers().clone();
    let mut total = headers
        .get(CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok());
    let ambiguous = !head.status().is_success() || total.is_none();
    let mut ranges = false;
    if ambiguous
        || !headers
            .get("accept-ranges")
            .is_some_and(|v| v.as_bytes().eq_ignore_ascii_case(b"bytes"))
    {
        let response = client.get(url).header(RANGE, "bytes=0-0").send().await?;
        final_url = response.url().clone();
        if response.status() == reqwest::StatusCode::PARTIAL_CONTENT {
            if let Some(value) = response
                .headers()
                .get(CONTENT_RANGE)
                .and_then(|v| v.to_str().ok())
            {
                total = parse_content_range(value)?;
                ranges = total.is_some();
            }
        }
        if headers.is_empty() {
            headers = response.headers().clone();
        }
    } else {
        ranges = true;
    }
    let filename = content_filename(headers.get(CONTENT_DISPOSITION).and_then(|v| v.to_str().ok()))
        .or_else(|| {
            final_url
                .path_segments()
                .and_then(|mut s| s.next_back())
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "download".into());
    Ok(RemoteMetadata {
        final_url: final_url.to_string(),
        _filename: sanitize_filename(&filename),
        total,
        _ranges: ranges,
        etag: header(&headers, ETAG),
        last_modified: header(&headers, LAST_MODIFIED),
    })
}
fn header(h: &reqwest::header::HeaderMap, n: reqwest::header::HeaderName) -> Option<String> {
    h.get(n).and_then(|v| v.to_str().ok()).map(str::to_owned)
}
fn parse_content_range(v: &str) -> Result<Option<u64>> {
    let Some((range, total)) = v.strip_prefix("bytes ").and_then(|s| s.split_once('/')) else {
        return Err(Error::InvalidRange(v.into()));
    };
    if range != "0-0" {
        return Err(Error::InvalidRange(v.into()));
    }
    Ok(if total == "*" {
        None
    } else {
        Some(total.parse().map_err(|_| Error::InvalidRange(v.into()))?)
    })
}
fn content_filename(v: Option<&str>) -> Option<String> {
    v?.split(';').find_map(|p| {
        p.trim()
            .strip_prefix("filename=")
            .map(|s| s.trim_matches('"').to_owned())
    })
}
pub fn sanitize_filename(v: &str) -> String {
    let s: String = v
        .chars()
        .map(|c| {
            if c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                '_'
            } else {
                c
            }
        })
        .collect();
    let s = s.trim_matches([' ', '.']);
    if s.is_empty() || s == ".." {
        "download".into()
    } else {
        s.chars().take(240).collect()
    }
}
