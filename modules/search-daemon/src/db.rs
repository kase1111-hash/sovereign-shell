//! SQLite database for the search daemon file index.
//!
//! Uses FTS5 with porter stemming for full-text search over file metadata
//! and content. Database lives at `%APPDATA%\SovereignShell\search-daemon\data\index.db`

use rusqlite::{params, Connection, Result as SqlResult};
use std::path::PathBuf;

/// A single indexed file entry (metadata).
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileMeta {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub extension: Option<String>,
    pub size: i64,
    pub modified: i64,
    pub content_indexed: bool,
    pub last_seen: i64,
}

/// A search hit with relevance score and optional content snippet.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchHit {
    pub path: String,
    pub name: String,
    pub extension: Option<String>,
    pub score: f64,
    pub snippet: Option<String>,
    pub size: i64,
    pub modified: i64,
}

/// Status information about the index.
#[derive(Debug, Clone, serde::Serialize)]
pub struct IndexStatus {
    pub indexed_files: i64,
    pub index_size_bytes: u64,
    pub state: String,
}

pub struct Database {
    conn: Connection,
    db_path: PathBuf,
}

impl Database {
    /// Open or create the search daemon database.
    pub fn open(db_path: &PathBuf) -> SqlResult<Self> {
        let conn = Connection::open(db_path)?;

        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA synchronous=NORMAL;")?;
        conn.execute_batch("PRAGMA cache_size=-64000;")?; // 64MB cache

        conn.execute_batch(
            "
            -- Metadata table for all indexed files
            CREATE TABLE IF NOT EXISTS files_meta (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT UNIQUE NOT NULL,
                name TEXT NOT NULL,
                extension TEXT,
                size INTEGER NOT NULL DEFAULT 0,
                modified INTEGER NOT NULL DEFAULT 0,
                content_indexed INTEGER NOT NULL DEFAULT 0,
                last_seen INTEGER NOT NULL DEFAULT 0
            );

            CREATE INDEX IF NOT EXISTS idx_files_path ON files_meta(path);
            CREATE INDEX IF NOT EXISTS idx_files_ext ON files_meta(extension);
            CREATE INDEX IF NOT EXISTS idx_files_modified ON files_meta(modified);

            -- FTS5 virtual table for full-text search
            CREATE VIRTUAL TABLE IF NOT EXISTS files_fts USING fts5(
                name,
                path,
                extension,
                content,
                parent_dir,
                tokenize='porter unicode61'
            );
            ",
        )?;

        Ok(Self {
            conn,
            db_path: db_path.clone(),
        })
    }

    /// Open database in the standard config location.
    pub fn open_default() -> SqlResult<Self> {
        let data_dir = sovereign_config::data_dir("search-daemon")
            .map_err(|e| rusqlite::Error::InvalidParameterName(format!("{e}")))?;
        let db_path = data_dir.join("index.db");
        Self::open(&db_path)
    }

    /// Insert or update a file's metadata and FTS entry.
    pub fn upsert_file(
        &self,
        path: &str,
        name: &str,
        extension: Option<&str>,
        size: i64,
        modified: i64,
        parent_dir: &str,
        content: Option<&str>,
        crawl_stamp: i64,
    ) -> SqlResult<()> {
        let content_indexed = content.is_some() as i32;

        // Upsert metadata
        self.conn.execute(
            "INSERT INTO files_meta (path, name, extension, size, modified, content_indexed, last_seen)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(path) DO UPDATE SET
                name = excluded.name,
                extension = excluded.extension,
                size = excluded.size,
                modified = excluded.modified,
                content_indexed = excluded.content_indexed,
                last_seen = excluded.last_seen",
            params![path, name, extension, size, modified, content_indexed, crawl_stamp],
        )?;

        // Delete old FTS entry if exists, then insert new one.
        // FTS5 external-content tables require manual sync.
        self.conn.execute(
            "DELETE FROM files_fts WHERE path = ?1",
            params![path],
        )?;

        self.conn.execute(
            "INSERT INTO files_fts (name, path, extension, content, parent_dir)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![name, path, extension.unwrap_or(""), content.unwrap_or(""), parent_dir],
        )?;

        Ok(())
    }

    /// Remove a file from the index by path.
    pub fn remove_file(&self, path: &str) -> SqlResult<()> {
        self.conn.execute("DELETE FROM files_meta WHERE path = ?1", params![path])?;
        self.conn.execute("DELETE FROM files_fts WHERE path = ?1", params![path])?;
        Ok(())
    }

