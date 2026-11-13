use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::thread;
use std::time::Duration;

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::AppState;

const POLL_INTERVAL: Duration = Duration::from_millis(700);

/// Watches the system clipboard in a background thread.
///
/// The clipboard plugins are blocking and must not run on the UI thread, hence the dedicated
/// thread. Content is deduplicated by hash; copying the same text/image again only bumps the
/// existing entry to the top instead of creating a duplicate.
pub fn spawn(app: AppHandle) {
    thread::spawn(move || {
        let mut initialized = false;
        let mut last_text_hash: Option<String> = None;
        let mut last_image_fingerprint: Option<u64> = None;

        loop {
            thread::sleep(POLL_INTERVAL);

            let max_items = app
                .try_state::<AppState>()
                .and_then(|state| state.settings.lock().ok().map(|s| s.max_items))
                .unwrap_or(100);

            let mut captured = false;

            match app.clipboard().read_text() {
                Ok(text) if !text.trim().is_empty() => {
                    let hash = hash_text(&text);
                    if last_text_hash.as_deref() == Some(hash.as_str()) {
                        continue;
                    }

                    last_text_hash = Some(hash.clone());
                    last_image_fingerprint = None;

                    if !initialized {
                        // Ignore whatever was already in the clipboard before launch.
                        initialized = true;
                        continue;
                    }

                    if let Some(state) = app.try_state::<AppState>() {
                        match state.db.insert_text(&text, &hash, max_items) {
                            Ok(inserted) => captured = inserted,
                            Err(error) => eprintln!("cliphistory: failed to save text: {error}"),
                        }
                    }
                }
                _ => {
                    let Ok(image) = app.clipboard().read_image() else {
                        // Nothing usable in the clipboard (typically empty). From now on, any
                        // content that shows up is new and should be captured.
                        initialized = true;
                        continue;
                    };

                    let width = image.width();
                    let height = image.height();
                    // Borrow the RGBA buffer instead of cloning it: a 4K image is ~33 MB and
                    // this code runs on every poll while an image sits in the clipboard.
                    let rgba = image.rgba();
                    let fingerprint = fingerprint(rgba, width, height);

                    if last_image_fingerprint == Some(fingerprint) {
                        continue;
                    }

                    last_image_fingerprint = Some(fingerprint);
                    last_text_hash = None;

                    if !initialized {
                        initialized = true;
                        continue;
                    }

                    let hash = hash_image(rgba, width, height);
                    if let Some(state) = app.try_state::<AppState>() {
                        match state
                            .db
                            .insert_image(rgba, width, height, &hash, max_items)
                        {
                            Ok(inserted) => captured = inserted,
                            Err(error) => eprintln!("cliphistory: failed to save image: {error}"),
                        }
                    }
                }
            }

            if captured {
                let _ = app.emit("history://changed", ());
            }
        }
    });
}

fn hash_text(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"text:");
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn hash_image(rgba: &[u8], width: u32, height: u32) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"image:");
    hasher.update(width.to_le_bytes());
    hasher.update(height.to_le_bytes());
    hasher.update(rgba);
    format!("{:x}", hasher.finalize())
}

/// Cheap fingerprint used to detect whether the clipboard image changed.
///
/// Hashing a 4K image on every poll would burn CPU, so only a sample of the pixels plus the
/// dimensions is hashed. The full SHA-256 is only computed when the fingerprint changes.
fn fingerprint(rgba: &[u8], width: u32, height: u32) -> u64 {
    let mut hasher = DefaultHasher::new();
    width.hash(&mut hasher);
    height.hash(&mut hasher);
    rgba.len().hash(&mut hasher);

    let step = (rgba.len() / 8192).max(1);
    let mut index = 0;
    while index < rgba.len() {
        hasher.write_u8(rgba[index]);
        index += step;
    }

    hasher.finish()
}
