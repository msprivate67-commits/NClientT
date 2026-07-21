//! Persistent user configuration.
//!
//! Mirrors the role of NClientV3's `SharedPreferences("Settings")` +
//! `AuthStore`. Stored as JSON in the app data directory so it is trivial to
//! inspect / back up.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use serde::{Deserialize, Serialize};

use crate::error::AppResult;

/// App version, used for the User-Agent header (mirrors NClientV3's
/// `ApiAuthInterceptor` which sends `NClient/<version>`).
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default mirror (the original nhentai host, like `Utility.ORIGINAL_URL`).
pub const DEFAULT_MIRROR: &str = "nhentai.net";

/// Default User-Agent. Uses the exact same format as NClientV3's
/// `ApiAuthInterceptor` so the server sees the same client identity.
/// NClientV3 sends: `NClient/<version> (https://github.com/maxwai/NClientV3)`
pub const DEFAULT_UA: &str = concat!(
    "NClient/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/maxwai/NClientV3)"
);

/// Title type used to display gallery titles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TitleType {
    Pretty,
    English,
    Japanese,
    #[default]
    Auto,
}

/// Sort orders, mirrors `SortType` (only the URL additions matter here).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SortType {
    #[default]
    RecentAllTime,
    PopularAllTime,
    PopularWeek,
    PopularDay,
    PopularMonth,
}

impl SortType {
    /// Mirrors `SortType#getUrlAddition`.
    pub fn url_addition(self) -> Option<&'static str> {
        match self {
            SortType::RecentAllTime => None,
            SortType::PopularAllTime => Some("popular"),
            SortType::PopularWeek => Some("popular-week"),
            SortType::PopularDay => Some("popular-today"),
            SortType::PopularMonth => Some("popular-month"),
        }
    }
}

/// Language filter, mirrors `Language` (the `All`/`Unknown` distinction in the
/// original collapses to `All` here for simplicity).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    All,
    #[default]
    English,
    Japanese,
    Chinese,
}

/// Proxy type for HTTP requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProxyType {
    #[default]
    None,
    Http,
    Socks5,
}

/// Data usage policy, mirrors `Global#DataUsageType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DataUsageType {
    None,
    Thumbnail,
    #[default]
    Full,
}

/// Local gallery sort order, mirrors `LocalSortType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LocalSortType {
    #[default]
    TitleAsc,
    TitleDesc,
    DateAsc,
    DateDesc,
    PagesAsc,
    PagesDesc,
}

/// Authentication credentials, mirrors `AuthStore` + `AuthCredentials`.
/// `api_key` corresponds to the original `AuthCredentials.Type.API_KEY`
/// and is sent as `Authorization: Key <secret>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AuthCredentials {
    pub api_key: String,
    /// Cached validity flag, refreshed whenever the API returns 401/403.
    pub valid: bool,
}

impl AuthCredentials {
    pub fn authorization_header(&self) -> Option<String> {
        let trimmed = self.api_key.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(format!("Key {}", trimmed))
        }
    }

    pub fn has_credentials(&self) -> bool {
        !self.api_key.trim().is_empty()
    }
}

