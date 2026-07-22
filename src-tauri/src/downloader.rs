//! Gallery download manager.
//!
//! Port of NClientV3's `async/downloader` package
//! (`GalleryDownloaderManager` + `GalleryDownloaderV2` + `DownloadQueue`).
//!
//! Each gallery is downloaded into `<download_dir>/<title>/.<id>` (the `.<id>`
//! marker file mirrors NClientV3's `createIdFile()`), with pages named
//! `001.<ext>`, `002.<ext>`, ... (mirrors `PageContainer#getPageName`).
//! A `.nomedia` metadata file stores the gallery JSON (mirrors
//! `GalleryDownloaderV2#writeNoMedia`).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::Semaphore;

use crate::api::ApiClient;
use crate::config::is_jpeg_corrupted;
use crate::db::Database;
use crate::error::{AppError, AppResult};
use crate::http::HttpClient;
use crate::models::Gallery;

/// Per-gallery status, mirrors `GalleryDownloaderV2.Status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Paused,
    Finished,
    Canceled,
    Failed,
}

impl DownloadStatus {
    fn db_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Downloading => "downloading",
            Self::Paused => "paused",
            Self::Finished => "finished",
            Self::Canceled => "canceled",
            Self::Failed => "failed",
        }
    }
}

/// Live progress for a download, emitted to the frontend via the
/// `download:progress` event.
#[derive(Debug, Clone, Serialize)]
pub struct DownloadProgress {
    pub id: i64,
    pub title: String,
    pub folder: String,
    pub status: DownloadStatus,
    pub done_pages: usize,
    pub total_pages: usize,
    pub current_page: Option<usize>,
    pub error: Option<String>,
    pub bytes_per_second: Option<f64>,
    pub total_bytes: Option<u64>,
}

/// Request payload used by the frontend to start a download.
#[derive(Debug, Clone, Deserialize)]
pub struct DownloadRequest {
    pub gallery_id: i64,
    /// Page range (inclusive); None => whole gallery.
    pub from_page: Option<usize>,
    pub to_page: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DownloadEntry {
    pub id: i64,
    pub title: String,
    pub folder: String,
    pub thumbnail: Option<String>,
    pub status: DownloadStatus,
    pub done_pages: usize,
    pub total_pages: usize,
}

/// Internal state for a single gallery download.
struct DownloadTask {
    id: i64,
    title: String,
    folder: PathBuf,
    thumbnail: Option<String>,
    pages: Vec<(usize, String)>, // (page_index, url)
    done: Mutex<Vec<bool>>,
    status: RwLock<DownloadStatus>,
    pause_flag: Mutex<bool>,
    cancel_flag: Mutex<bool>,
    bytes_downloaded: AtomicU64,
    start_time: Mutex<Option<Instant>>,
}

impl DownloadTask {
    fn done_count(&self) -> usize {
        self.done.lock().iter().filter(|b| **b).count()
    }
    fn mark_done(&self, idx: usize) {
        let mut g = self.done.lock();
        if idx < g.len() {
            g[idx] = true;
        }
    }
    fn next_pending(&self) -> Option<usize> {
        let g = self.done.lock();
        g.iter().position(|b| !b)
    }
    fn add_bytes(&self, n: u64) {
        let mut t = self.start_time.lock();
        if t.is_none() {
            *t = Some(Instant::now());
        }
        self.bytes_downloaded.fetch_add(n, Ordering::SeqCst);
    }
    fn speed(&self) -> Option<f64> {
        let t = self.start_time.lock();
        let start = (*t)?;
        let elapsed = start.elapsed().as_secs_f64();
        if elapsed < 0.1 {
            return None;
        }
        let bytes = self.bytes_downloaded.load(Ordering::SeqCst) as f64;
        Some(bytes / elapsed)
    }
    fn total_bytes_downloaded(&self) -> u64 {
        self.bytes_downloaded.load(Ordering::SeqCst)
    }
}

/// The application-wide download manager.
pub struct DownloadManager {
    download_dir: RwLock<PathBuf>,
    http: Arc<HttpClient>,
    db: Database,
    /// Per-gallery active tasks.
    tasks: RwLock<HashMap<i64, Arc<DownloadTask>>>,
    /// -----------------------------------------------------------------------
    /// WARNING — SERVER-SIDE RATE LIMITING
    /// -----------------------------------------------------------------------
    /// This semaphore limits concurrent gallery downloads. It MUST remain at 1.
    /// Increasing this value will download multiple galleries in parallel,
    /// opening excessive TCP connections to the source server and triggering
    /// IP bans / Cloudflare throttling. **DO NOT CHANGE without maintainer approval.**
    /// -----------------------------------------------------------------------
    sem: Arc<Semaphore>,
    /// App handle used to emit progress events.
    app: Mutex<Option<AppHandle>>,
}

impl std::fmt::Debug for DownloadManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DownloadManager").finish()
    }
}

