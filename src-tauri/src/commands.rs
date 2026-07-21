//! Tauri command handlers — the bridge between the frontend and the backend.
//!
//! Every function here is exposed to JS via `invoke('name', { ... })`.

use std::path::PathBuf;

use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

use crate::api::ApiClient;
use crate::cloudflare;
use crate::config::{AuthCredentials, Settings, TitleType};
use crate::db::{DownloadRow, FavoriteRow, ReadProgressRow};
use crate::downloader::{DownloadRequest, DownloadStatus};
use crate::error::{AppError, AppResult};
use crate::http::HttpClient;
use crate::models::*;
use crate::AppState;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn api(state: &State<AppState>) -> ApiClient {
    ApiClient::new(state.http.clone(), state.config.clone())
}

fn settings(state: &State<AppState>) -> Settings {
    state.config.get()
}

// ===========================================================================
// Settings
// ===========================================================================

#[tauri::command]
pub fn settings_get(state: State<AppState>) -> AppResult<Settings> {
    Ok(settings(&state))
}

#[tauri::command]
pub fn settings_set(
    state: State<'_, AppState>,
    new_settings: Settings,
) -> AppResult<Settings> {
    let updated = state.config.replace(new_settings)?;
    // Mirror / UA change => rebuild HTTP client (cookies preserved).
    state.http.rebuild(&updated);
    state
        .downloads
        .set_download_dir(updated.download_dir.clone());
    Ok(updated)
}

#[tauri::command]
pub fn settings_get_paths(app: AppHandle) -> AppResult<serde_json::Value> {
    let data = app.path().app_data_dir()?;
    Ok(serde_json::json!({
        "app_data": data,
        "log_dir": app.path().app_log_dir().ok(),
    }))
}

#[tauri::command]
pub async fn settings_pick_directory(state: State<'_, AppState>) -> AppResult<Option<String>> {
    // No native directory dialog on Android — return the recommended default
    // (the app's external files directory) so the frontend can prefill the field
    // and the user can switch via `settings_list_download_candidates`.
    #[cfg(target_os = "android")]
    {
        let candidates = crate::config::download_candidates(&state.config.app_data);
        // Prefer the "app external storage (recommended)" entry; fall back to
        // the last candidate (internal storage), which always exists.
        let pick = candidates
            .iter()
            .find(|(label, _)| label.contains("recommended"))
            .or_else(|| candidates.last())
            .map(|(_, p)| p.to_string_lossy().to_string())
            .unwrap_or_else(|| {
                state
                    .config
                    .app_data
                    .join("NClientT")
                    .join("Download")
                    .to_string_lossy()
                    .to_string()
            });
        Ok(Some(pick))
    }
    #[cfg(not(target_os = "android"))]
    {
        // Desktop uses the native dialog in the frontend; nothing to return.
        let _ = state;
        Ok(None)
    }
}

/// Return candidate download directories `(label, path)` the user can pick from
/// when the native directory dialog is unavailable (Android). See
/// [`crate::config::download_candidates`].
#[tauri::command]
pub fn settings_list_download_candidates(
    state: State<'_, AppState>,
) -> AppResult<Vec<(String, String)>> {
    Ok(crate::config::download_candidates(&state.config.app_data)
        .into_iter()
        .map(|(label, path)| (label.to_string(), path.to_string_lossy().to_string()))
        .collect())
}

#[tauri::command]
pub fn settings_clear_cookies(state: State<'_, AppState>) -> AppResult<()> {
    state.http.clear_cookies()
}

// ===========================================================================
// Auth + Cloudflare
// ===========================================================================

#[tauri::command]
pub fn auth_get(state: State<'_, AppState>) -> AppResult<AuthCredentials> {
    Ok(state.config.get().auth)
}

#[tauri::command]
pub fn auth_set_api_key(
    state: State<'_, AppState>,
    api_key: String,
) -> AppResult<Settings> {
    let updated = state.config.update(|s| {
        s.auth = AuthCredentials {
            api_key: api_key.trim().to_string(),
            valid: true,
        };
    })?;
    Ok(updated)
}

#[tauri::command]
pub fn auth_clear(state: State<'_, AppState>) -> AppResult<Settings> {
    let updated = state.config.update(|s| {
        s.auth = AuthCredentials::default();
    })?;
    Ok(updated)
}

#[tauri::command]
pub fn auth_status(state: State<'_, AppState>) -> AppResult<AuthStatus> {
    let s = settings(&state);
    Ok(AuthStatus {
        has_credentials: s.auth.has_credentials(),
        api_key_valid: s.auth.valid,
        cloudflare_solved: cloudflare::is_solved(),
    })
}