/// All persisted settings. Mirrors the union of NClientV3's preference keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    // --- site / network -----------------------------------------------------
    /// nhentai mirror host, e.g. `nhentai.net`. Mirrors `Utility.getHost()`.
    pub mirror: String,
    /// Override User-Agent. Empty => use [`DEFAULT_UA`].
    pub user_agent: String,
    /// HTTP connect / read timeout in seconds.
    pub request_timeout_secs: u64,
    /// API key auth (sent as `Authorization: Key <key>`).
    pub auth: AuthCredentials,

    // --- proxy ---------------------------------------------------------------
    pub proxy_type: ProxyType,
    pub proxy_host: String,
    pub proxy_port: u16,
    pub proxy_username: String,
    pub proxy_password: String,

    // --- browsing -----------------------------------------------------------
    pub sort_type: SortType,
    pub only_language: Language,
    pub title_type: TitleType,
    pub exact_tag_match: bool,
    pub remove_avoided_galleries: bool,
    pub show_titles: bool,

    // --- display ------------------------------------------------------------
    pub column_count: u32,
    /// Number of page-thumbnail columns in gallery detail.
    /// 0 = auto (responsive CSS grid). 2-10 = fixed count.
    pub page_thumbnail_columns: u32,
    pub use_rtl: bool,
    pub default_zoom_pct: u32,
    pub button_change_page: bool,

    // --- data usage ---------------------------------------------------------
    pub usage_wifi: DataUsageType,
    pub usage_mobile: DataUsageType,

    // --- history / favorites limits ----------------------------------------
    pub keep_history: bool,
    pub max_history: u32,
    pub favorite_limit: u32,

    // --- downloads ----------------------------------------------------------
    pub download_dir: PathBuf,
    pub parallel_downloads: u32,
    pub parallel_pages: u32,

    // --- security -----------------------------------------------------------
    pub lock_screen: bool,
    pub pin: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mirror: DEFAULT_MIRROR.to_string(),
            user_agent: String::new(),
            request_timeout_secs: 30,
            auth: AuthCredentials::default(),
            proxy_type: ProxyType::None,
            proxy_host: String::new(),
            proxy_port: 1080,
            proxy_username: String::new(),
            proxy_password: String::new(),
            sort_type: SortType::RecentAllTime,
            only_language: Language::All,
            title_type: TitleType::Auto,
            exact_tag_match: false,
            remove_avoided_galleries: true,
            show_titles: true,
            column_count: 3,
            page_thumbnail_columns: 0,
            use_rtl: false,
            default_zoom_pct: 100,
            button_change_page: true,
            usage_wifi: DataUsageType::Full,
            usage_mobile: DataUsageType::Thumbnail,
            keep_history: true,
            max_history: 100,
            favorite_limit: 100,
            download_dir: PathBuf::new(), // filled in by ConfigStore::load_or_init
            parallel_downloads: 3,
            parallel_pages: 8,
            lock_screen: false,
            pin: String::new(),
        }
    }
}

/// Thread-safe settings store. Reads are cheap (inner RwLock), writes are
/// persisted to disk synchronously.
#[derive(Debug)]
pub struct ConfigStore {
    settings: RwLock<Settings>,
    path: PathBuf,
    pub app_data: PathBuf,
}