impl DownloadManager {
    pub fn new(download_dir: PathBuf, http: Arc<HttpClient>, db: Database) -> Self {
        std::fs::create_dir_all(&download_dir).ok();
        Self {
            download_dir: RwLock::new(download_dir),
            http,
            db,
            tasks: RwLock::new(HashMap::new()),
            sem: Arc::new(Semaphore::new(1)), // MUST stay at 1 to avoid server IP bans / rate limiting. DO NOT CHANGE.
            app: Mutex::new(None),
        }
    }

    pub fn set_app_handle(&self, app: AppHandle) {
        *self.app.lock() = Some(app);
    }

    pub fn set_download_dir(&self, dir: PathBuf) {
        std::fs::create_dir_all(&dir).ok();
        *self.download_dir.write() = dir;
    }

    pub fn list(&self) -> Vec<DownloadEntry> {
        let tasks = self.tasks.read();
        let mut entries: Vec<DownloadEntry> = tasks
            .values()
            .map(|t| DownloadEntry {
                id: t.id,
                title: t.title.clone(),
                folder: t.folder.to_string_lossy().to_string(),
                thumbnail: t.thumbnail.clone(),
                status: *t.status.read(),
                done_pages: t.done_count(),
                total_pages: t.pages.len(),
            })
            .collect();
        // Merge persisted (resumable) rows from the DB.
        if let Ok(rows) = self.db.downloads_all() {
            for r in rows {
                if entries.iter().any(|e| e.id == r.id) {
                    continue;
                }
                let status = match r.status.as_str() {
                    "finished" => DownloadStatus::Finished,
                    "paused" => DownloadStatus::Paused,
                    "canceled" => DownloadStatus::Canceled,
                    "failed" => DownloadStatus::Failed,
                    "downloading" => DownloadStatus::Paused,
                    _ => DownloadStatus::Pending,
                };
                entries.push(DownloadEntry {
                    id: r.id,
                    title: r.title,
                    folder: r.folder,
                    thumbnail: if r.thumbnail.is_empty() {
                        None
                    } else {
                        Some(r.thumbnail)
                    },
                    status,
                    done_pages: r.done_pages,
                    total_pages: r.total_pages,
                });
            }
        }
        entries.sort_by(|a, b| b.id.cmp(&a.id));
        entries
    }

    /// Enqueue a download. Fetches gallery detail first, creates the folder,
    /// then runs the page-fetch loop on a background task.
    pub async fn enqueue(
        self: Arc<Self>,
        api: ApiClient,
        req: DownloadRequest,
    ) -> AppResult<DownloadEntry> {
        // Fetch gallery detail.
        let gallery = api.gallery(req.gallery_id).await?;
        let entry = self.spawn_for_gallery(api.clone(), gallery, req.from_page, req.to_page)?;
        Ok(entry)
    }

    /// Spawn a download for an already-fetched gallery (used internally).
    pub fn spawn_for_gallery(
        self: Arc<Self>,
        api: ApiClient,
        gallery: Gallery,
        from_page: Option<usize>,
        to_page: Option<usize>,
    ) -> AppResult<DownloadEntry> {
        let settings = api.config.get();
        let title_pref = settings.title_type;
        let folder_name = gallery.download_folder_name(title_pref);
        let folder = self.allocate_folder(&folder_name, gallery.id);
        std::fs::create_dir_all(&folder).ok();
        write_id_file(&folder, gallery.id);
        write_gallery_meta(&folder, &gallery);

        // Build the page list (page_index, url).
        let pages = gallery
            .pages
            .iter()
            .enumerate()
            .filter(|(i, _)| match (from_page, to_page) {
                (Some(a), Some(b)) => *i >= a.saturating_sub(1) && *i < b,
                (Some(a), None) => *i >= a.saturating_sub(1),
                (None, Some(b)) => *i < b,
                (None, None) => true,
            })
            .filter_map(|(i, p)| p.path.as_deref().map(|u| (i, u.to_string())))
            .collect::<Vec<_>>();

        let total = pages.len();
        let task = Arc::new(DownloadTask {
            id: gallery.id,
            title: gallery.best_title(title_pref),
            folder: folder.clone(),
            thumbnail: gallery.thumbnail.thumbnail_or_path().map(String::from),
            pages,
            done: Mutex::new(vec![false; total]),
            status: RwLock::new(DownloadStatus::Pending),
            pause_flag: Mutex::new(false),
            cancel_flag: Mutex::new(false),
            bytes_downloaded: AtomicU64::new(0),
            start_time: Mutex::new(None),
        });

        let entry = DownloadEntry {
            id: task.id,
            title: task.title.clone(),
            folder: task.folder.to_string_lossy().to_string(),
            thumbnail: task.thumbnail.clone(),
            status: DownloadStatus::Pending,
            done_pages: 0,
            total_pages: total,
        };

        self.tasks.write().insert(task.id, task.clone());
        self.persist(task.id, &entry, DownloadStatus::Pending)?;
        self.emit_progress(task.id, DownloadStatus::Pending, 0, total, None, None);

        let me = self.clone();
        let api = api.clone();
        tokio::spawn(async move {
            me.run_task(api, task).await;
        });

        Ok(entry)
    }