#[tauri::command]
pub async fn cloudflare_check(state: State<'_, AppState>) -> AppResult<bool> {
    // Probe the API base with a lightweight request; if CF blocks, we get
    // `AppError::Cloudflare`.
    let s = settings(&state);
    let url = format!("{}galleries?page=1", state.config.api_base_url());
    match state.http.get_text(&url, true, &s).await {
        Ok(_) => {
            cloudflare::set_state(crate::models::CfState::Solved);
            Ok(false)
        }
        Err(AppError::Cloudflare) => {
            cloudflare::set_state(crate::models::CfState::Needed);
            Ok(true)
        }
        Err(e) => {
            log::info!("cloudflare check: {e}");
            Ok(false)
        }
    }
}

#[tauri::command]
pub fn cloudflare_open_challenge(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let base = state.config.base_url();
    cloudflare::open_challenge(&app, state.http.clone(), base)
}

#[tauri::command]
pub fn cloudflare_is_solved() -> bool {
    cloudflare::is_solved()
}

// ===========================================================================
// API: browse / search / random / detail
// ===========================================================================

#[tauri::command]
pub async fn api_browse(
    state: State<'_, AppState>,
    page: u32,
    sort: crate::config::SortType,
) -> AppResult<SearchPage> {
    api(&state).browse(page, sort).await
}

#[tauri::command]
pub async fn api_search(
    state: State<'_, AppState>,
    query: SearchQuery,
) -> AppResult<SearchPage> {
    api(&state).search(&query).await
}

#[tauri::command]
pub async fn api_random(state: State<'_, AppState>) -> AppResult<Gallery> {
    api(&state).random().await
}

#[tauri::command]
pub async fn api_get_gallery(
    state: State<'_, AppState>,
    id: i64,
) -> AppResult<Gallery> {
    let g = api(&state).gallery(id).await?;
    let s = settings(&state);
    // Record visit in history (mirrors NClientV3's history table).
    if s.keep_history {
        let _ = state.db.history_add(
            g.id,
            &g.best_title(s.title_type),
            g.media_id,
            g.thumbnail.thumbnail_or_path().unwrap_or(""),
        );
    }
    Ok(g)
}

#[tauri::command]
pub async fn api_get_user(state: State<'_, AppState>) -> AppResult<User> {
    api(&state).user().await
}

#[tauri::command]
pub async fn api_get_comments(
    state: State<'_, AppState>,
    gallery_id: i64,
) -> AppResult<CommentsPage> {
    api(&state).comments(gallery_id).await
}

#[tauri::command]
pub async fn api_get_favorites_page(
    state: State<'_, AppState>,
    page: u32,
    query: Option<String>,
) -> AppResult<FavoritesPage> {
    api(&state).favorites_page(page, query.as_deref()).await
}

#[tauri::command]
pub async fn api_get_tags(
    state: State<'_, AppState>,
    type_filter: Option<TagType>,
) -> AppResult<Vec<Tag>> {
    let cached = state.db.tags_by_type(type_filter).unwrap_or_default();
    if !cached.is_empty() {
        return Ok(cached);
    }
    let remote = api(&state).popular_tags().await?;
    for t in &remote {
        let _ = state.db.tag_insert_or_update(t);
    }
    Ok(remote)
}

#[tauri::command]
pub async fn api_get_popular_tags(state: State<'_, AppState>) -> AppResult<Vec<Tag>> {
    api(&state).popular_tags().await
}

// ===========================================================================
// Favorites (local DB)
// ===========================================================================

#[tauri::command]
pub fn fav_add(
    state: State<'_, AppState>,
    id: i64,
    title: String,
    media_id: i64,
    thumbnail: String,
) -> AppResult<()> {
    state.db.fav_add(id, &title, media_id, &thumbnail)
}

#[tauri::command]
pub fn fav_remove(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    state.db.fav_remove(id)
}

#[tauri::command]
pub fn fav_is_favorite(state: State<'_, AppState>, id: i64) -> AppResult<bool> {
    state.db.fav_is(id)
}

#[tauri::command]
pub fn fav_list(
    state: State<'_, AppState>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> AppResult<Vec<FavoriteRow>> {
    state.db.fav_list(limit.unwrap_or(100), offset.unwrap_or(0))
}

// ===========================================================================
// Tags (local DB)
// ===========================================================================

#[tauri::command]
pub fn tags_get_all(state: State<'_, AppState>) -> AppResult<Vec<Tag>> {
    state.db.tags_all()
}

