use crate::{
    AddDownloadRequest, DownloadFilter, DownloadId, DownloadSnapshot, DownloadState, Error, GlobalStatus,
    Result, http, verify,
};
use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::header::{CONTENT_RANGE, RANGE};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    io::{AsyncSeekExt, AsyncWriteExt},
    sync::Mutex,
};
use tokio_util::sync::CancellationToken;

#[async_trait]
pub trait DownloadService {
    async fn add(&self, r: AddDownloadRequest) -> Result<DownloadId>;
    async fn start(&self, id: DownloadId) -> Result<()>;
    async fn pause(&self, id: DownloadId) -> Result<()>;
    async fn resume(&self, id: DownloadId) -> Result<()>;
    async fn cancel(&self, id: DownloadId) -> Result<()>;
    async fn remove(&self, id: DownloadId, delete_data: bool) -> Result<()>;
    async fn list(&self, f: DownloadFilter) -> Result<Vec<DownloadSnapshot>>;
    async fn get(&self, id: DownloadId) -> Result<DownloadSnapshot>;
    async fn start_all(&self) -> Result<()>;
    async fn pause_all(&self) -> Result<()>;
    async fn global_status(&self) -> Result<GlobalStatus>;
}

#[derive(Clone)]
pub struct DownloadManager {
    store: Arc<Mutex<crate::persistence::Store>>,
    client: reqwest::Client,
    tasks: Arc<Mutex<HashMap<DownloadId, CancellationToken>>>,
    default_dir: PathBuf,
}
impl DownloadManager {
    pub fn open(db: impl AsRef<Path>, default_dir: impl Into<PathBuf>) -> Result<Self> {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(10))
            .user_agent("Hyper-Get/0.1")
            .build()?;
        Ok(Self {
            store: Arc::new(Mutex::new(crate::persistence::Store::open(db.as_ref())?)),
            client,
            tasks: Default::default(),
            default_dir: default_dir.into(),
        })
    }
    async fn run(&self, id: DownloadId, token: CancellationToken) -> Result<()> {
        let result = self.run_inner(id, token).await;
        if let Err(ref e) = result {
            let _ = self.store.lock().await.fail(id, &e.to_string());
        }
        self.tasks.lock().await.remove(&id);
        result
    }
    async fn run_inner(&self, id: DownloadId, token: CancellationToken) -> Result<()> {
        self.store
            .lock()
            .await
            .transition(id, DownloadState::Connecting)?;
        let d = self.get(id).await?;
        let meta = http::probe(&self.client, &d.url).await?;
        if d.downloaded_bytes > 0
            && (d.total_bytes.zip(meta.total).is_some_and(|(a, b)| a != b)
                || d.etag
                    .as_ref()
                    .zip(meta.etag.as_ref())
                    .is_some_and(|(a, b)| a != b)
                || d.last_modified
                    .as_ref()
                    .zip(meta.last_modified.as_ref())
                    .is_some_and(|(a, b)| a != b))
        {
            return Err(Error::RemoteChanged);
        }
        self.store.lock().await.metadata(
            id,
            &meta.final_url,
            meta.total,
            meta.etag.as_deref(),
            meta.last_modified.as_deref(),
        )?;
        self.store
            .lock()
            .await
            .transition(id, DownloadState::Downloading)?;
        if let Some(parent) = d.destination.parent() {
            tokio::fs::create_dir_all(parent).await?
        }
        tokio::fs::create_dir_all(&d.temporary_directory).await?;
        let part = d.temporary_directory.join("download.part");
        let actual = tokio::fs::metadata(&part).await.map(|m| m.len()).unwrap_or(0);
        self.store.lock().await.progress(id, actual, meta.total)?;
        let mut request = self.client.get(&meta.final_url);
        if actual > 0 {
            request = request.header(RANGE, format!("bytes={actual}-"));
        }
        let response = request.send().await?;
        if actual > 0 {
            if response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
                return Err(Error::InvalidRange("resume request was not honored".into()));
            }
            let prefix = format!("bytes {actual}-");
            if !response
                .headers()
                .get(CONTENT_RANGE)
                .and_then(|v| v.to_str().ok())
                .is_some_and(|v| v.starts_with(&prefix))
            {
                return Err(Error::InvalidRange("resume Content-Range mismatch".into()));
            }
        }
        if !response.status().is_success() {
            return Err(Error::Task(format!("HTTP {}", response.status())));
        }
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&part)
            .await?;
        file.seek(std::io::SeekFrom::Start(actual)).await?;
        let mut received = actual;
        let mut stream = response.bytes_stream();
        let mut last = std::time::Instant::now();
        while let Some(chunk) = tokio::select! {_=token.cancelled()=>{file.flush().await?;self.store.lock().await.progress(id,received,meta.total)?;return Ok(())},v=stream.next()=>v}
        {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            received += chunk.len() as u64;
            if last.elapsed() >= std::time::Duration::from_millis(250) {
                self.store.lock().await.progress(id, received, meta.total)?;
                last = std::time::Instant::now();
            }
        }
        file.flush().await?;
        file.sync_data().await?;
        self.store.lock().await.progress(id, received, meta.total)?;
        if let Some(total) = meta.total {
            if received != total {
                return Err(Error::Task(format!("incomplete response: {received}/{total}")));
            }
        }
        drop(file);
        if let Some(sum) = d.checksum_sha256.as_deref() {
            self.store.lock().await.transition(id, DownloadState::Verifying)?;
            verify::sha256(&part, sum).await?
        }
        tokio::fs::rename(&part, &d.destination).await?;
        self.store.lock().await.complete(id)?;
        Ok(())
    }
}
#[async_trait]
impl DownloadService for DownloadManager {
    async fn add(&self, r: AddDownloadRequest) -> Result<DownloadId> {
        let url = url::Url::parse(&r.url).map_err(|e| Error::InvalidUrl(e.to_string()))?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err(Error::InvalidUrl("only http and https are supported".into()));
        }
        let id = DownloadId::new_v4();
        let filename = r
            .output
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .map(http::sanitize_filename)
            .or_else(|| {
                url.path_segments()
                    .and_then(|mut s| s.next_back())
                    .filter(|s| !s.is_empty())
                    .map(http::sanitize_filename)
            })
            .unwrap_or_else(|| "download".into());
        let destination = r.output.unwrap_or_else(|| self.default_dir.join(&filename));
        let temp = self.default_dir.join(".hyper-get").join(id.to_string());
        let now = chrono::Utc::now().to_rfc3339();
        let d = DownloadSnapshot {
            id,
            url: r.url,
            final_url: None,
            filename,
            destination,
            temporary_directory: temp,
            state: DownloadState::Queued,
            queue_position: 0,
            total_bytes: None,
            downloaded_bytes: 0,
            connection_count: r.connections.clamp(1, 32),
            etag: None,
            last_modified: None,
            checksum_sha256: r.checksum_sha256,
            created_at: now.clone(),
            updated_at: now,
            completed_at: None,
            error: None,
        };
        self.store.lock().await.insert(&d)?;
        if r.start_immediately {
            self.start(id).await?
        }
        Ok(id)
    }
    async fn start(&self, id: DownloadId) -> Result<()> {
        let d = self.get(id).await?;
        if !matches!(
            d.state,
            DownloadState::Queued | DownloadState::Paused | DownloadState::Failed | DownloadState::Cancelled
        ) {
            return Err(Error::InvalidTransition {
                from: d.state,
                to: DownloadState::Connecting,
            });
        }
        let mut tasks = self.tasks.lock().await;
        if tasks.contains_key(&id) {
            return Err(Error::AlreadyActive);
        }
        let token = CancellationToken::new();
        tasks.insert(id, token.clone());
        let this = self.clone();
        tokio::spawn(async move {
            let _ = this.run(id, token).await;
        });
        Ok(())
    }
    async fn pause(&self, id: DownloadId) -> Result<()> {
        let d = self.get(id).await?;
        if !d.state.is_active() {
            return Err(Error::InvalidTransition {
                from: d.state,
                to: DownloadState::Paused,
            });
        }
        self.store.lock().await.transition(id, DownloadState::Pausing)?;
        if let Some(t) = self.tasks.lock().await.get(&id) {
            t.cancel()
        }
        self.store.lock().await.transition(id, DownloadState::Paused)?;
        Ok(())
    }
    async fn resume(&self, id: DownloadId) -> Result<()> {
        self.start(id).await
    }
    async fn cancel(&self, id: DownloadId) -> Result<()> {
        let d = self.get(id).await?;
        if matches!(d.state, DownloadState::Completed) {
            return Err(Error::InvalidTransition {
                from: d.state,
                to: DownloadState::Cancelled,
            });
        }
        if let Some(t) = self.tasks.lock().await.get(&id) {
            t.cancel()
        }
        self.store.lock().await.transition(id, DownloadState::Cancelled)?;
        Ok(())
    }
    async fn remove(&self, id: DownloadId, delete: bool) -> Result<()> {
        let d = self.get(id).await?;
        if d.state.is_active() {
            return Err(Error::Task("pause or cancel the active download first".into()));
        }
        if delete {
            if tokio::fs::try_exists(&d.destination).await? {
                tokio::fs::remove_file(&d.destination).await?
            }
            if tokio::fs::try_exists(&d.temporary_directory).await? {
                tokio::fs::remove_dir_all(&d.temporary_directory).await?
            }
        }
        self.store.lock().await.remove(id)
    }
    async fn list(&self, f: DownloadFilter) -> Result<Vec<DownloadSnapshot>> {
        self.store.lock().await.list(&f)
    }
    async fn get(&self, id: DownloadId) -> Result<DownloadSnapshot> {
        self.store.lock().await.get(id)
    }
    async fn start_all(&self) -> Result<()> {
        for d in self.list(Default::default()).await? {
            if matches!(
                d.state,
                DownloadState::Queued | DownloadState::Paused | DownloadState::Failed
            ) {
                self.start(d.id).await?
            }
        }
        Ok(())
    }
    async fn pause_all(&self) -> Result<()> {
        for d in self.list(Default::default()).await? {
            if d.state.is_active() {
                self.pause(d.id).await?
            }
        }
        Ok(())
    }
    async fn global_status(&self) -> Result<GlobalStatus> {
        let all = self.list(Default::default()).await?;
        let mut g = GlobalStatus::default();
        for d in all {
            g.downloaded_bytes += d.downloaded_bytes;
            if let Some(n) = d.total_bytes {
                g.known_total_bytes += n
            }
            match d.state {
                DownloadState::Queued => g.queued += 1,
                DownloadState::Paused => g.paused += 1,
                DownloadState::Completed => g.completed += 1,
                DownloadState::Failed => g.failed += 1,
                s if s.is_active() => g.active += 1,
                _ => {}
            }
        }
        g.percentage =
            (g.known_total_bytes > 0).then(|| g.downloaded_bytes as f64 * 100.0 / g.known_total_bytes as f64);
        Ok(g)
    }
}