    async fn run_task(self: Arc<Self>, api: ApiClient, task: Arc<DownloadTask>) {
        // Acquire one of the limited download slots.
        let _permit = match self.sem.clone().acquire_owned().await {
            Ok(p) => p,
            Err(_) => return,
        };
        {
            let mut s = task.status.write();
            if *s == DownloadStatus::Canceled || *s == DownloadStatus::Paused {
                return;
            }
            *s = DownloadStatus::Downloading;
        }
        self.emit_progress(task.id, DownloadStatus::Downloading, task.done_count(), task.pages.len(), None, None);

        // Per-gallery page-fetch semaphore. Must remain at 8 to avoid
        // request spikes that the server may treat as abuse.
        // DO NOT increase without maintainer approval.
        let page_sem = Arc::new(Semaphore::new(api.config.get().parallel_pages as usize));
        let http = self.http.clone();
        let settings = api.config.get();

        let mut failed = false;
        loop {
            // Find the next pending page index.
            let idx = match task.next_pending() {
                Some(i) => i,
                None => break,
            };

            if *task.cancel_flag.lock() {
                self.set_status(&task, DownloadStatus::Canceled);
                self.emit_progress(task.id, DownloadStatus::Canceled, task.done_count(), task.pages.len(), None, None);
                self.cleanup_canceled(&task);
                return;
            }
            if *task.pause_flag.lock() {
                self.set_status(&task, DownloadStatus::Paused);
                self.emit_progress(task.id, DownloadStatus::Paused, task.done_count(), task.pages.len(), None, None);
                return;
            }

            let (page_index, url) = task.pages[idx].clone();
            let page_name = page_file_name(page_index, &url);
            let file_path = task.folder.join(&page_name);

            // Skip if already present and not corrupted.
            if file_path.exists() && !is_corrupted(&file_path, &page_name) {
                task.mark_done(idx);
                self.emit_progress(task.id, DownloadStatus::Downloading, task.done_count(), task.pages.len(), Some(page_index), None);
                continue;
            }

            let permit = match page_sem.clone().acquire_owned().await {
                Ok(p) => p,
                Err(_) => break,
            };
            let http = http.clone();
            let settings = settings.clone();
            let res = download_one_async(&http, &settings, &url, &file_path).await;
            drop(permit);

            match res {
                Ok(written) => {
                    task.mark_done(idx);
                    task.add_bytes(written);
                    self.emit_progress(task.id, DownloadStatus::Downloading, task.done_count(), task.pages.len(), Some(page_index), None);
                }
                Err(e) => {
                    log::warn!("page {} fetch failed for gallery {}: {}", page_index, task.id, e);
                    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                    let http = http.clone();
                    let settings = settings.clone();
                    let url2 = url.clone();
                    let file_path2 = file_path.clone();
                    let retry = download_one_async(&http, &settings, &url2, &file_path2).await;
                    match retry {
                        Ok(written) => {
                            task.mark_done(idx);
                            task.add_bytes(written);
                            self.emit_progress(task.id, DownloadStatus::Downloading, task.done_count(), task.pages.len(), Some(page_index), None);
                        }
                        Err(_) => {
                            failed = true;
                            self.emit_progress(task.id, DownloadStatus::Failed, task.done_count(), task.pages.len(), Some(page_index), Some(format!("page {} failed", page_index)));
                            break;
                        }
                    }
                }
            }
        }

        let total = task.pages.len();
        let done = task.done_count();
        if failed {
            self.set_status(&task, DownloadStatus::Failed);
            self.emit_progress(task.id, DownloadStatus::Failed, done, total, None, Some("download failed".into()));
        } else if done == total {
            self.set_status(&task, DownloadStatus::Finished);
            self.emit_progress(task.id, DownloadStatus::Finished, done, total, None, None);
            // Index finished gallery into the local library.
            self.index_finished(&task);
        } else {
            // Incomplete (paused): persist status.
            self.set_status(&task, DownloadStatus::Paused);
            self.emit_progress(task.id, DownloadStatus::Paused, done, total, None, None);
        }
    }

