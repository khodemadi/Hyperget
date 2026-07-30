# Core
SQLite uses WAL and explicit migration `001_initial.sql`. Active states become `Paused` on startup. Partial bytes are reconciled from disk before requests; resumed responses and remote validators are checked before appending.

Migration `002_queue_settings.sql` preserves existing records while adding scheduler and limiter settings. Queue order is priority (`Critical`, `High`, `Normal`, `Low`), then persisted position and creation time. The scheduler serializes assignment and defaults to three active downloads. Aggregate progress is byte weighted and ignores unknown totals. A shared asynchronous rate gate applies the combined global rate; an optional per-download gate applies the stricter file rate.

Migration `003_download_preferences.sql` adds destination, last-directory, Quick Bar, wildcard, and duplicate-name preferences. Core sanitizes filenames, blocks traversal and Windows reserved device names, confines paths to the selected directory, and chooses `file (n).ext` for collisions.
