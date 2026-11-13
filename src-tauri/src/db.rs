use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension, Transaction};
use tauri::{AppHandle, Manager};

use crate::models::{Entry, Stats};

/// What needs to be written back to the system clipboard for a given entry.
pub enum CopyPayload {
    Text(String),
    Image(PathBuf),
}

pub struct Db {
    conn: Mutex<Connection>,
    images_dir: PathBuf,
}

impl Db {
    pub fn open(app: &AppHandle) -> Result<Self, String> {
        let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

        let images_dir = data_dir.join("images");
        fs::create_dir_all(&images_dir).map_err(|e| e.to_string())?;

        let conn = Connection::open(data_dir.join("cliphistory.db")).map_err(db_err)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             CREATE TABLE IF NOT EXISTS entries (
                 id           INTEGER PRIMARY KEY AUTOINCREMENT,
                 kind         TEXT    NOT NULL CHECK (kind IN ('text', 'image')),
                 content      TEXT,
                 image_path   TEXT,
                 thumb        BLOB,
                 img_w        INTEGER,
                 img_h        INTEGER,
                 hash         TEXT    NOT NULL,
                 pinned       INTEGER NOT NULL DEFAULT 0,
                 created_at   INTEGER NOT NULL,
                 last_used_at INTEGER NOT NULL
             );
             CREATE UNIQUE INDEX IF NOT EXISTS idx_entries_hash ON entries(hash);
             CREATE INDEX IF NOT EXISTS idx_entries_recency
                 ON entries(pinned DESC, last_used_at DESC);",
        )
        .map_err(db_err)?;

        let db = Self {
            conn: Mutex::new(conn),
            images_dir,
        };

        if let Err(error) = db.remove_orphan_images() {
            eprintln!("cliphistory: failed to clean up orphan images: {error}");
        }