#[tauri::command]
pub fn tags_get_by_type(
    state: State<'_, AppState>,
    type_filter: Option<TagType>,
) -> AppResult<Vec<Tag>> {
    state.db.tags_by_type(type_filter)
}

#[tauri::command]
pub fn tags_set_status(
    state: State<'_, AppState>,
    id: i64,
    status: TagStatus,
) -> AppResult<()> {
    state.db.tag_set_status(id, status)
}

#[tauri::command]
pub fn tags_add_blacklist(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    state.db.tag_set_blacklist(id, true)
}

#[tauri::command]
pub fn tags_remove_blacklist(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    state.db.tag_set_blacklist(id, false)
}

#[tauri::command]
pub async fn tags_search(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> AppResult<Vec<Tag>> {
    api(&state).search_tags(&query, limit.unwrap_or(50)).await
}

#[tauri::command]
pub async fn tags_get_popular(state: State<'_, AppState>) -> AppResult<Vec<Tag>> {
    api(&state).popular_tags().await
}

// ===========================================================================
// History
// ===========================================================================

#[tauri::command]
pub fn history_add(
    state: State<'_, AppState>,
    id: i64,
    title: String,
    media_id: i64,
    thumbnail: String,
) -> AppResult<()> {
    state.db.history_add(id, &title, media_id, &thumbnail)
}

#[tauri::command]
pub fn history_list(state: State<'_, AppState>, limit: Option<u32>) -> AppResult<Vec<HistoryEntry>> {
    state.db.history_list(limit.unwrap_or(200))
}

#[tauri::command]
pub fn history_clear(state: State<'_, AppState>) -> AppResult<()> {
    state.db.history_clear()
}

// ===========================================================================
// Read progress
// ===========================================================================

#[tauri::command]
pub fn read_progress_set(
    state: State<'_, AppState>,
    gallery_id: i64,
    last_page: usize,
    total_pages: usize,
) -> AppResult<()> {
    state
        .db
        .read_progress_upsert(gallery_id, last_page, total_pages)
}

#[tauri::command]
pub fn read_progress_reset(state: State<'_, AppState>, gallery_id: i64) -> AppResult<()> {
    state.db.read_progress_reset(gallery_id)
}

#[tauri::command]
pub fn read_progress_get(
    state: State<'_, AppState>,
    gallery_id: i64,
) -> AppResult<Option<ReadProgressRow>> {
    state.db.read_progress_get(gallery_id)
}

/// IDs of galleries the user has read >= 50% of. The frontend uses this to
/// badge covers in the gallery grid and local library.
#[tauri::command]
pub fn read_progress_ids(state: State<'_, AppState>) -> AppResult<Vec<i64>> {
    state.db.read_progress_ids()
}

// ===========================================================================
// Local library
// ===========================================================================

#[tauri::command]
pub fn local_scan(state: State<'_, AppState>) -> AppResult<Vec<LocalGallery>> {
    let mut found = Vec::new();
    // `mut` is only needed on Android, where we may push an extra scan dir.
    #[allow(unused_mut)]
    let mut dirs = vec![state.config.download_dir()];
    // On Android also scan the internal fallback if different.
    #[cfg(target_os = "android")]
    {
        let internal = &state.config.app_data.join("NClientT").join("Download");
        if *internal != *dirs[0] && internal.exists() {
            dirs.push(internal.clone());
        }
    }
    for dir in &dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for e in entries.flatten() {
                if !e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue;
                }
                if let Some(lg) = read_local_gallery(&e.path()) {
                    let _ = state.db.local_upsert(&lg);
                    found.push(lg);
                }
            }
        }
    }
    // Merge any DB-only entries.
    if let Ok(all) = state.db.local_all() {
        for lg in all {
            if !found.iter().any(|f| f.folder == lg.folder) {
                found.push(lg);
            }
        }
    }
    Ok(found)
}

#[tauri::command]
pub fn local_list(state: State<'_, AppState>) -> AppResult<Vec<LocalGallery>> {
    let mut items = state.db.local_all().unwrap_or_default();
    // Quick scan of download dir: add folders on disk but missing from DB.
    let dir = state.config.download_dir();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for e in entries.flatten() {
            if !e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let p = e.path();
            let folder_str = p.to_string_lossy().to_string();
            if items.iter().any(|i| i.folder == folder_str) {
                continue;
            }
            if let Some(lg) = read_local_gallery(&p) {
                let _ = state.db.local_upsert(&lg);
                items.push(lg);
            }
        }
    }
    Ok(items)
}

