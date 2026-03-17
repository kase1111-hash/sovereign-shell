//! SQLite database for the launcher index.
//!
//! Uses FTS5 for fast fuzzy-prefix search over indexed applications.
//! Database lives at `%APPDATA%\SovereignShell\launcher\data\index.db`

use rusqlite::{params, Connection, Result as SqlResult};
use std::path::PathBuf;

/// A single indexed application entry.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AppEntry {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub keywords: String,
    pub launch_count: i64,
    pub last_launched: Option<i64>,
    pub icon_path: Option<String>,
}

/// Search result with relevance score.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResult {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub launch_count: i64,
    pub score: f64,
    pub icon_path: Option<String>,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create the launcher database.
    pub fn open(db_path: &PathBuf) -> SqlResult<Self> {
        let conn = Connection::open(db_path)?;

        // Enable WAL mode for better concurrent read performance
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        // Create tables
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS apps (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE,
                keywords TEXT NOT NULL DEFAULT '',
                launch_count INTEGER NOT NULL DEFAULT 0,
                last_launched INTEGER,
                icon_path TEXT
            );

            -- FTS5 virtual table for fast search
            CREATE VIRTUAL TABLE IF NOT EXISTS apps_fts USING fts5(
                name,
                keywords,
                content='apps',
                content_rowid='id',
                tokenize='unicode61'
            );

            -- Triggers to keep FTS in sync with the apps table
            CREATE TRIGGER IF NOT EXISTS apps_ai AFTER INSERT ON apps BEGIN
                INSERT INTO apps_fts(rowid, name, keywords)
                VALUES (new.id, new.name, new.keywords);
            END;

            CREATE TRIGGER IF NOT EXISTS apps_ad AFTER DELETE ON apps BEGIN
                INSERT INTO apps_fts(apps_fts, rowid, name, keywords)
                VALUES ('delete', old.id, old.name, old.keywords);
            END;

            CREATE TRIGGER IF NOT EXISTS apps_au AFTER UPDATE ON apps BEGIN
                INSERT INTO apps_fts(apps_fts, rowid, name, keywords)
                VALUES ('delete', old.id, old.name, old.keywords);
                INSERT INTO apps_fts(rowid, name, keywords)
                VALUES (new.id, new.name, new.keywords);
            END;
            ",
        )?;

        Ok(Self { conn })
    }

    /// Open database in the standard config location.
    pub fn open_default() -> SqlResult<Self> {
        let data_dir = sovereign_config::data_dir("launcher")
            .map_err(|e| rusqlite::Error::InvalidParameterName(format!("{e}")))?;
        let db_path = data_dir.join("index.db");
        Self::open(&db_path)
    }

    /// Insert or update an application entry.
    /// If the path already exists, updates name and keywords but preserves launch_count.
    pub fn upsert_app(
        &self,
        name: &str,
        path: &str,
        keywords: &str,
        icon_path: Option<&str>,
    ) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO apps (name, path, keywords, icon_path)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(path) DO UPDATE SET
                name = excluded.name,
                keywords = excluded.keywords,
                icon_path = excluded.icon_path",
            params![name, path, keywords, icon_path],
        )?;
        Ok(())
    }

    /// Remove entries whose paths no longer exist on disk.
    pub fn prune_missing(&self) -> SqlResult<usize> {
        let mut stmt = self.conn.prepare("SELECT id, path FROM apps")?;
        let entries: Vec<(i64, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();

        let mut removed = 0;
        for (id, path) in entries {
            if !std::path::Path::new(&path).exists() {
                self.conn.execute("DELETE FROM apps WHERE id = ?1", params![id])?;
                removed += 1;
            }
        }
        Ok(removed)
    }

    /// Search for apps matching the query string.
    /// Uses FTS5 prefix matching and boosts by launch_count.
    pub fn search(&self, query: &str, max_results: usize) -> SqlResult<Vec<SearchResult>> {
        if query.trim().is_empty() {
            // Return most-launched apps when query is empty
            let mut stmt = self.conn.prepare(
                "SELECT id, name, path, launch_count, icon_path
                 FROM apps
                 ORDER BY launch_count DESC, name ASC
                 LIMIT ?1",
            )?;
            let results = stmt
                .query_map(params![max_results], |row| {
                    Ok(SearchResult {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        path: row.get(2)?,
                        launch_count: row.get(3)?,
                        score: 1.0,
                        icon_path: row.get(4)?,
                    })
                })?
                .filter_map(|r| r.ok())
                .collect();
            return Ok(results);
        }

        // Prepare FTS5 query — add * for prefix matching on each term
        let fts_query: String = query
            .split_whitespace()
            .map(|term| format!("{}*", term.replace('"', "")))
            .collect::<Vec<_>>()
            .join(" ");

        let mut stmt = self.conn.prepare(
            "SELECT a.id, a.name, a.path, a.launch_count,
                    bm25(apps_fts, 10.0, 1.0) as rank, a.icon_path
             FROM apps_fts f
             JOIN apps a ON a.id = f.rowid
             WHERE apps_fts MATCH ?1
             ORDER BY (rank * -1.0) + (a.launch_count * 0.5) DESC
             LIMIT ?2",
        )?;

        let results = stmt
            .query_map(params![fts_query, max_results], |row| {
                Ok(SearchResult {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    launch_count: row.get(3)?,
                    score: row.get::<_, f64>(4)?.abs(),
                    icon_path: row.get(5)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(results)
    }

    /// Record a launch — increment count and update timestamp.
    pub fn record_launch(&self, id: i64) -> SqlResult<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        self.conn.execute(
            "UPDATE apps SET launch_count = launch_count + 1, last_launched = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    }

    /// Get total number of indexed apps.
    pub fn count(&self) -> SqlResult<i64> {
        self.conn.query_row("SELECT COUNT(*) FROM apps", [], |row| row.get(0))
    }
}