        Ok(db)
    }

    /// Removes image files that are not referenced by any entry (e.g. left behind by a crash).
    fn remove_orphan_images(&self) -> Result<(), String> {
        let known: HashSet<String> = {
            let conn = self.lock()?;
            let mut stmt = conn
                .prepare("SELECT image_path FROM entries WHERE image_path IS NOT NULL")
                .map_err(db_err)?;
            let paths = stmt
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(db_err)?
                .collect::<rusqlite::Result<HashSet<_>>>()
                .map_err(db_err)?;
            paths
        };

        let Ok(files) = fs::read_dir(&self.images_dir) else {
            return Ok(());
        };

        for file in files.flatten() {
            let name = file.file_name().to_string_lossy().to_string();
            if !known.contains(&name) {
                let _ = fs::remove_file(file.path());
            }
        }

        Ok(())
    }

    fn lock(&self) -> Result<MutexGuard<'_, Connection>, String> {
        self.conn
            .lock()
            .map_err(|_| "Database unavailable.".to_string())
    }

    /// Inserts a text entry, or bumps an existing identical entry to the top.
    ///
    /// Returns `true` when a new entry was created.
    pub fn insert_text(&self, text: &str, hash: &str, max_items: usize) -> Result<bool, String> {
        let mut conn = self.lock()?;
        let now = now_ms();

        let tx = conn.transaction().map_err(db_err)?;

        let existing: Option<i64> = tx
            .query_row("SELECT id FROM entries WHERE hash = ?1", [hash], |row| {
                row.get(0)
            })
            .optional()
            .map_err(db_err)?;

        if let Some(id) = existing {
            tx.execute(
                "UPDATE entries SET last_used_at = ?1 WHERE id = ?2",
                params![now, id],
            )
            .map_err(db_err)?;
            tx.commit().map_err(db_err)?;
            return Ok(false);
        }

        tx.execute(
            "INSERT INTO entries
                (kind, content, image_path, thumb, img_w, img_h, hash, pinned, created_at, last_used_at)
             VALUES ('text', ?1, NULL, NULL, NULL, NULL, ?2, 0, ?3, ?3)",
            params![text, hash, now],
        )
        .map_err(db_err)?;

        let stale = prune(&tx, max_items).map_err(db_err)?;
        tx.commit().map_err(db_err)?;
        remove_files(&self.images_dir, &stale);

        Ok(true)
    }

    /// Saves an RGBA image to disk and inserts its metadata, or bumps an existing identical one.
    ///
    /// Returns `true` when a new entry was created.
    pub fn insert_image(
        &self,
        rgba: &[u8],
        width: u32,
        height: u32,
        hash: &str,
        max_items: usize,
    ) -> Result<bool, String> {
        if width == 0 || height == 0 || width > 8192 || height > 8192 {
            // Ignore pathological images instead of filling memory and the disk.
            return Ok(false);
        }

        let now = now_ms();

        // Check for duplicates and release the lock before writing the image: encoding a
        // 4K screenshot can take a while and should not block the UI.
        {
            let conn = self.lock()?;
            let existing: Option<i64> = conn
                .query_row("SELECT id FROM entries WHERE hash = ?1", [hash], |row| {
                    row.get(0)
                })
                .optional()
                .map_err(db_err)?;

            if let Some(id) = existing {
                conn.execute(
                    "UPDATE entries SET last_used_at = ?1 WHERE id = ?2",
                    params![now, id],
                )
                .map_err(db_err)?;
                return Ok(false);
            }
        }

        let file_name = format!("{}-{}.png", now, &hash[..hash.len().min(16)]);
        let file_path = self.images_dir.join(&file_name);
        write_png(&file_path, rgba, width, height)?;

        let mut conn = self.lock()?;
        let tx = conn.transaction().map_err(db_err)?;

        let insert = tx.execute(
            "INSERT INTO entries
                (kind, content, image_path, img_w, img_h, hash, pinned, created_at, last_used_at)
             VALUES ('image', NULL, ?1, ?2, ?3, ?4, 0, ?5, ?5)",
            params![file_name, width, height, hash, now],
        );

        if let Err(err) = insert {
            let _ = fs::remove_file(&file_path);
            return Err(db_err(err));
        }

        let stale = prune(&tx, max_items).map_err(db_err)?;
        tx.commit().map_err(db_err)?;
        remove_files(&self.images_dir, &stale);

        Ok(true)
    }

    /// Lists entries, newest first, with pinned entries on top.
    ///
    /// Text is truncated to a display-friendly preview; the full content stays in the database.
    pub fn list(&self, search: Option<&str>) -> Result<Vec<Entry>, String> {
        let conn = self.lock()?;
        let term = search.map(str::trim).filter(|value| !value.is_empty());

        let base = "SELECT id, kind, substr(content, 1, 2000), img_w, img_h, pinned, created_at
                    FROM entries";

        let mut entries: Vec<Entry> = match term {
            Some(query) => {
                let sql = format!(
                    "{base} WHERE content LIKE ?1 ESCAPE '\\'
                     ORDER BY pinned DESC, last_used_at DESC, id DESC LIMIT 500"
                );
                let mut stmt = conn.prepare(&sql).map_err(db_err)?;
                let rows = stmt
                    .query_map(params![like_pattern(query)], |row| map_entry(row))
                    .map_err(db_err)?;
                rows.collect::<rusqlite::Result<Vec<_>>>()
                    .map_err(db_err)?
            }
            None => {
                let sql = format!(
                    "{base} ORDER BY pinned DESC, last_used_at DESC, id DESC LIMIT 500"
                );
                let mut stmt = conn.prepare(&sql).map_err(db_err)?;
                let rows = stmt
                    .query_map([], |row| map_entry(row))
                    .map_err(db_err)?;
                rows.collect::<rusqlite::Result<Vec<_>>>()
                    .map_err(db_err)?
            }
        };

        // Empty text entries should never exist, but keep the list clean if they do.
        entries.retain(|entry| entry.kind == "image" || entry.text.is_some());
        Ok(entries)
    }

    pub fn payload(&self, id: i64) -> Result<Option<CopyPayload>, String> {
        let conn = self.lock()?;
        let row: Option<(String, Option<String>, Option<String>)> = conn
            .query_row(
                "SELECT kind, content, image_path FROM entries WHERE id = ?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(db_err)?;

        Ok(match row {
            Some((kind, _, Some(path))) if kind == "image" => {
                Some(CopyPayload::Image(self.images_dir.join(path)))
            }
            Some((_, Some(text), _)) => Some(CopyPayload::Text(text)),
            _ => None,
        })
    }

    pub fn touch(&self, id: i64) -> Result<(), String> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE entries SET last_used_at = ?1 WHERE id = ?2",
            params![now_ms(), id],
        )
        .map_err(db_err)?;
        Ok(())
    }

    pub fn toggle_pin(&self, id: i64) -> Result<bool, String> {
        let conn = self.lock()?;
        let changed = conn
            .execute(
                "UPDATE entries SET pinned = CASE pinned WHEN 0 THEN 1 ELSE 0 END WHERE id = ?1",
                [id],
            )
            .map_err(db_err)?;

        if changed == 0 {
            return Err("Item not found.".to_string());
        }

        let pinned: i64 = conn
            .query_row("SELECT pinned FROM entries WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .map_err(db_err)?;
        Ok(pinned != 0)
    }

    pub fn delete(&self, id: i64) -> Result<(), String> {
        let conn = self.lock()?;
        let image_path: Option<Option<String>> = conn
            .query_row("SELECT image_path FROM entries WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .optional()
            .map_err(db_err)?;

        conn.execute("DELETE FROM entries WHERE id = ?1", [id])
            .map_err(db_err)?;
        drop(conn);

        if let Some(Some(path)) = image_path {
            let _ = fs::remove_file(self.images_dir.join(path));
        }
        Ok(())
    }

    /// Deletes every unpinned entry. Pinned entries are preserved.
    pub fn clear_unpinned(&self) -> Result<usize, String> {
        let conn = self.lock()?;

        let mut stmt = conn
            .prepare("SELECT image_path FROM entries WHERE pinned = 0")
            .map_err(db_err)?;
        let files = stmt
            .query_map([], |row| row.get::<_, Option<String>>(0))
            .map_err(db_err)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(db_err)?;
        drop(stmt);

        let removed = conn
            .execute("DELETE FROM entries WHERE pinned = 0", [])
            .map_err(db_err)?;
        drop(conn);

        remove_files(
            &self.images_dir,
            &files.into_iter().flatten().collect::<Vec<_>>(),
        );
        Ok(removed)
    }

    /// Applies the retention limit, deleting the oldest unpinned entries above it.
    pub fn prune_now(&self, max_items: usize) -> Result<(), String> {
        let mut conn = self.lock()?;
        let tx = conn.transaction().map_err(db_err)?;
        let stale = prune(&tx, max_items).map_err(db_err)?;
        tx.commit().map_err(db_err)?;
        drop(conn);
        remove_files(&self.images_dir, &stale);
        Ok(())
    }

    pub fn stats(&self) -> Result<Stats, String> {
        let conn = self.lock()?;
        let (total, pinned, images): (i64, i64, i64) = conn
            .query_row(
                "SELECT COUNT(*),
                        COALESCE(SUM(CASE WHEN pinned = 1 THEN 1 ELSE 0 END), 0),
                        COALESCE(SUM(CASE WHEN kind = 'image' THEN 1 ELSE 0 END), 0)
                 FROM entries",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(db_err)?;

        Ok(Stats {
            total: total as usize,
            pinned: pinned as usize,
            images: images as usize,
        })
    }
}

fn db_err(error: rusqlite::Error) -> String {
    error.to_string()
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

fn like_pattern(query: &str) -> String {
    let escaped = query
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{escaped}%")
}

fn map_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<Entry> {
    let width: Option<i64> = row.get(3)?;
    let height: Option<i64> = row.get(4)?;

    Ok(Entry {
        id: row.get(0)?,
        kind: row.get(1)?,
        text: row.get(2)?,
        width: width.map(|value| value as u32),
        height: height.map(|value| value as u32),
        pinned: row.get::<_, i64>(5)? != 0,
        created_at: row.get(6)?,
    })
}

/// Deletes unpinned entries beyond `max_items` (pinned entries always survive).
fn prune(tx: &Transaction<'_>, max_items: usize) -> rusqlite::Result<Vec<String>> {
    let budget = max_items.max(1) as i64;

    let mut stmt = tx.prepare(
        "SELECT image_path FROM entries
         WHERE pinned = 0 AND id NOT IN (
             SELECT id FROM entries
             WHERE pinned = 0
             ORDER BY last_used_at DESC, id DESC
             LIMIT ?1
         )",
    )?;
    let stale = stmt
        .query_map([budget], |row| row.get::<_, Option<String>>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    drop(stmt);

    if !stale.is_empty() {
        tx.execute(
            "DELETE FROM entries
             WHERE pinned = 0 AND id NOT IN (
                 SELECT id FROM entries
                 WHERE pinned = 0
                 ORDER BY last_used_at DESC, id DESC
                 LIMIT ?1
             )",
            [budget],
        )?;
    }

    Ok(stale.into_iter().flatten().collect())
}

fn remove_files(images_dir: &Path, files: &[String]) {
    for file in files {
        let _ = fs::remove_file(images_dir.join(file));
    }
}

fn write_png(path: &Path, rgba: &[u8], width: u32, height: u32) -> Result<(), String> {
    use image::ImageEncoder;

    let file = fs::File::create(path).map_err(|e| e.to_string())?;
    let writer = std::io::BufWriter::new(file);
    image::codecs::png::PngEncoder::new(writer)
        .write_image(rgba, width, height, image::ExtendedColorType::Rgba8)
        .map_err(|e| e.to_string())
}