    fn set_status(&self, task: &Arc<DownloadTask>, status: DownloadStatus) {
        *task.status.write() = status;
        let entry = DownloadEntry {
            id: task.id,
            title: task.title.clone(),
            folder: task.folder.to_string_lossy().to_string(),
            thumbnail: task.thumbnail.clone(),
            status,
            done_pages: task.done_count(),
            total_pages: task.pages.len(),
        };
        let _ = self.persist(task.id, &entry, status);
    }

    fn persist(
        &self,
        id: i64,
        entry: &DownloadEntry,
        status: DownloadStatus,
    ) -> AppResult<()> {
        let _ = self.db.download_upsert(
            id,
            &entry.title,
            0,
            entry.thumbnail.as_deref().unwrap_or(""),
            &entry.folder,
            entry.total_pages,
            entry.done_pages,
            status.db_str(),
        );
        Ok(())
    }

    fn cleanup_canceled(&self, task: &Arc<DownloadTask>) {
        let _ = std::fs::remove_dir_all(&task.folder);
        let _ = self.db.download_remove(task.id);
    }

    fn index_finished(&self, task: &Arc<DownloadTask>) {
        let page_files = list_image_files(&task.folder);
        let lg = crate::models::LocalGallery {
            id: task.id,
            title: task.title.clone(),
            thumbnail_path: page_files.first().cloned(),
            folder: task.folder.to_string_lossy().to_string(),
            num_pages: page_files.len(),
            page_files,
            media_id: 0,
            scanned_at: chrono::Utc::now().to_rfc3339(),
            translated_title: String::new(),
        };
        let _ = self.db.local_upsert(&lg);
        let _ = self.db.download_remove(task.id);
    }

    /// Pick a non-conflicting folder name (matches `findFolder`).
    fn allocate_folder(&self, name: &str, id: i64) -> PathBuf {
        let base = self.download_dir.read().clone();
        let mut folder = base.join(name);
        if usable_folder(&folder, id) {
            return folder;
        }
        let mut i = 1;
        loop {
            folder = base.join(format!("{}.DUP{}", name, i));
            if usable_folder(&folder, id) {
                return folder;
            }
            i += 1;
        }
    }

    // --- control ----------------------------------------------------------

    pub fn pause(&self, id: i64) -> AppResult<()> {
        if let Some(t) = self.tasks.read().get(&id).cloned() {
            *t.pause_flag.lock() = true;
        }
        Ok(())
    }

    pub fn resume(self: Arc<Self>, api: ApiClient, id: i64) -> AppResult<()> {
        // Re-fetch the gallery and re-spawn if the in-memory task is gone.
        let task = {
            let tasks = self.tasks.read();
            tasks.get(&id).cloned()
        };
        if let Some(t) = task {
            // Resume by spawning a continuation task.
            *t.pause_flag.lock() = false;
            *t.status.write() = DownloadStatus::Pending;
            let me = self.clone();
            let t = t.clone();
            tokio::spawn(async move {
                me.run_task(api, t).await;
            });
            return Ok(());
        }
        // Not in memory: restart from DB metadata by re-fetching detail.
        let entry = self.tasks_resume_from_db(api, id);
        entry
    }

    fn tasks_resume_from_db(self: Arc<Self>, api: ApiClient, id: i64) -> AppResult<()> {
        let me = self.clone();
        tokio::spawn(async move {
            match api.gallery(id).await {
                Ok(g) => {
                    let _ = me.clone().spawn_for_gallery(api, g, None, None);
                }
                Err(e) => log::warn!("resume failed for {}: {}", id, e),
            }
        });
        Ok(())
    }

    pub fn cancel(&self, id: i64) -> AppResult<()> {
        if let Some(t) = self.tasks.read().get(&id).cloned() {
            *t.cancel_flag.lock() = true;
        } else {
            // Not in memory: drop the persisted row + folder.
            if let Ok(rows) = self.db.downloads_all() {
                if let Some(r) = rows.into_iter().find(|r| r.id == id) {
                    let _ = std::fs::remove_dir_all(&r.folder);
                    let _ = self.db.download_remove(id);
                }
            }
        }
        Ok(())
    }

