PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS downloads (
 id TEXT PRIMARY KEY, url TEXT NOT NULL, final_url TEXT, filename TEXT NOT NULL,
 destination TEXT NOT NULL, temporary_directory TEXT NOT NULL, status TEXT NOT NULL,
 priority INTEGER NOT NULL DEFAULT 0, queue_position INTEGER NOT NULL,
 total_bytes INTEGER, downloaded_bytes INTEGER NOT NULL DEFAULT 0,
 connection_count INTEGER NOT NULL DEFAULT 4, etag TEXT, last_modified TEXT, media_type TEXT,
 checksum_sha256 TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, completed_at TEXT,
 error_code TEXT, error_message TEXT
);
CREATE TABLE IF NOT EXISTS segments (
 download_id TEXT NOT NULL REFERENCES downloads(id) ON DELETE CASCADE,
 segment_index INTEGER NOT NULL, start_byte INTEGER NOT NULL, end_byte INTEGER NOT NULL,
 downloaded_bytes INTEGER NOT NULL DEFAULT 0, status TEXT NOT NULL,
 retry_count INTEGER NOT NULL DEFAULT 0, temporary_path TEXT NOT NULL,
 PRIMARY KEY(download_id, segment_index)
);
CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS downloads_queue ON downloads(queue_position);

