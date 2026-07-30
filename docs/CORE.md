# Core
SQLite uses WAL and explicit migration `001_initial.sql`. Active states become `Paused` on startup. Partial bytes are reconciled from disk before requests; resumed responses and remote validators are checked before appending.