    pub fn clear_finished(&self) -> AppResult<()> {
        let finished: Vec<i64> = self
            .tasks
            .read()
            .iter()
            .filter(|(_, t)| *t.status.read() == DownloadStatus::Finished)
            .map(|(k, _)| *k)
            .collect();
        for id in finished {
            self.tasks.write().remove(&id);
        }
        let _ = self.db;
        Ok(())
    }

    fn emit_progress(
        &self,
        id: i64,
        status: DownloadStatus,
        done: usize,
        total: usize,
        current: Option<usize>,
        error: Option<String>,
    ) {
        let app_opt = self.app.lock();
        if let Some(app) = app_opt.as_ref() {
            let tasks = self.tasks.read();
            let task = tasks.get(&id);
            let title = task.map(|t| t.title.clone()).unwrap_or_default();
            let folder = task.map(|t| t.folder.to_string_lossy().to_string()).unwrap_or_default();
            let (bps, tbytes) = task
                .map(|t| (t.speed(), Some(t.total_bytes_downloaded())))
                .unwrap_or((None, None));
            let _ = app.emit(
                "download:progress",
                DownloadProgress {
                    id,
                    title,
                    folder,
                    status,
                    done_pages: done,
                    total_pages: total,
                    current_page: current,
                    error,
                    bytes_per_second: bps,
                    total_bytes: tbytes,
                },
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Free helpers
// ---------------------------------------------------------------------------

fn usable_folder(folder: &Path, id: i64) -> bool {
    if !folder.exists() {
        return true;
    }
    if folder.join(format!(".{}", id)).exists() {
        return true;
    }
    // Has a different id marker.
    if let Ok(entries) = std::fs::read_dir(folder) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.len() > 1 && name.starts_with('.') && name[1..].chars().all(|c| c.is_ascii_digit()) {
                return false;
            }
        }
    }
    true
}

fn write_id_file(folder: &Path, id: i64) {
    let _ = std::fs::write(folder.join(format!(".{}", id)), b"");
}

/// Mirrors `GalleryDownloaderV2#writeNoMedia`. We store the gallery JSON so
/// the local library scanner can restore metadata without re-fetching.
fn write_gallery_meta(folder: &Path, gallery: &Gallery) {
    let nomedia = folder.join(".nomedia");
    if let Ok(json) = serde_json::to_string_pretty(gallery) {
        let _ = std::fs::write(nomedia, json);
    }
}

fn page_file_name(page_index: usize, url: &str) -> String {
    let ext = url
        .rsplit('.')
        .next()
        .map(|e| {
            let e = e.split('?').next().unwrap_or(e);
            let trimmed = e.trim();
            if trimmed.len() <= 5 && trimmed.chars().all(|c| c.is_ascii_alphanumeric()) {
                trimmed.to_string()
            } else {
                "jpg".to_string()
            }
        })
        .unwrap_or_else(|| "jpg".to_string());
    format!("{:03}.{}", page_index + 1, ext)
}

fn is_corrupted(path: &Path, name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        return is_jpeg_corrupted(path);
    }
    // For non-JPEG, accept anything bigger than a few bytes.
    path.metadata().map(|m| m.len() < 32).unwrap_or(true)
}

async fn download_one_async(
    http: &HttpClient,
    settings: &crate::config::Settings,
    url: &str,
    dest: &Path,
) -> AppResult<u64> {
    let resp = http.get_stream(url, settings).await?;
    let expected = resp.content_length();
    let tmp = dest.with_extension("part");
    if let Some(parent) = tmp.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    use tokio::io::AsyncWriteExt;
    let mut file = tokio::fs::File::create(&tmp).await?;
    let bytes = resp.bytes().await?;
    file.write_all(&bytes).await?;
    file.flush().await?;
    drop(file);
    let written = bytes.len() as u64;
    if let Some(exp) = expected {
        if written != exp {
            let _ = std::fs::remove_file(&tmp);
            return Err(AppError::Other(format!(
                "size mismatch: {} != {}",
                written, exp
            )));
        }
    }
    tokio::fs::rename(&tmp, dest).await?;
    Ok(written)
}

fn list_image_files(folder: &Path) -> Vec<String> {
    let mut files = Vec::new();
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
                files.push(e.path().to_string_lossy().to_string());
            }
        }
    }
    files.sort();
    files
}
