use crate::{DownloadFilter, DownloadId, DownloadSnapshot, DownloadState, Error, Result};
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
        tx.execute("INSERT INTO downloads(id,url,final_url,filename,destination,temporary_directory,status,queue_position,total_bytes,downloaded_bytes,connection_count,etag,last_modified,checksum_sha256,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",params![d.id.to_string(),d.url,d.final_url,d.filename,d.destination.to_string_lossy(),d.temporary_directory.to_string_lossy(),state(d.state),max,d.total_bytes.map(|v|v as i64),d.downloaded_bytes as i64,d.connection_count,d.etag,d.last_modified,d.checksum_sha256,d.created_at,d.updated_at])?;
        tx.commit()?;
        Ok(())
    }
    pub fn get(&self, id: DownloadId) -> Result<DownloadSnapshot> {
        self.conn.query_row("SELECT id,url,final_url,filename,destination,temporary_directory,status,queue_position,total_bytes,downloaded_bytes,connection_count,etag,last_modified,checksum_sha256,created_at,updated_at,completed_at,error_message FROM downloads WHERE id=?1",[id.to_string()],row).optional()?.ok_or(Error::NotFound(id))
    }
    pub fn list(&self, f: &DownloadFilter) -> Result<Vec<DownloadSnapshot>> {
        let mut st=self.conn.prepare("SELECT id,url,final_url,filename,destination,temporary_directory,status,queue_position,total_bytes,downloaded_bytes,connection_count,etag,last_modified,checksum_sha256,created_at,updated_at,completed_at,error_message FROM downloads ORDER BY queue_position")?;
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
        total_bytes: r.get::<_, Option<i64>>(8)?.map(|v| v as u64),
        downloaded_bytes: r.get::<_, i64>(9)? as u64,
        connection_count: r.get(10)?,
        etag: r.get(11)?,
        last_modified: r.get(12)?,
        checksum_sha256: r.get(13)?,
        created_at: r.get(14)?,
        updated_at: r.get(15)?,
        completed_at: r.get(16)?,
        error: r.get(17)?,
    })
}
