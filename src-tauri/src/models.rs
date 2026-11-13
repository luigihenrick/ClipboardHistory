use serde::{Deserialize, Serialize};

pub const DEFAULT_SHORTCUT: &str = "CommandOrControl+Shift+V";
pub const DEFAULT_LANGUAGE: &str = "en";
pub const DEFAULT_MAX_ITEMS: usize = 100;
pub const MIN_MAX_ITEMS: usize = 10;
pub const MAX_MAX_ITEMS: usize = 2000;

pub fn normalize_language(value: &str) -> String {
    match value.trim() {
        "pt-BR" | "pt_BR" | "pt" => "pt-BR".to_string(),
        _ => DEFAULT_LANGUAGE.to_string(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// Global shortcut accelerator, e.g. `CommandOrControl+Shift+V`.
    pub shortcut: String,
    /// Maximum number of unpinned entries kept in the database.
    pub max_items: usize,
    /// Whether the app should be registered to run at login.
    pub start_at_login: bool,
    /// Whether the history popup hides itself when it loses focus.
    pub hide_on_blur: bool,
    /// Whether the app should simulate Ctrl/Cmd+V after copying an entry.
    pub auto_paste: bool,
    /// UI and tray language: `en` or `pt-BR`.
    pub language: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            shortcut: DEFAULT_SHORTCUT.to_string(),
            max_items: DEFAULT_MAX_ITEMS,
            start_at_login: false,
            hide_on_blur: true,
            auto_paste: true,
            language: DEFAULT_LANGUAGE.to_string(),
        }
    }
}

impl Settings {
    /// Clamps values to sane ranges. Accepts anything coming from disk or the frontend.
    pub fn sanitize(mut self) -> Self {
        let shortcut = self.shortcut.trim().to_string();
        self.shortcut = if shortcut.is_empty() {
            DEFAULT_SHORTCUT.to_string()
        } else {
            shortcut
        };
        self.max_items = self.max_items.clamp(MIN_MAX_ITEMS, MAX_MAX_ITEMS);
        self.language = normalize_language(&self.language);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_language_is_english() {
        assert_eq!(Settings::default().language, "en");
    }

    #[test]
    fn normalizes_language_values() {
        assert_eq!(normalize_language("en"), "en");
        assert_eq!(normalize_language("pt"), "pt-BR");
        assert_eq!(normalize_language("pt_BR"), "pt-BR");
        assert_eq!(normalize_language("pt-BR"), "pt-BR");
        assert_eq!(normalize_language("fr"), "en");
        assert_eq!(normalize_language(""), "en");
    }

    #[test]
    fn old_settings_without_language_default_to_english() {
        let json = r#"{
            "shortcut": "CommandOrControl+Shift+V",
            "maxItems": 50,
            "startAtLogin": false,
            "hideOnBlur": true,
            "autoPaste": true
        }"#;

        let settings: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.language, "en");
        assert_eq!(settings.max_items, 50);
    }

    #[test]
    fn sanitize_normalizes_invalid_language() {
        let settings = Settings {
            language: "fr".to_string(),
            ..Settings::default()
        };

        assert_eq!(settings.sanitize().language, "en");
    }
}


#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: i64,
    /// `"text"` or `"image"`.
    pub kind: String,
    /// Text preview (may be truncated) or `None` for images.
    pub text: Option<String>,
    /// PNG thumbnail as a data URL, for images only.
    pub thumbnail: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub pinned: bool,
    /// Creation time in milliseconds since the Unix epoch.
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub total: usize,
    pub pinned: usize,
    pub images: usize,
}
