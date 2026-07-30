use crate::{DownloadFilter, DownloadId, DownloadSnapshot, DownloadState, Error, Priority, Result, Settings};
use rusqlite::{Connection, OptionalExtension, params};
use std::{path::Path, str::FromStr};
pub struct Store {
    conn: Connection,
}
impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p)?
        }
        let conn = Connection::open(path)?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        conn.execute_batch(include_str!("../migrations/001_initial.sql"))?;
        let migrated: bool = conn
            .prepare("SELECT 1 FROM pragma_table_info('downloads') WHERE name='started_at'")?
            .exists([])?;
        if !migrated {
            conn.execute_batch(include_str!("../migrations/002_queue_settings.sql"))?;
        }
        conn.execute_batch(include_str!("../migrations/003_download_preferences.sql"))?;
        let s = Self { conn };
        s.recover()?;
        Ok(s)
    }
    fn recover(&self) -> Result<()> {
        self.conn.execute("UPDATE downloads SET status='Paused', updated_at=?1 WHERE status IN ('Resolving','Connecting','Downloading','Pausing','RetryWaiting','Merging','Verifying')",[chrono::Utc::now().to_rfc3339()])?;
        Ok(())
    }
    pub fn insert(&mut self, d: &DownloadSnapshot) -> Result<()> {
        let tx = self.conn.transaction()?;
        let max: i64 = tx.query_row(
            "SELECT COALESCE(MAX(queue_position),-1)+1 FROM downloads",
            [],
            |r| r.get(0),
        )?;
        tx.execute("INSERT INTO downloads(id,url,final_url,filename,destination,temporary_directory,status,priority,queue_position,total_bytes,downloaded_bytes,connection_count,etag,last_modified,checksum_sha256,created_at,updated_at,start_immediately,per_download_speed_limit,started_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20)",params![d.id.to_string(),d.url,d.final_url,d.filename,d.destination.to_string_lossy(),d.temporary_directory.to_string_lossy(),state(d.state),priority(d.priority),max,d.total_bytes.map(|v|v as i64),d.downloaded_bytes as i64,d.connection_count,d.etag,d.last_modified,d.checksum_sha256,d.created_at,d.updated_at,d.start_immediately,d.per_download_speed_limit.map(|v|v as i64),d.started_at])?;
        tx.commit()?;
        Ok(())
    }
    pub fn get(&self, id: DownloadId) -> Result<DownloadSnapshot> {
        self.conn
            .query_row(&format!("{} WHERE id=?1", SELECT), [id.to_string()], row)
            .optional()?
            .ok_or(Error::NotFound(id))
    }
    pub fn list(&self, f: &DownloadFilter) -> Result<Vec<DownloadSnapshot>> {
        let mut st=self.conn.prepare(&format!("{} ORDER BY CASE priority WHEN 'Critical' THEN 3 WHEN 'High' THEN 2 WHEN 'Normal' THEN 1 ELSE 0 END DESC, queue_position, created_at",SELECT))?;
        let all = st
            .query_map([], row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(all
            .into_iter()
            .filter(|d| {
                f.state.is_none_or(|s| s == d.state)
                    && f.search.as_ref().is_none_or(|q| {
                        d.filename.to_lowercase().contains(&q.to_lowercase()) || d.url.contains(q)
                    })
            })
            .collect())
    }
    pub fn transition(&self, id: DownloadId, to: DownloadState) -> Result<()> {
        let d = self.get(id)?;
        if !d.state.can_transition(to) {
            return Err(Error::InvalidTransition { from: d.state, to });
        }
        self.conn.execute(
            "UPDATE downloads SET status=?1,updated_at=?2,error_message=NULL WHERE id=?3",
            params![state(to), chrono::Utc::now().to_rfc3339(), id.to_string()],
        )?;
        Ok(())
    }
    pub fn progress(&self, id: DownloadId, n: u64, total: Option<u64>) -> Result<()> {
        self.conn.execute("UPDATE downloads SET downloaded_bytes=?1,total_bytes=COALESCE(?2,total_bytes),updated_at=?3 WHERE id=?4",params![n as i64,total.map(|v|v as i64),chrono::Utc::now().to_rfc3339(),id.to_string()])?;
        Ok(())
    }
    pub fn metadata(
        &self,
        id: DownloadId,
        final_url: &str,
        total: Option<u64>,
        etag: Option<&str>,
        last: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE downloads SET final_url=?1,total_bytes=?2,etag=?3,last_modified=?4 WHERE id=?5",
            params![final_url, total.map(|v| v as i64), etag, last, id.to_string()],
        )?;
        Ok(())
    }
    pub fn complete(&self, id: DownloadId) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE downloads SET status='Completed',completed_at=?1,updated_at=?1 WHERE id=?2",
            params![now, id.to_string()],
        )?;
        Ok(())
    }
    pub fn fail(&self, id: DownloadId, msg: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE downloads SET status='Failed',error_message=?1,updated_at=?2 WHERE id=?3",
            params![msg, chrono::Utc::now().to_rfc3339(), id.to_string()],
        )?;
        Ok(())
    }
    pub fn remove(&self, id: DownloadId) -> Result<()> {
        if self
            .conn
            .execute("DELETE FROM downloads WHERE id=?1", [id.to_string()])?
            == 0
        {
            return Err(Error::NotFound(id));
        }
        Ok(())
    }
    pub fn set_priority(&self, id: DownloadId, p: Priority) -> Result<()> {
        self.get(id)?;
        self.conn.execute(
            "UPDATE downloads SET priority=?1,updated_at=?2 WHERE id=?3",
            params![priority(p), chrono::Utc::now().to_rfc3339(), id.to_string()],
        )?;
        Ok(())
    }
    pub fn reorder(&mut self, ids: &[DownloadId]) -> Result<()> {
        let tx = self.conn.transaction()?;
        for (id, pos) in ids.iter().zip(0_i64..) {
            tx.execute(
                "UPDATE downloads SET queue_position=?1 WHERE id=?2",
                params![pos, id.to_string()],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
    pub fn settings(&self) -> Result<Settings> {
        let get = |key: &str| -> Result<Option<String>> {
            Ok(self
                .conn
                .query_row("SELECT value FROM settings WHERE key=?1", [key], |r| r.get(0))
                .optional()?)
        };
        let d = Settings::default();
        Ok(Settings {
            maximum_simultaneous_downloads: get("maximum_simultaneous_downloads")?
                .and_then(|v| v.parse().ok())
                .unwrap_or(d.maximum_simultaneous_downloads),
            default_connections_per_file: get("default_connections_per_file")?
                .and_then(|v| v.parse().ok())
                .unwrap_or(d.default_connections_per_file),
            default_retry_count: get("default_retry_count")?
                .and_then(|v| v.parse().ok())
                .unwrap_or(d.default_retry_count),
            initial_retry_delay_seconds: get("initial_retry_delay_seconds")?
                .and_then(|v| v.parse().ok())
                .unwrap_or(d.initial_retry_delay_seconds),
            maximum_retry_delay_seconds: get("maximum_retry_delay_seconds")?
                .and_then(|v| v.parse().ok())
                .unwrap_or(d.maximum_retry_delay_seconds),
            auto_retry: get("auto_retry")?.is_none_or(|v| v == "true"),
            auto_start_next: get("auto_start_next")?.is_none_or(|v| v == "true"),
            restore_unfinished_downloads: get("restore_unfinished_downloads")?.is_none_or(|v| v == "true"),
            global_speed_limit_mode: get("global_speed_limit_mode")?.unwrap_or(d.global_speed_limit_mode),
            global_speed_limit_bytes: get("global_speed_limit_bytes")?
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            default_priority: parse_priority(&get("default_priority")?.unwrap_or_default()),
            confirm_before_delete: get("confirm_before_delete")?.is_none_or(|v| v == "true"),
            theme: get("theme")?.unwrap_or(d.theme),
            default_download_directory: get("default_download_directory")?.unwrap_or_default(),
            ask_where_to_save: get("ask_where_to_save")?.is_some_and(|v| v == "true"),
            remember_last_directory: get("remember_last_directory")?.is_none_or(|v| v == "true"),
            last_selected_directory: get("last_selected_directory")?.unwrap_or_default(),
            create_category_subfolders: get("create_category_subfolders")?.is_some_and(|v| v == "true"),
            wildcard_batch_behavior: get("wildcard_batch_behavior")?.unwrap_or_else(|| "preview".into()),
            wildcard_auto_start: get("wildcard_auto_start")?.is_some_and(|v| v == "true"),
            quick_download_bar_expanded: get("quick_download_bar_expanded")?.is_none_or(|v| v == "true"),
            duplicate_filename_behavior: get("duplicate_filename_behavior")?
                .unwrap_or_else(|| "rename".into()),
        })
    }
    pub fn update_settings(&mut self, s: &Settings) -> Result<()> {
        if !(1..=32).contains(&s.maximum_simultaneous_downloads) {
            return Err(Error::Task(
                "maximum simultaneous downloads must be 1..=32".into(),
            ));
        }
        let tx = self.conn.transaction()?;
        let vals = [
            (
                "maximum_simultaneous_downloads",
                s.maximum_simultaneous_downloads.to_string(),
            ),
            (
                "default_connections_per_file",
                s.default_connections_per_file.to_string(),
            ),
            ("global_speed_limit_mode", s.global_speed_limit_mode.clone()),
            ("global_speed_limit_bytes", s.global_speed_limit_bytes.to_string()),
            ("auto_start_next", s.auto_start_next.to_string()),
            ("default_priority", priority(s.default_priority).into()),
            ("confirm_before_delete", s.confirm_before_delete.to_string()),
            ("theme", s.theme.clone()),
            ("default_download_directory", s.default_download_directory.clone()),
            ("ask_where_to_save", s.ask_where_to_save.to_string()),
            ("remember_last_directory", s.remember_last_directory.to_string()),
            ("last_selected_directory", s.last_selected_directory.clone()),
            (
                "create_category_subfolders",
                s.create_category_subfolders.to_string(),
            ),
            ("wildcard_batch_behavior", s.wildcard_batch_behavior.clone()),
            ("wildcard_auto_start", s.wildcard_auto_start.to_string()),
            (
                "quick_download_bar_expanded",
                s.quick_download_bar_expanded.to_string(),
            ),
            (
                "duplicate_filename_behavior",
                s.duplicate_filename_behavior.clone(),
            ),
        ];
        for (k, v) in vals {
            tx.execute("INSERT INTO settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",params![k,v])?;
        }
        tx.commit()?;
        Ok(())
    }
}
const SELECT: &str = "SELECT id,url,final_url,filename,destination,temporary_directory,status,queue_position,total_bytes,downloaded_bytes,connection_count,etag,last_modified,checksum_sha256,created_at,updated_at,completed_at,error_message,CAST(priority AS TEXT),start_immediately,per_download_speed_limit,started_at FROM downloads";
fn priority(p: Priority) -> &'static str {
    match p {
        Priority::Low => "Low",
        Priority::Normal => "Normal",
        Priority::High => "High",
        Priority::Critical => "Critical",
    }
}
fn parse_priority(p: &str) -> Priority {
    match p {
        "Low" | "low" => Priority::Low,
        "High" | "high" => Priority::High,
        "Critical" | "critical" => Priority::Critical,
        _ => Priority::Normal,
    }
}
fn state(s: DownloadState) -> &'static str {
    match s {
        DownloadState::Created => "Created",
        DownloadState::Resolving => "Resolving",
        DownloadState::Queued => "Queued",
        DownloadState::Connecting => "Connecting",
        DownloadState::Downloading => "Downloading",
        DownloadState::Pausing => "Pausing",
        DownloadState::Paused => "Paused",
        DownloadState::RetryWaiting => "RetryWaiting",
        DownloadState::Merging => "Merging",
        DownloadState::Verifying => "Verifying",
        DownloadState::Completed => "Completed",
        DownloadState::Failed => "Failed",
        DownloadState::Cancelled => "Cancelled",
    }
}
fn parse(s: &str) -> DownloadState {
    match s {
        "Created" => DownloadState::Created,
        "Resolving" => DownloadState::Resolving,
        "Queued" => DownloadState::Queued,
        "Connecting" => DownloadState::Connecting,
        "Downloading" => DownloadState::Downloading,
        "Pausing" => DownloadState::Pausing,
        "Paused" => DownloadState::Paused,
        "RetryWaiting" => DownloadState::RetryWaiting,
        "Merging" => DownloadState::Merging,
        "Verifying" => DownloadState::Verifying,
        "Completed" => DownloadState::Completed,
        "Failed" => DownloadState::Failed,
        "Cancelled" => DownloadState::Cancelled,
        _ => DownloadState::Failed,
    }
}
fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<DownloadSnapshot> {
    let id: String = r.get(0)?;
    Ok(DownloadSnapshot {
        id: DownloadId::from_str(&id).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?,
        url: r.get(1)?,
        final_url: r.get(2)?,
        filename: r.get(3)?,
        destination: r.get::<_, String>(4)?.into(),
        temporary_directory: r.get::<_, String>(5)?.into(),
        state: parse(&r.get::<_, String>(6)?),
        queue_position: r.get(7)?,
        priority: parse_priority(&r.get::<_, String>(18)?),
        start_immediately: r.get(19)?,
        per_download_speed_limit: r.get::<_, Option<i64>>(20)?.map(|v| v as u64),
        total_bytes: r.get::<_, Option<i64>>(8)?.map(|v| v as u64),
        downloaded_bytes: r.get::<_, i64>(9)? as u64,
        connection_count: r.get(10)?,
        etag: r.get(11)?,
        last_modified: r.get(12)?,
        checksum_sha256: r.get(13)?,
        created_at: r.get(14)?,
        started_at: r.get(21)?,
        updated_at: r.get(15)?,
        completed_at: r.get(16)?,
        error: r.get(17)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn settings_survive_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.sqlite3");
        {
            let mut store = Store::open(&path).unwrap();
            let mut settings = store.settings().unwrap();
            settings.maximum_simultaneous_downloads = 7;
            settings.global_speed_limit_mode = "custom".into();
            settings.global_speed_limit_bytes = 5 * 1024 * 1024;
            store.update_settings(&settings).unwrap();
        }
        let store = Store::open(&path).unwrap();
        let settings = store.settings().unwrap();
        assert_eq!(settings.maximum_simultaneous_downloads, 7);
        assert_eq!(settings.global_speed_limit_bytes, 5 * 1024 * 1024);
    }
}
