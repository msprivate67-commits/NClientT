//! SQLite-backed persistence.
//!
//! Port of NClientV3's `async/database` package. Tables:
//! - `favorites`   : local favorite galleries
//! - `history`     : recently visited galleries
//! - `tags`        : cached tags (with a user `status` flag and an
//!                   `online_blacklist` flag)
//! - `downloads`   : in-progress / paused downloads (resumable across restarts)
//! - `local_meta`  : scanned local gallery metadata cache

use std::path::Path;
use std::sync::Arc;

use chrono::Utc;
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::models::{HistoryEntry, LocalGallery, Tag, TagStatus, TagType};

/// Thread-safe SQLite handle. All access goes through a single `Mutex`; given
/// the access pattern of a desktop client this is plenty.
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Database").finish()
    }
}

impl Database {
    pub fn open(app_data: &Path) -> AppResult<Self> {
        std::fs::create_dir_all(app_data).ok();
        let path = app_data.join("nclientt.db");
        let conn = Connection::open(&path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        migrate(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn with_conn<F, T>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&Connection) -> AppResult<T>,
    {
        let conn = self.conn.lock();
        f(&conn)
    }

    // ---------------------------------------------------------------------
    // Favorites
    // ---------------------------------------------------------------------

    pub fn fav_add(
        &self,
        id: i64,
        title: &str,
        media_id: i64,
        thumbnail: &str,
    ) -> AppResult<()> {
        self.with_conn(|c| {
            c.execute(
                "INSERT OR REPLACE INTO favorites (id, title, media_id, thumbnail, added_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, title, media_id, thumbnail, Utc::now().to_rfc3339()],
            )?;
            Ok(())
        })
    }

    pub fn fav_remove(&self, id: i64) -> AppResult<()> {
        self.with_conn(|c| {
            c.execute("DELETE FROM favorites WHERE id = ?1", params![id])?;
            Ok(())
        })
    }

    pub fn fav_is(&self, id: i64) -> AppResult<bool> {
        self.with_conn(|c| {
            let v: i64 = c.query_row(
                "SELECT COUNT(*) FROM favorites WHERE id = ?1",
                params![id],
                |r| r.get(0),
            )?;
            Ok(v > 0)
        })
    }

    pub fn fav_list(&self, limit: u32, offset: u32) -> AppResult<Vec<FavoriteRow>> {
        self.with_conn(|c| {
            let mut stmt = c.prepare(
                "SELECT id, title, media_id, thumbnail, added_at
                 FROM favorites ORDER BY added_at DESC LIMIT ?1 OFFSET ?2",
            )?;
            let rows = stmt
                .query_map(params![limit, offset], |r| {
                    Ok(FavoriteRow {
                        id: r.get(0)?,
                        title: r.get(1)?,
                        media_id: r.get(2)?,
                        thumbnail: r.get(3)?,
                        added_at: r.get(4)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    // ---------------------------------------------------------------------
    // Read progress
    // ---------------------------------------------------------------------

    /// Record (or advance) the furthest page the user has reached in a gallery.
    /// Only moves the marker forward; re-reading earlier pages never regresses
    /// it. `read` flips to true once `last_page` crosses 50% of `total_pages`.
    pub fn read_progress_upsert(
        &self,
        gallery_id: i64,
        last_page: usize,
        total_pages: usize,
    ) -> AppResult<()> {
        self.with_conn(|c| {
            let now = Utc::now().to_rfc3339();
            let total = total_pages as i64;
            let page = last_page as i64;
            // A gallery counts as "read" once >= 50% of its pages have been
            // viewed. We recompute rather than trust the caller so partial /
            // out-of-order reports still resolve to the right state.
            let read = if total > 0 && page * 2 >= total { 1 } else { 0 };
            c.execute(
                "INSERT INTO read_progress (gallery_id, last_page, total_pages, read, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(gallery_id) DO UPDATE SET
                    last_page   = MAX(excluded.last_page, read_progress.last_page),
                    total_pages = excluded.total_pages,
                    read        = MAX(excluded.read, read_progress.read),
                    updated_at  = excluded.updated_at",
                params![gallery_id, page, total, read, now],
            )?;
            Ok(())
        })
    }

    /// Reset (or remove) a gallery's read progress. Used when the user wants
    /// to mark something as unread again.
    pub fn read_progress_reset(&self, gallery_id: i64) -> AppResult<()> {
        self.with_conn(|c| {
            c.execute(
                "DELETE FROM read_progress WHERE gallery_id = ?1",
                params![gallery_id],
            )?;
            Ok(())
        })
    }

    /// Lookup a single gallery's progress.
    pub fn read_progress_get(&self, gallery_id: i64) -> AppResult<Option<ReadProgressRow>> {
        self.with_conn(|c| {
            let mut stmt = c.prepare(
                "SELECT gallery_id, last_page, total_pages, read, updated_at
                 FROM read_progress WHERE gallery_id = ?1",
            )?;
            let mut rows = stmt.query_map(params![gallery_id], row_to_read_progress)?;
            match rows.next().transpose()? {
                Some(row) => Ok(Some(row)),
                None => Ok(None),
            }
        })
    }

    /// Return the set of gallery IDs the user has finished (>= 50%). Used by
    /// the frontend to badge covers in the online gallery + local library.
    pub fn read_progress_ids(&self) -> AppResult<Vec<i64>> {
        self.with_conn(|c| {
            let mut stmt = c.prepare("SELECT gallery_id FROM read_progress WHERE read = 1")?;
            let rows = stmt
                .query_map([], |r| r.get::<_, i64>(0))?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }
}

/// One row of read-progress state. `read` mirrors the SQL boolean (0/1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadProgressRow {
    pub gallery_id: i64,
    pub last_page: i64,
    pub total_pages: i64,
    pub read: bool,
    pub updated_at: String,
}

fn row_to_read_progress(row: &rusqlite::Row<'_>) -> rusqlite::Result<ReadProgressRow> {
    let read_int: i64 = row.get(3)?;
    Ok(ReadProgressRow {
        gallery_id: row.get(0)?,
        last_page: row.get(1)?,
        total_pages: row.get(2)?,
        read: read_int != 0,
        updated_at: row.get(4)?,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoriteRow {
    pub id: i64,
    pub title: String,
    pub media_id: i64,
    pub thumbnail: String,
    pub added_at: String,
}

/// Global database reference, registered by the runtime at startup so the
/// API client (which doesn't carry a `Database` handle) can still read the
/// local tag cache.
static GLOBAL_DB: once_cell::sync::OnceCell<Database> = once_cell::sync::OnceCell::new();

pub fn register_global(db: &Database) {
    let _ = GLOBAL_DB.set(db.clone());
}

fn global() -> Option<&'static Database> {
    GLOBAL_DB.get()
}

/// Free functions used by `api.rs` (read path) without needing a `Database`
/// handle, so the API client doesn't have to clone the DB around.
pub fn tags_get_by_ids(ids: &[i64]) -> AppResult<Vec<Tag>> {
    let Some(db) = global() else {
        return Ok(vec![]);
    };
    if ids.is_empty() {
        return Ok(vec![]);
    }
    db.with_conn(|c| {
        let placeholders = (0..ids.len()).map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT id, name, type, count, status FROM tags WHERE id IN ({})",
            placeholders
        );
        let mut stmt = c.prepare(&sql)?;
        let args: Vec<&dyn rusqlite::ToSql> =
            ids.iter().map(|i| i as &dyn rusqlite::ToSql).collect();
        let rows = stmt.query_map(args.as_slice(), row_to_tag)?.collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    })
}

pub fn tags_search(query: &str, limit: usize) -> AppResult<Vec<Tag>> {
    let Some(db) = global() else {
        return Ok(vec![]);
    };
    db.tags_search(query, limit)
}

pub fn tag_upsert(t: &Tag) -> AppResult<()> {
    let Some(db) = global() else {
        return Ok(());
    };
    db.tag_insert_or_update(t)
}

// ---------------------------------------------------------------------------
// Migrations
// ---------------------------------------------------------------------------

fn migrate(conn: &Connection) -> AppResult<()> {
    for stmt in MIGRATIONS {
        conn.execute_batch(stmt)?;
    }
    Ok(())
}

const MIGRATIONS: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS favorites (
        id           INTEGER PRIMARY KEY,
        title        TEXT NOT NULL,
        media_id     INTEGER NOT NULL DEFAULT 0,
        thumbnail    TEXT NOT NULL DEFAULT '',
        added_at     TEXT NOT NULL
    );",
    "CREATE TABLE IF NOT EXISTS history (
        gallery_id   INTEGER PRIMARY KEY,
        title        TEXT NOT NULL,
        media_id     INTEGER NOT NULL DEFAULT 0,
        thumbnail    TEXT NOT NULL DEFAULT '',
        visited_at   TEXT NOT NULL
    );",
    "CREATE TABLE IF NOT EXISTS tags (
        id                INTEGER PRIMARY KEY,
        name              TEXT NOT NULL,
        type              TEXT NOT NULL DEFAULT 'tag',
        count             INTEGER NOT NULL DEFAULT 0,
        status            TEXT NOT NULL DEFAULT 'default',
        online_blacklist  INTEGER NOT NULL DEFAULT 0
    );",
    "CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name);",
    "CREATE INDEX IF NOT EXISTS idx_tags_status ON tags(status);",
    "CREATE TABLE IF NOT EXISTS downloads (
        id           INTEGER PRIMARY KEY,
        title        TEXT NOT NULL,
        media_id     INTEGER NOT NULL DEFAULT 0,
        thumbnail    TEXT NOT NULL DEFAULT '',
        folder       TEXT NOT NULL,
        total_pages  INTEGER NOT NULL DEFAULT 0,
        done_pages   INTEGER NOT NULL DEFAULT 0,
        status       TEXT NOT NULL DEFAULT 'pending',
        created_at   TEXT NOT NULL,
        updated_at   TEXT NOT NULL
    );",
    "CREATE TABLE IF NOT EXISTS local_meta (
        folder       TEXT PRIMARY KEY,
        gallery_id   INTEGER NOT NULL,
        title        TEXT NOT NULL,
        media_id     INTEGER NOT NULL DEFAULT 0,
        thumbnail    TEXT NOT NULL DEFAULT '',
        num_pages    INTEGER NOT NULL DEFAULT 0,
        page_files   TEXT NOT NULL DEFAULT '[]',
        scanned_at   TEXT NOT NULL
    );",
    // Read progress: per-gallery furthest page reached + total pages known at
    // the time. `read` is 1 when the user has seen >= 50% of the gallery.
    "CREATE TABLE IF NOT EXISTS read_progress (
        gallery_id   INTEGER PRIMARY KEY,
        last_page    INTEGER NOT NULL DEFAULT 0,
        total_pages  INTEGER NOT NULL DEFAULT 0,
        read         INTEGER NOT NULL DEFAULT 0,
        updated_at   TEXT NOT NULL
    );",
    "CREATE INDEX IF NOT EXISTS idx_read_progress_read ON read_progress(read);",
];

// ---------------------------------------------------------------------------
// Public helpers (used by commands) implemented on the richer Database type
// ---------------------------------------------------------------------------

impl Database {
    pub fn history_add(
        &self,
        id: i64,
        title: &str,
        media_id: i64,
        thumbnail: &str,
    ) -> AppResult<()> {
        self.with_conn(|c| {
            let now = Utc::now().to_rfc3339();
            c.execute(
                "INSERT OR REPLACE INTO history (gallery_id, title, media_id, thumbnail, visited_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, title, media_id, thumbnail, now],
            )?;
            // Trim history to max size.
            c.execute(
                "DELETE FROM history WHERE gallery_id NOT IN (
                    SELECT gallery_id FROM history ORDER BY visited_at DESC LIMIT 500
                 );",
                [],
            )?;
            Ok(())
        })
    }

    pub fn history_list(&self, limit: u32) -> AppResult<Vec<HistoryEntry>> {
        self.with_conn(|c| {
            let mut stmt = c.prepare(
                "SELECT gallery_id, title, media_id, thumbnail, visited_at
                 FROM history ORDER BY visited_at DESC LIMIT ?1",
            )?;
            let rows = stmt
                .query_map(params![limit], |r| {
                    let ts: String = r.get(4)?;
                    Ok(HistoryEntry {
                        gallery_id: r.get(0)?,
                        title: r.get(1)?,
                        media_id: r.get(2)?,
                        thumbnail: r.get(3)?,
                        visited_at: chrono::DateTime::parse_from_rfc3339(&ts)
                            .map(|d| d.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now()),
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    pub fn history_clear(&self) -> AppResult<()> {
        self.with_conn(|c| {
            c.execute("DELETE FROM history", [])?;
            Ok(())
        })
    }

    // Tags ---------------------------------------------------------------

    pub fn tag_insert_or_update(&self, t: &Tag) -> AppResult<()> {
        self.with_conn(|c| {
            c.execute(
                "INSERT INTO tags (id, name, type, count, status, online_blacklist)
                 VALUES (?1, ?2, ?3, ?4, ?5, 0)
                 ON CONFLICT(id) DO UPDATE SET
                    name = excluded.name,
                    type = excluded.type,
                    count = excluded.count",
                params![t.id, t.name, t.tag_type.single(), t.count, status_str(t.status)],
            )?;
            Ok(())
        })
    }

    pub fn tag_set_status(&self, id: i64, status: TagStatus) -> AppResult<()> {
        self.with_conn(|c| {
            c.execute(
                "UPDATE tags SET status = ?1 WHERE id = ?2",
                params![status_str(status), id],
            )?;
            Ok(())
        })
    }

    pub fn tag_set_blacklist(&self, id: i64, blacklisted: bool) -> AppResult<()> {
        self.with_conn(|c| {
            c.execute(
                "UPDATE tags SET online_blacklist = ?1 WHERE id = ?2",
                params![if blacklisted { 1 } else { 0 }, id],
            )?;
            Ok(())
        })
    }

    pub fn tags_by_type(&self, type_filter: Option<TagType>) -> AppResult<Vec<Tag>> {
        self.with_conn(|c| {
            let mut sql = String::from(
                "SELECT id, name, type, count, status FROM tags",
            );
            let mut args: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
            if let Some(t) = type_filter {
                sql.push_str(" WHERE type = ?1");
                args.push(Box::new(t.single().to_string()));
            }
            sql.push_str(" ORDER BY count DESC LIMIT 2000");
            let mut stmt = c.prepare(&sql)?;
            let arg_refs: Vec<&dyn rusqlite::ToSql> = args.iter().map(|b| b.as_ref()).collect();
            let rows = stmt
                .query_map(arg_refs.as_slice(), row_to_tag)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    pub fn tags_all(&self) -> AppResult<Vec<Tag>> {
        self.tags_by_type(None)
    }

    pub fn tags_status(&self, status: TagStatus) -> AppResult<Vec<Tag>> {
        self.with_conn(|c| {
            let mut stmt = c.prepare(
                "SELECT id, name, type, count, status FROM tags WHERE status = ?1",
            )?;
            let rows = stmt
                .query_map(params![status_str(status)], row_to_tag)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    pub fn tags_blacklisted(&self) -> AppResult<Vec<Tag>> {
        self.with_conn(|c| {
            let mut stmt = c.prepare(
                "SELECT id, name, type, count, status FROM tags WHERE online_blacklist = 1",
            )?;
            let rows = stmt.query_map([], row_to_tag)?.collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    pub fn tags_search(&self, query: &str, limit: usize) -> AppResult<Vec<Tag>> {
        self.with_conn(|c| {
            let pattern = format!("%{}%", query.to_ascii_lowercase());
            let mut stmt = c.prepare(
                "SELECT id, name, type, count, status FROM tags
                 WHERE LOWER(name) LIKE ?1
                 ORDER BY count DESC LIMIT ?2",
            )?;
            let rows = stmt
                .query_map(params![pattern, limit as i64], row_to_tag)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    // Downloads ----------------------------------------------------------

    pub fn download_upsert(
        &self,
        id: i64,
        title: &str,
        media_id: i64,
        thumbnail: &str,
        folder: &str,
        total_pages: usize,
        done_pages: usize,
        status: &str,
    ) -> AppResult<()> {
        self.with_conn(|c| {
            let now = Utc::now().to_rfc3339();
            c.execute(
                "INSERT INTO downloads (id, title, media_id, thumbnail, folder,
                                        total_pages, done_pages, status,
                                        created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)
                 ON CONFLICT(id) DO UPDATE SET
                    title = excluded.title,
                    folder = excluded.folder,
                    total_pages = excluded.total_pages,
                    done_pages = excluded.done_pages,
                    status = excluded.status,
                    updated_at = excluded.updated_at",
                params![id, title, media_id, thumbnail, folder,
                        total_pages as i64, done_pages as i64, status, now],
            )?;
            Ok(())
        })
    }

    pub fn download_remove(&self, id: i64) -> AppResult<()> {
        self.with_conn(|c| {
            c.execute("DELETE FROM downloads WHERE id = ?1", params![id])?;
            Ok(())
        })
    }

    pub fn downloads_all(&self) -> AppResult<Vec<DownloadRow>> {
        self.with_conn(|c| {
            let mut stmt = c.prepare(
                "SELECT id, title, media_id, thumbnail, folder,
                        total_pages, done_pages, status, updated_at
                 FROM downloads ORDER BY updated_at DESC",
            )?;
            let rows = stmt
                .query_map([], |r| {
                    Ok(DownloadRow {
                        id: r.get(0)?,
                        title: r.get(1)?,
                        media_id: r.get(2)?,
                        thumbnail: r.get(3)?,
                        folder: r.get(4)?,
                        total_pages: r.get(5)?,
                        done_pages: r.get(6)?,
                        status: r.get(7)?,
                        updated_at: r.get(8)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    // Local library ------------------------------------------------------

    pub fn local_upsert(&self, g: &LocalGallery) -> AppResult<()> {
        self.with_conn(|c| {
            let page_files = serde_json::to_string(&g.page_files).unwrap_or_else(|_| "[]".into());
            c.execute(
                "INSERT INTO local_meta (folder, gallery_id, title, media_id,
                                          thumbnail, num_pages, page_files, scanned_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(folder) DO UPDATE SET
                    gallery_id = excluded.gallery_id,
                    title = excluded.title,
                    media_id = excluded.media_id,
                    thumbnail = excluded.thumbnail,
                    num_pages = excluded.num_pages,
                    page_files = excluded.page_files,
                    scanned_at = excluded.scanned_at",
                params![g.folder, g.id, g.title, g.media_id, g.thumbnail_path,
                        g.num_pages as i64, page_files, Utc::now().to_rfc3339()],
            )?;
            Ok(())
        })
    }

    pub fn local_remove(&self, folder: &str) -> AppResult<()> {
        self.with_conn(|c| {
            c.execute("DELETE FROM local_meta WHERE folder = ?1", params![folder])?;
            Ok(())
        })
    }

    /// IDs of galleries that exist on disk in the local library (downloaded).
    /// Only rows with a non-zero `gallery_id` count — folders without an id
    /// marker can't be matched against online galleries anyway.
    pub fn local_ids(&self) -> AppResult<Vec<i64>> {
        self.with_conn(|c| {
            let mut stmt =
                c.prepare("SELECT gallery_id FROM local_meta WHERE gallery_id != 0")?;
            let rows = stmt
                .query_map([], |r| r.get::<_, i64>(0))?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    pub fn local_all(&self) -> AppResult<Vec<LocalGallery>> {
        self.with_conn(|c| {
            let mut stmt = c.prepare(
                "SELECT folder, gallery_id, title, media_id, thumbnail,
                        num_pages, page_files
                 FROM local_meta ORDER BY title COLLATE NOCASE ASC",
            )?;
            let rows = stmt
                .query_map([], |r| {
                    let page_files_json: String = r.get(6)?;
                    let page_files: Vec<String> =
                        serde_json::from_str(&page_files_json).unwrap_or_default();
                    Ok(LocalGallery {
                        folder: r.get(0)?,
                        id: r.get(1)?,
                        title: r.get(2)?,
                        media_id: r.get(3)?,
                        thumbnail_path: r.get(4)?,
                        num_pages: r.get::<_, i64>(5)? as usize,
                        page_files,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRow {
    pub id: i64,
    pub title: String,
    pub media_id: i64,
    pub thumbnail: String,
    pub folder: String,
    pub total_pages: usize,
    pub done_pages: usize,
    pub status: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Row helpers
// ---------------------------------------------------------------------------

fn row_to_tag(row: &rusqlite::Row<'_>) -> rusqlite::Result<Tag> {
    let type_str: String = row.get(2)?;
    let status_str: String = row.get(4)?;
    Ok(Tag {
        id: row.get(0)?,
        name: row.get(1)?,
        tag_type: TagType::from_name(&type_str),
        count: row.get(3)?,
        status: parse_status(&status_str),
    })
}

fn status_str(s: TagStatus) -> &'static str {
    match s {
        TagStatus::Default => "default",
        TagStatus::Accepted => "accepted",
        TagStatus::Avoided => "avoided",
    }
}

fn parse_status(s: &str) -> TagStatus {
    match s {
        "accepted" => TagStatus::Accepted,
        "avoided" => TagStatus::Avoided,
        _ => TagStatus::Default,
    }
}