    /// Remove all files that were not seen in the current crawl pass.
    pub fn prune_unseen(&self, crawl_stamp: i64) -> SqlResult<usize> {
        // Get paths to remove
        let mut stmt = self.conn.prepare(
            "SELECT path FROM files_meta WHERE last_seen < ?1"
        )?;
        let stale_paths: Vec<String> = stmt
            .query_map(params![crawl_stamp], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        let count = stale_paths.len();
        for path in &stale_paths {
            self.remove_file(path)?;
        }
        Ok(count)
    }

    /// Search for files matching a query.
    pub fn search(
        &self,
        query: &str,
        max_results: usize,
        file_types: Option<&[String]>,
    ) -> SqlResult<Vec<SearchHit>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        // Build FTS5 query with prefix matching.
        // Sanitize: quote each term to prevent FTS5 operator injection.
        let fts_query: String = query
            .split_whitespace()
            .map(|term| {
                let sanitized = term.replace('"', "");
                format!("\"{}\"*", sanitized)
            })
            .collect::<Vec<_>>()
            .join(" ");

        let (sql, has_type_filter) = if file_types.is_some() && !file_types.unwrap().is_empty() {
            // With extension filter — join through metadata
            (
                "SELECT f.path, f.name, m.extension, bm25(files_fts, 10.0, 1.0, 2.0, 5.0, 1.0) as rank,
                        snippet(files_fts, 3, '<b>', '</b>', '...', 32) as snip,
                        m.size, m.modified
                 FROM files_fts f
                 JOIN files_meta m ON m.path = f.path
                 WHERE files_fts MATCH ?1 AND m.extension IN (SELECT value FROM json_each(?3))
                 ORDER BY rank
                 LIMIT ?2".to_string(),
                true,
            )
        } else {
            (
                "SELECT f.path, f.name, f.extension, bm25(files_fts, 10.0, 1.0, 2.0, 5.0, 1.0) as rank,
                        snippet(files_fts, 3, '<b>', '</b>', '...', 32) as snip,
                        COALESCE(m.size, 0), COALESCE(m.modified, 0)
                 FROM files_fts f
                 LEFT JOIN files_meta m ON m.path = f.path
                 WHERE files_fts MATCH ?1
                 ORDER BY rank
                 LIMIT ?2".to_string(),
                false,
            )
        };

        let mut stmt = self.conn.prepare(&sql)?;

        let rows = if has_type_filter {
            let types_json = serde_json::to_string(file_types.unwrap())
                .unwrap_or_else(|_| "[]".to_string());
            stmt.query_map(params![fts_query, max_results, types_json], |row| {
                Ok(SearchHit {
                    path: row.get(0)?,
                    name: row.get(1)?,
                    extension: row.get(2)?,
                    score: row.get::<_, f64>(3)?.abs(),
                    snippet: row.get(4)?,
                    size: row.get(5)?,
                    modified: row.get(6)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect()
        } else {
            stmt.query_map(params![fts_query, max_results], |row| {
                Ok(SearchHit {
                    path: row.get(0)?,
                    name: row.get(1)?,
                    extension: row.get(2)?,
                    score: row.get::<_, f64>(3)?.abs(),
                    snippet: row.get(4)?,
                    size: row.get(5)?,
                    modified: row.get(6)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect()
        };

        Ok(rows)
    }

    /// Get index statistics.
    pub fn status(&self) -> SqlResult<IndexStatus> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM files_meta", [], |row| row.get(0),
        )?;

        let size = std::fs::metadata(&self.db_path)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(IndexStatus {
            indexed_files: count,
            index_size_bytes: size,
            state: "idle".to_string(),
        })
    }

    /// Get the count of indexed files.
    pub fn count(&self) -> SqlResult<i64> {
        self.conn.query_row("SELECT COUNT(*) FROM files_meta", [], |row| row.get(0))
    }

    /// Begin a transaction for batch operations.
    pub fn begin_batch(&self) -> SqlResult<()> {
        self.conn.execute_batch("BEGIN TRANSACTION")?;
        Ok(())
    }

    /// Commit a batch transaction.
    pub fn commit_batch(&self) -> SqlResult<()> {
        self.conn.execute_batch("COMMIT")?;
        Ok(())
    }
}