#[tauri::command]
pub fn local_delete(state: State<'_, AppState>, folder: String) -> AppResult<()> {
    let path = PathBuf::from(&folder);
    if path.exists() {
        std::fs::remove_dir_all(&path)?;
    }
    state.db.local_remove(&folder)
}

#[tauri::command]
pub fn local_import_folder(_folder: String) -> AppResult<bool> {
    // Reserved: future "import existing gallery folder" flow.
    Ok(false)
}

fn read_local_gallery(folder: &std::path::Path) -> Option<LocalGallery> {
    let mut id = 0i64;
    let mut title = folder
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let mut media_id = 0i64;

    // Read the `.<id>` marker file.
    if let Ok(entries) = std::fs::read_dir(folder) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.len() > 1 && name.starts_with('.') && name[1..].chars().all(|c| c.is_ascii_digit()) {
                id = name[1..].parse().unwrap_or(0);
            }
        }
    }
    // Read metadata from `.nomedia` if present.
    let nomedia = folder.join(".nomedia");
    if let Ok(content) = std::fs::read_to_string(&nomedia) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
            if id == 0 {
                id = v.get("id").and_then(|x| x.as_i64()).unwrap_or(0);
            }
            media_id = v
                .get("media_id")
                .and_then(|x| x.as_str())
                .and_then(|s| s.parse().ok())
                .or_else(|| v.get("media_id").and_then(|x| x.as_i64()))
                .unwrap_or(0);
            let pref = TitleType::Pretty;
            let titles = v.get("title");
            let pick = |key: &str| -> String {
                titles
                    .and_then(|t| t.get(key))
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string()
            };
            let pretty = pick("pretty");
            let english = pick("english");
            let new_title = if !pretty.is_empty() {
                pretty
            } else if !english.is_empty() {
                english
            } else {
                String::new()
            };
            if !new_title.is_empty() {
                title = new_title;
            }
            let _ = pref;
        }
    }

    let mut page_files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(folder) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            let lower = name.to_ascii_lowercase();
            if lower.ends_with(".jpg")
                || lower.ends_with(".jpeg")
                || lower.ends_with(".png")
                || lower.ends_with(".gif")
                || lower.ends_with(".webp")
            {
                page_files.push(e.path().to_string_lossy().to_string());
            }
        }
    }
    page_files.sort();
    if page_files.is_empty() && id == 0 {
        return None;
    }

    Some(LocalGallery {
        id,
        title,
        thumbnail_path: page_files.first().cloned(),
        folder: folder.to_string_lossy().to_string(),
        num_pages: page_files.len(),
        page_files,
        media_id,
    })
}

// ===========================================================================
// Downloader
// ===========================================================================

#[tauri::command]
pub async fn download_gallery(
    state: State<'_, AppState>,
    req: DownloadRequest,
) -> AppResult<serde_json::Value> {
    let api = api(&state);
    let mgr = state.downloads.clone();
    let entry = mgr.enqueue(api, req).await?;
    Ok(serde_json::to_value(entry)?)
}

#[tauri::command]
pub async fn download_range(
    state: State<'_, AppState>,
    gallery_id: i64,
    from_page: Option<usize>,
    to_page: Option<usize>,
) -> AppResult<serde_json::Value> {
    let api = api(&state);
    let mgr = state.downloads.clone();
    let entry = mgr
        .enqueue(
            api,
            DownloadRequest {
                gallery_id,
                from_page,
                to_page,
            },
        )
        .await?;
    Ok(serde_json::to_value(entry)?)
}

#[tauri::command]
pub fn download_list(state: State<'_, AppState>) -> Vec<serde_json::Value> {
    state
        .downloads
        .list()
        .into_iter()
        .filter_map(|e| serde_json::to_value(e).ok())
        .collect()
}

#[tauri::command]
pub async fn download_cancel(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    state.downloads.cancel(id)
}

#[tauri::command]
pub fn download_pause(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    state.downloads.pause(id)
}

#[tauri::command]
pub async fn download_resume(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let api = api(&state);
    state.downloads.clone().resume(api, id)
}

#[tauri::command]
pub fn download_clear(state: State<'_, AppState>) -> AppResult<()> {
    state.downloads.clear_finished()
}

/// Surface persisted (resumable) downloads to the frontend on startup.
#[tauri::command]
pub fn download_rows(state: State<'_, AppState>) -> AppResult<Vec<DownloadRow>> {
    state.db.downloads_all()
}

// ===========================================================================
// Export
// ===========================================================================

