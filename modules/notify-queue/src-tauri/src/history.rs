//! Searchable notification history backed by SQLite.

use crate::queue::{Notification, Priority};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A notification record from history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub source: String,
    pub title: String,
    pub body: String,
    pub priority: String,
    pub timestamp: String,
}

pub struct HistoryDb {
    conn: Connection,
}

impl HistoryDb {
    /// Open or create the history database.
    pub fn open(db_path: &Path) -> Result<Self, String> {
        let conn =
            Connection::open(db_path).map_err(|e| format!("Failed to open history db: {}", e))?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS notifications (
                id TEXT PRIMARY KEY,
                source TEXT NOT NULL,
                title TEXT NOT NULL,
                body TEXT NOT NULL DEFAULT '',
                priority TEXT NOT NULL DEFAULT 'normal',
                timestamp TEXT NOT NULL
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS notifications_fts USING fts5(
                title, body, source,
                content='notifications',
                content_rowid='rowid',
                tokenize='porter'
            );

            CREATE TRIGGER IF NOT EXISTS notifications_ai AFTER INSERT ON notifications BEGIN
                INSERT INTO notifications_fts(rowid, title, body, source)
                VALUES (new.rowid, new.title, new.body, new.source);
            END;

            CREATE TRIGGER IF NOT EXISTS notifications_ad AFTER DELETE ON notifications BEGIN
                INSERT INTO notifications_fts(notifications_fts, rowid, title, body, source)
                VALUES ('delete', old.rowid, old.title, old.body, old.source);
            END;

            CREATE INDEX IF NOT EXISTS idx_notifications_source ON notifications(source);
            CREATE INDEX IF NOT EXISTS idx_notifications_timestamp ON notifications(timestamp);
            ",
        )
        .map_err(|e| format!("Failed to initialize history db: {}", e))?;

        Ok(Self { conn })
    }

    /// Store a notification in history.
    pub fn store(&self, notif: &Notification) -> Result<(), String> {
        let priority_str = match notif.priority {
            Priority::Low => "low",
            Priority::Normal => "normal",
            Priority::High => "high",
            Priority::Critical => "critical",
        };

        self.conn
            .execute(
                "INSERT OR REPLACE INTO notifications (id, source, title, body, priority, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    notif.id,
                    notif.source,
                    notif.title,
                    notif.body,
                    priority_str,
                    notif.timestamp.to_rfc3339(),
                ],
            )
            .map_err(|e| format!("Failed to store notification: {}", e))?;

        Ok(())
    }

    /// Search notification history using full-text search.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<HistoryEntry>, String> {
        // Sanitize: quote each term to prevent FTS5 operator injection.
        let fts_query = query
            .split_whitespace()
            .map(|w| {
                let sanitized = w.replace('"', "");
                format!("\"{}\"*", sanitized)
            })
            .collect::<Vec<_>>()
            .join(" ");

        let mut stmt = self
            .conn
            .prepare(
                "SELECT n.id, n.source, n.title, n.body, n.priority, n.timestamp
                 FROM notifications n
                 JOIN notifications_fts f ON n.rowid = f.rowid
                 WHERE notifications_fts MATCH ?1
                 ORDER BY bm25(notifications_fts) ASC
                 LIMIT ?2",
            )
            .map_err(|e| format!("Search prepare error: {}", e))?;

        let rows = stmt
            .query_map(params![fts_query, limit as i64], |row| {
                Ok(HistoryEntry {
                    id: row.get(0)?,
                    source: row.get(1)?,
                    title: row.get(2)?,
                    body: row.get(3)?,
                    priority: row.get(4)?,
                    timestamp: row.get(5)?,
                })
            })
            .map_err(|e| format!("Search query error: {}", e))?;

        let mut results = Vec::new();
        for row in rows {
            if let Ok(entry) = row {
                results.push(entry);
            }
        }
        Ok(results)
    }

    /// Get recent history entries.
    pub fn get_recent(&self, limit: usize) -> Result<Vec<HistoryEntry>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, source, title, body, priority, timestamp
                 FROM notifications
                 ORDER BY timestamp DESC
                 LIMIT ?1",
            )
            .map_err(|e| format!("Recent query error: {}", e))?;

        let rows = stmt
            .query_map(params![limit as i64], |row| {
                Ok(HistoryEntry {
                    id: row.get(0)?,
                    source: row.get(1)?,
                    title: row.get(2)?,
                    body: row.get(3)?,
                    priority: row.get(4)?,
                    timestamp: row.get(5)?,
                })
            })
            .map_err(|e| format!("Recent query error: {}", e))?;

        let mut results = Vec::new();
        for row in rows {
            if let Ok(entry) = row {
                results.push(entry);
            }
        }
        Ok(results)
    }

    /// Get history filtered by source.
    pub fn get_by_source(&self, source: &str, limit: usize) -> Result<Vec<HistoryEntry>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, source, title, body, priority, timestamp
                 FROM notifications
                 WHERE source = ?1
                 ORDER BY timestamp DESC
                 LIMIT ?2",
            )
            .map_err(|e| format!("Source query error: {}", e))?;

        let rows = stmt
            .query_map(params![source, limit as i64], |row| {
                Ok(HistoryEntry {
                    id: row.get(0)?,
                    source: row.get(1)?,
                    title: row.get(2)?,
                    body: row.get(3)?,
                    priority: row.get(4)?,
                    timestamp: row.get(5)?,
                })
            })
            .map_err(|e| format!("Source query error: {}", e))?;

        let mut results = Vec::new();
        for row in rows {
            if let Ok(entry) = row {
                results.push(entry);
            }
        }
        Ok(results)
    }

    /// Purge notifications older than `retention_days`.
    pub fn purge(&self, retention_days: u32) -> Result<usize, String> {
        let cutoff = Utc::now() - chrono::Duration::days(retention_days as i64);
        let cutoff_str = cutoff.to_rfc3339();

        let count = self
            .conn
            .execute(
                "DELETE FROM notifications WHERE timestamp < ?1",
                params![cutoff_str],
            )
            .map_err(|e| format!("Purge error: {}", e))?;

        if count > 0 {
            log::info!("Purged {} notifications older than {} days", count, retention_days);
        }

        Ok(count)
    }

    /// Get total count of notifications in history.
    pub fn count(&self) -> Result<usize, String> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM notifications", [], |row| row.get(0))
            .map_err(|e| format!("Count error: {}", e))?;
        Ok(count as usize)
    }
}