impl ConfigStore {
    /// Load settings from `<app_data>/settings.json`, creating it if missing.
    /// `download_dir` defaults to `<app_data>/NClientT/Download`.
    pub fn load_or_init(app_data: &Path) -> Self {
        let path = app_data.join("settings.json");
        let default_download = default_download_dir(app_data);

        let mut settings = match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
                log::warn!("settings.json invalid ({e}), using defaults");
                Settings {
                    download_dir: default_download.clone(),
                    ..Settings::default()
                }
            }),
            Err(_) => Settings {
                download_dir: default_download.clone(),
                ..Settings::default()
            },
        };

        if settings.download_dir.as_os_str().is_empty() {
            settings.download_dir = default_download;
        }
        fs::create_dir_all(&settings.download_dir).ok();

        let store = Self {
            settings: RwLock::new(settings.clone()),
            path,
            app_data: app_data.to_path_buf(),
        };
        store.persist(&settings);
        store
    }

    pub fn get(&self) -> Settings {
        self.settings.read().unwrap().clone()
    }

    /// Path of the `settings.json` file (its parent dir holds cookies.json
    /// and the database too).
    pub fn path_of_setting(&self) -> Option<PathBuf> {
        if self.path.as_os_str().is_empty() {
            None
        } else {
            Some(self.path.clone())
        }
    }

    pub fn update<F: FnOnce(&mut Settings)>(&self, f: F) -> AppResult<Settings> {
        let mut settings = self.settings.write().unwrap();
        f(&mut settings);
        let snapshot = settings.clone();
        drop(settings);
        self.persist(&snapshot);
        Ok(snapshot)
    }

    pub fn replace(&self, new_settings: Settings) -> AppResult<Settings> {
        let mut sanitized = new_settings;
        if sanitized.download_dir.as_os_str().is_empty() {
            sanitized.download_dir = self.get().download_dir;
        }
        fs::create_dir_all(&sanitized.download_dir).ok();
        self.persist(&sanitized);
        *self.settings.write().unwrap() = sanitized.clone();
        Ok(sanitized)
    }

    fn persist(&self, settings: &Settings) {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).ok();
        }
        match serde_json::to_string_pretty(settings) {
            Ok(json) => {
                if let Err(e) = fs::write(&self.path, json) {
                    log::error!("failed to write settings.json: {e}");
                }
            }
            Err(e) => log::error!("failed to serialize settings: {e}"),
        }
    }

    // --- convenience accessors used by other modules -----------------------

    /// Mirrors `Utility.getBaseUrl()` -> `https://<mirror>/`.
    pub fn base_url(&self) -> String {
        format!("https://{}/", self.get().mirror)
    }

    /// Mirrors `Utility.getApiBaseUrl()` -> `https://<mirror>/api/v2/`.
    pub fn api_base_url(&self) -> String {
        format!("https://{}/api/v2/", self.get().mirror)
    }

    pub fn download_dir(&self) -> PathBuf {
        self.get().download_dir
    }

    /// Effective User-Agent (override or default).
    pub fn user_agent(&self) -> String {
        let s = self.get();
        if s.user_agent.trim().is_empty() {
            DEFAULT_UA.to_string()
        } else {
            s.user_agent.trim().to_string()
        }
    }
}

/// Quick helper to read the cookie store file path used by [`crate::http`].
pub fn cookie_db_path(app_data: &Path) -> PathBuf {
    app_data.join("cookies.json")
}

/// Default download directory, platform-aware.
/// On Android we first try the public Download directory, then the app-specific
/// external storage (which works without special permissions on API 29+),
/// lastly fall back to internal app data.
fn default_download_dir(app_data: &Path) -> PathBuf {
    #[cfg(target_os = "android")]
    {
        let public = PathBuf::from("/storage/emulated/0/Download/NClientT");
        if std::fs::create_dir_all(&public).is_ok() {
            log::info!("Android download dir: {}", public.display());
            return public;
        }
        log::warn!("Cannot use public download dir; trying app external storage");
        if let Ok(ext) = std::env::var("EXTERNAL_STORAGE") {
            let ext_path = PathBuf::from(format!(
                "{}/Android/data/com.nclientt.app/files/NClientT/Download",
                ext.trim_end_matches('/')
            ));
            if std::fs::create_dir_all(&ext_path).is_ok() {
                log::info!("Android download dir (app external): {}", ext_path.display());
                return ext_path;
            }
        }
        let fallback = app_data.join("NClientT").join("Download");
        log::info!("Android download dir (internal): {}", fallback.display());
        return fallback;
    }
    #[cfg(not(target_os = "android"))]
    {
        app_data.join("NClientT").join("Download")
    }
}

/// Check whether the trailing two bytes of a JPEG file are `FF D9`.
/// Mirrors `Global#isJPEGCorrupted`.
pub fn is_jpeg_corrupted(path: &Path) -> bool {
    use std::io::{Read, Seek, SeekFrom};
    if !path.exists() {
        return true;
    }
    let Ok(mut f) = fs::OpenOptions::new().read(true).open(path) else {
        return true;
    };
    let len = f.metadata().map(|m| m.len()).unwrap_or(0);
    if len < 10 {
        return true;
    }
    if f.seek(SeekFrom::End(-2)).is_err() {
        return true;
    }
    let mut eoi = [0u8; 2];
    if f.read_exact(&mut eoi).is_err() {
        return true;
    }
    eoi[0] != 0xFF || eoi[1] != 0xD9
}