#[tauri::command]
pub fn export_pdf(folder: String, out: Option<String>) -> AppResult<String> {
    let path = crate::export::export_pdf(
        std::path::Path::new(&folder),
        out.as_deref().map(std::path::Path::new),
    )?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn export_zip(folder: String, out: Option<String>) -> AppResult<String> {
    let path = crate::export::export_zip(
        std::path::Path::new(&folder),
        out.as_deref().map(std::path::Path::new),
    )?;
    Ok(path.to_string_lossy().to_string())
}

// ===========================================================================
// Misc: open URLs / paths, asset resolution, image proxy
// ===========================================================================

#[tauri::command]
pub fn open_in_browser(app: AppHandle, state: State<'_, AppState>, path: String) -> AppResult<()> {
    let base = state.config.base_url();
    let url = if path.starts_with("http") {
        path
    } else if let Ok(id) = path.parse::<i64>() {
        format!("{}g/{}", base, id)
    } else {
        format!("{}{}", base, path.trim_start_matches('/'))
    };
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| AppError::Other(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn open_path(app: AppHandle, path: String) -> AppResult<()> {
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|e| AppError::Other(e.to_string()))?;
    Ok(())
}

/// Convert a `file://` or absolute path to an asset URL the frontend can
/// load in `<img src>`. The scheme is platform-specific: WebView2
/// (Windows/Android) requires `http://asset.localhost/`, while macOS/Linux
/// use `asset://localhost/`.
#[tauri::command]
pub fn resolve_asset(path: String) -> AppResult<String> {
    let p = if let Some(rest) = path.strip_prefix("file://") {
        rest.to_string()
    } else {
        path
    };
    Ok(asset_url(&p))
}

/// For remote images we cannot load directly from the renderer (CSP), return
/// a hint the frontend uses to set `src` — the `asset` protocol handler
/// serves local files; remote ones go through the normal `<img>` with a
/// relaxed CSP. This command exists so the frontend can ask for the right
/// scheme given a path/URL.
#[tauri::command]
pub fn image_proxy_url(url: String) -> String {
    if url.starts_with("http") {
        url
    } else {
        asset_url(url.trim_start_matches('/'))
    }
}

/// Build a per-platform asset URL for a local path. Mirrors the scheme
/// selection done by Tauri's frontend `convertFileSrc` helper:
/// `http://asset.localhost/<path>` on Windows & Android, `asset://localhost/`
/// elsewhere. The path is percent-encoded like the JS `encodeURIComponent`
/// that `convertFileSrc` uses.
fn asset_url(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let encoded = percent_encode_path(&normalized);
    // WebView2 (Windows) and Android's WebView only recognise the asset
    // protocol under the `http://asset.localhost` host; macOS/Linux use the
    // bare `asset://` scheme.
    #[cfg(any(target_os = "windows", target_os = "android"))]
    {
        format!("http://asset.localhost/{}", encoded)
    }
    #[cfg(not(any(target_os = "windows", target_os = "android")))]
    {
        format!("asset://localhost/{}", encoded)
    }
}

/// Percent-encode a path for use in a URL path component, matching JS
/// `encodeURIComponent` (encodes everything except the unreserved set
/// `A-Za-z0-9-_.!~*'()` and the path separators `/`).
fn percent_encode_path(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        match b {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'_'
            | b'.'
            | b'!'
            | b'~'
            | b'*'
            | b'\''
            | b'('
            | b')'
            | b'/' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

/// Read a local image file as base64 data URL. Useful when the asset protocol
/// is unavailable or for tiny thumbnails.
#[tauri::command]
pub fn read_local_image(path: String) -> AppResult<Option<String>> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(&p)?;
    let mime = mime_guess::from_path(&p)
        .first_or_octet_stream()
        .essence_str()
        .to_string();
    // Default to image/jpeg for known image extensions that mime_guess reports
    // as application/octet-stream (e.g. some .webp / .gif variants).
    let mime = if mime == "application/octet-stream" {
        "image/jpeg".to_string()
    } else {
        mime
    };
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
    Ok(Some(format!("data:{};base64,{}", mime, b64)))
}

/// Register the download manager's app handle for event emission. Called once
/// from the frontend shortly after startup (and also at `setup` time).
#[tauri::command]
pub fn register_app(app: AppHandle, state: State<'_, AppState>) -> AppResult<()> {
    state.downloads.set_app_handle(app);
    Ok(())
}

// Suppress unused-import warning while keeping `HttpClient` /
// `DownloadStatus` reachable for type inference in tooling.
#[allow(dead_code)]
fn _type_anchors(_h: &HttpClient, _s: DownloadStatus, _t: &Tag) {}
