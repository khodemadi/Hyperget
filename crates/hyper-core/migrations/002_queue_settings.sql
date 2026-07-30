ALTER TABLE downloads ADD COLUMN start_immediately INTEGER NOT NULL DEFAULT 0;
ALTER TABLE downloads ADD COLUMN per_download_speed_limit INTEGER;
ALTER TABLE downloads ADD COLUMN started_at TEXT;
INSERT OR IGNORE INTO settings(key,value) VALUES
 ('maximum_simultaneous_downloads','3'),('default_connections_per_file','8'),
 ('default_retry_count','5'),('initial_retry_delay_seconds','1'),
 ('maximum_retry_delay_seconds','30'),('auto_retry','true'),('auto_start_next','true'),
 ('restore_unfinished_downloads','true'),('global_speed_limit_mode','unlimited'),
 ('global_speed_limit_bytes','0'),('default_priority','normal'),
 ('confirm_before_delete','true'),('theme','system');
