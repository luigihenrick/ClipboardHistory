use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use tauri::{AppHandle, Manager};

use crate::models::{Entry, Stats};

/// Enough text for the two-line preview; the full clipboard text stays in SQLite.
const PREVIEW_CHARS: usize = 300;
/// Small enough to keep the popup payload light while remaining legible on HiDPI screens.
const THUMBNAIL_MAX: u32 = 64;
const THUMBNAIL_QUALITY: u8 = 72;
/// Only images visible in the popup need previews; backfill recent ones at startup.
const BACKFILL_LIMIT: usize = 100;

/// All entries follow the normal most-recently-used order. Pinned entries are protected from
/// deletion/pruning, but they are not forced to the top; copying any entry moves it to the top.
const LIST_ORDER: &str = "ORDER BY last_used_at DESC, id DESC";

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

        let thumbnail = make_thumbnail(rgba, width, height)?;
        let file_name = format!("{}-{}.png", now, &hash[..hash.len().min(16)]);
        let file_path = self.images_dir.join(&file_name);
        write_png(&file_path, rgba, width, height)?;

        let mut conn = self.lock()?;
        let tx = conn.transaction().map_err(db_err)?;

        let insert = tx.execute(
            "INSERT INTO entries
                (kind, content, image_path, thumb, img_w, img_h, hash, pinned, created_at, last_used_at)
             VALUES ('image', NULL, ?1, ?2, ?3, ?4, ?5, 0, ?6, ?6)",
            params![file_name, thumbnail, width, height, hash, now],
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

    /// Generates previews for image entries that predate thumbnail support or whose stored
    /// thumbnails are too large. Runs in the background at startup so it never blocks the UI.
    pub fn backfill_thumbnails(&self) -> Result<usize, String> {
        let rows: Vec<(i64, String)> = {
            let conn = self.lock()?;
            let mut stmt = conn
                .prepare(
                    "SELECT id, image_path FROM entries
                     WHERE kind = 'image' AND image_path IS NOT NULL
                       AND (
                         thumb IS NULL
                         OR substr(thumb, 1, 4) = X'89504E47'
                         OR length(thumb) > 8192
                       )
                     ORDER BY last_used_at DESC, id DESC
                     LIMIT ?1",
                )
                .map_err(db_err)?;

            let collected = stmt
                .query_map([BACKFILL_LIMIT as i64], |row| {
                    Ok((row.get(0)?, row.get(1)?))
                })
                .map_err(db_err)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(db_err)?;

            collected
        };
        let mut updated = 0;
        for (id, file_name) in rows {
            let file_path = self.images_dir.join(&file_name);
            let Ok(bytes) = fs::read(&file_path) else {
                continue;
            };
            let Ok(image) = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
            else {
                continue;
            };

            let rgba = image.to_rgba8();
            let (width, height) = rgba.dimensions();
            let Ok(thumbnail) = make_thumbnail(rgba.as_raw(), width, height) else {
                continue;
            };

            let conn = self.lock()?;
            conn.execute(
                "UPDATE entries SET thumb = ?1 WHERE id = ?2",
                params![thumbnail, id],
            )
            .map_err(db_err)?;
            updated += 1;
        }

        Ok(updated)
    }

    /// Lists every retained entry, newest first.
    ///
    /// The database already enforces the configured retention limit, so applying a fixed display
    /// limit here would silently hide entries when the user increases that setting.
    /// Text is truncated to a display-friendly preview; the full content stays in the database.
    pub fn list(&self, search: Option<&str>) -> Result<Vec<Entry>, String> {
        let conn = self.lock()?;
        let term = search.map(str::trim).filter(|value| !value.is_empty());

        let base = format!(
            "SELECT id, kind, substr(content, 1, {PREVIEW_CHARS}), thumb, img_w, img_h, pinned, created_at
             FROM entries"
        );

        let mut entries: Vec<Entry> = match term {
            Some(query) => {
                let sql = format!("{base} WHERE content LIKE ?1 ESCAPE '\\' {LIST_ORDER}");
                let mut stmt = conn.prepare(&sql).map_err(db_err)?;
                let rows = stmt
                    .query_map(params![like_pattern(query)], |row| map_entry(row))
                    .map_err(db_err)?;
                rows.collect::<rusqlite::Result<Vec<_>>>().map_err(db_err)?
            }
            None => {
                let sql = format!("{base} {LIST_ORDER}");
                let mut stmt = conn.prepare(&sql).map_err(db_err)?;
                let rows = stmt.query_map([], |row| map_entry(row)).map_err(db_err)?;
                rows.collect::<rusqlite::Result<Vec<_>>>().map_err(db_err)?
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
            .query_row(
                "SELECT image_path FROM entries WHERE id = ?1",
                [id],
                |row| row.get(0),
            )
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
    let thumbnail: Option<Vec<u8>> = row.get(3)?;
    let width: Option<i64> = row.get(4)?;
    let height: Option<i64> = row.get(5)?;

    Ok(Entry {
        id: row.get(0)?,
        kind: row.get(1)?,
        text: row.get(2)?,
        thumbnail: thumbnail.map(thumbnail_data_url),
        width: width.map(|value| value as u32),
        height: height.map(|value| value as u32),
        pinned: row.get::<_, i64>(6)? != 0,
        created_at: row.get(7)?,
    })
}

fn thumbnail_data_url(bytes: Vec<u8>) -> String {
    let mime = if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        "image/png"
    } else {
        "image/jpeg"
    };

    format!("data:{mime};base64,{}", BASE64.encode(bytes))
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

fn make_thumbnail(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, String> {
    let source = image::RgbaImage::from_raw(width, height, rgba.to_vec())
        .ok_or_else(|| "Invalid image.".to_string())?;

    let longest = width.max(height).max(1);
    let thumb_width = ((width as u64 * THUMBNAIL_MAX as u64) / longest as u64).max(1) as u32;
    let thumb_height = ((height as u64 * THUMBNAIL_MAX as u64) / longest as u64).max(1) as u32;

    let thumbnail = image::imageops::thumbnail(&source, thumb_width, thumb_height);
    let rgb = image::DynamicImage::ImageRgba8(thumbnail).to_rgb8();
    let mut buffer = Vec::new();
    let mut encoder =
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, THUMBNAIL_QUALITY);
    encoder
        .encode(
            rgb.as_raw(),
            thumb_width,
            thumb_height,
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| e.to_string())?;

    Ok(buffer)
}

fn write_png(path: &Path, rgba: &[u8], width: u32, height: u32) -> Result<(), String> {
    use image::ImageEncoder;

    let file = fs::File::create(path).map_err(|e| e.to_string())?;
    let writer = std::io::BufWriter::new(file);
    image::codecs::png::PngEncoder::new(writer)
        .write_image(rgba, width, height, image::ExtendedColorType::Rgba8)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::{Db, LIST_ORDER};
    use rusqlite::Connection;
    use std::path::PathBuf;
    use std::sync::Mutex;

    fn ordered_ids(conn: &Connection) -> Vec<i64> {
        let mut stmt = conn
            .prepare(&format!("SELECT id FROM entries {LIST_ORDER}"))
            .unwrap();

        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap()
    }

    #[test]
    fn pinned_entries_follow_the_normal_recency_order() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE entries (
                 id           INTEGER PRIMARY KEY,
                 pinned       INTEGER NOT NULL,
                 created_at   INTEGER NOT NULL,
                 last_used_at INTEGER NOT NULL
             );
             INSERT INTO entries VALUES (1, 1, 100, 100);
             INSERT INTO entries VALUES (2, 1, 200, 50);
             INSERT INTO entries VALUES (3, 0, 300, 500);
             INSERT INTO entries VALUES (4, 0, 400, 400);",
        )
        .unwrap();

        // Pinned items do not jump ahead of more recently used items.
        assert_eq!(ordered_ids(&conn), vec![3, 4, 1, 2]);

        // Copying a pinned item moves it to the top, just like any other item.
        conn.execute("UPDATE entries SET last_used_at = 999 WHERE id = 1", [])
            .unwrap();
        assert_eq!(ordered_ids(&conn), vec![1, 3, 4, 2]);
    }

    fn db_with_text_entries(count: usize) -> Db {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE entries (
                 id           INTEGER PRIMARY KEY AUTOINCREMENT,
                 kind         TEXT    NOT NULL,
                 content      TEXT,
                 image_path   TEXT,
                 thumb        BLOB,
                 img_w        INTEGER,
                 img_h        INTEGER,
                 hash         TEXT    NOT NULL,
                 pinned       INTEGER NOT NULL DEFAULT 0,
                 created_at   INTEGER NOT NULL,
                 last_used_at INTEGER NOT NULL
             );",
        )
        .unwrap();

        for index in 0..count {
            conn.execute(
                "INSERT INTO entries
                    (kind, content, hash, pinned, created_at, last_used_at)
                 VALUES ('text', ?1, ?2, 0, ?3, ?3)",
                rusqlite::params![
                    format!("entry-{index}"),
                    format!("hash-{index}"),
                    index as i64
                ],
            )
            .unwrap();
        }

        Db {
            conn: Mutex::new(conn),
            images_dir: PathBuf::from("/tmp"),
        }
    }

    #[test]
    fn lists_every_retained_entry_above_one_hundred() {
        let db = db_with_text_entries(150);

        let entries = db.list(None).unwrap();
        assert_eq!(entries.len(), 150);
        assert!(entries.iter().all(|entry| entry.kind == "text"));
    }

    #[test]
    fn applies_a_configured_retention_limit_above_one_hundred() {
        let db = db_with_text_entries(150);

        db.prune_now(120).unwrap();
        assert_eq!(db.list(None).unwrap().len(), 120);
    }
}
