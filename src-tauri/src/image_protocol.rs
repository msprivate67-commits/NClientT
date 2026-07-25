//! Image protocol shared by local-library and remote gallery images.
//!
//! Renderer-side HTTP requests do not use the proxy configured for reqwest,
//! and the built-in asset protocol is not consistent across Linux WebKitGTK
//! versions. Routing both kinds of images through one custom protocol keeps
//! image loading on the Rust side, where proxy, cookies and headers are known.

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, Weak};

use once_cell::sync::Lazy;
use reqwest::header::CONTENT_TYPE;
use tauri::{http, Manager, UriSchemeContext, UriSchemeResponder, Wry};
use tokio::sync::{Mutex as AsyncMutex, Semaphore};

use crate::{config::Settings, http::HttpClient, AppState};

pub const SCHEME: &str = "nclient-image";
const MAX_REMOTE_IMAGE_REQUESTS: usize = 10;
const MAX_CACHED_IMAGES: usize = 512;
const MAX_CACHE_BYTES: usize = 256 * 1024 * 1024;
static REMOTE_IMAGE_LIMIT: Semaphore = Semaphore::const_new(MAX_REMOTE_IMAGE_REQUESTS);
static REMOTE_IMAGE_CACHE: Lazy<Mutex<RemoteImageCache>> =
    Lazy::new(|| Mutex::new(RemoteImageCache::default()));
static REMOTE_IMAGE_LOCKS: Lazy<Mutex<HashMap<String, Weak<AsyncMutex<()>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Clone)]
struct CachedImage {
    content_type: String,
    body: Arc<[u8]>,
}

#[derive(Default)]
struct RemoteImageCache {
    images: HashMap<String, CachedImage>,
    order: VecDeque<String>,
    bytes: usize,
}

impl RemoteImageCache {
    fn get(&mut self, source: &str) -> Option<CachedImage> {
        let image = self.images.get(source)?.clone();
        if let Some(position) = self.order.iter().position(|key| key == source) {
            self.order.remove(position);
        }
        self.order.push_back(source.to_string());
        Some(image)
    }

    fn insert(&mut self, source: String, image: CachedImage) {
        let image_bytes = image.body.len();
        if image_bytes > MAX_CACHE_BYTES {
            return;
        }

        if let Some(previous) = self.images.remove(&source) {
            self.bytes = self.bytes.saturating_sub(previous.body.len());
            self.order.retain(|key| key != &source);
        }
        self.bytes += image_bytes;
        self.order.push_back(source.clone());
        self.images.insert(source, image);

        while self.images.len() > MAX_CACHED_IMAGES || self.bytes > MAX_CACHE_BYTES {
            let Some(oldest) = self.order.pop_front() else {
                break;
            };
            if let Some(removed) = self.images.remove(&oldest) {
                self.bytes = self.bytes.saturating_sub(removed.body.len());
            }
        }
    }
}

pub fn handle(
    context: UriSchemeContext<'_, Wry>,
    request: http::Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let source = request
        .uri()
        .path()
        .strip_prefix('/')
        .and_then(percent_decode)
        .unwrap_or_default();
    let state = context.app_handle().state::<AppState>();
    let http = state.http.clone();
    let config = state.config.clone();

    tauri::async_runtime::spawn(async move {
        let response = if source.starts_with("http://") || source.starts_with("https://") {
            let settings = config.get();
            remote_image_response(&http, &settings, &source).await
        } else {
            let path = local_path(&source);
            match std::fs::read(&path) {
                Ok(body) => {
                    let content_type = mime_guess::from_path(&path)
                        .first_or_octet_stream()
                        .essence_str()
                        .to_string();
                    build_response(200, &content_type, body)
                }
                Err(error) => error_response(404, &error.to_string()),
            }
        };
        responder.respond(response);
    });
}

async fn remote_image_response(
    http: &HttpClient,
    settings: &Settings,
    source: &str,
) -> http::Response<Vec<u8>> {
    if let Some(image) = cached_image(source) {
        return cached_response(image);
    }

    // The gallery preloader and the reader can request the same URL at almost
    // the same time. Serialize only identical URLs, then check the cache again
    // so one HTTP response supplies every waiter.
    let request_lock = {
        let mut locks = REMOTE_IMAGE_LOCKS.lock().unwrap();
        locks.retain(|_, lock| lock.strong_count() > 0);
        if let Some(lock) = locks.get(source).and_then(Weak::upgrade) {
            lock
        } else {
            let lock = Arc::new(AsyncMutex::new(()));
            locks.insert(source.to_string(), Arc::downgrade(&lock));
            lock
        }
    };
    let _request_guard = request_lock.lock().await;
    if let Some(image) = cached_image(source) {
        return cached_response(image);
    }

    let _permit = REMOTE_IMAGE_LIMIT
        .acquire()
        .await
        .expect("remote image semaphore closed");
    match http
        .request(reqwest::Method::GET, source, false, settings)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            let content_type = response
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .unwrap_or("application/octet-stream")
                .to_string();
            match response.bytes().await {
                Ok(body) => {
                    if status.is_success() {
                        let image = CachedImage {
                            content_type: content_type.clone(),
                            body: Arc::from(body.as_ref()),
                        };
                        REMOTE_IMAGE_CACHE
                            .lock()
                            .unwrap()
                            .insert(source.to_string(), image);
                    }
                    build_response(status.as_u16(), &content_type, body.to_vec())
                }
                Err(error) => error_response(502, &error.to_string()),
            }
        }
        Err(error) => error_response(502, &error.to_string()),
    }
}

fn cached_image(source: &str) -> Option<CachedImage> {
    REMOTE_IMAGE_CACHE.lock().unwrap().get(source)
}

fn cached_response(image: CachedImage) -> http::Response<Vec<u8>> {
    build_response(200, &image.content_type, image.body.as_ref().to_vec())
}

fn local_path(source: &str) -> PathBuf {
    let path = source.strip_prefix("file://").unwrap_or(source);
    #[cfg(target_os = "windows")]
    let path = path.strip_prefix('/').unwrap_or(path);
    PathBuf::from(path)
}

fn build_response(status: u16, content_type: &str, body: Vec<u8>) -> http::Response<Vec<u8>> {
    http::Response::builder()
        .status(status)
        .header(http::header::CONTENT_TYPE, content_type)
        .header(http::header::CACHE_CONTROL, "private, max-age=3600")
        .header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(body)
        .expect("valid image protocol response")
}

fn error_response(status: u16, message: &str) -> http::Response<Vec<u8>> {
    build_response(
        status,
        "text/plain; charset=utf-8",
        message.as_bytes().to_vec(),
    )
}

fn percent_decode(encoded: &str) -> Option<String> {
    let bytes = encoded.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = *bytes.get(index + 1)?;
            let low = *bytes.get(index + 2)?;
            decoded.push((hex_value(high)? << 4) | hex_value(low)?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{local_path, percent_decode, CachedImage, RemoteImageCache};

    fn cached(body: &[u8]) -> CachedImage {
        CachedImage {
            content_type: "image/jpeg".to_string(),
            body: Arc::from(body),
        }
    }

    #[test]
    fn decodes_remote_url_and_unicode_path() {
        assert_eq!(
            percent_decode("https%3A%2F%2Ft.example%2F%E6%BC%AB%E7%94%BB.jpg"),
            Some("https://t.example/漫画.jpg".to_string())
        );
    }

    #[test]
    fn strips_file_scheme_from_local_paths() {
        let path = local_path("file:///tmp/gallery/001.jpg");
        assert!(path.ends_with("tmp/gallery/001.jpg"));
    }

    #[test]
    fn image_cache_reuses_bytes_and_refreshes_recency() {
        let mut cache = RemoteImageCache::default();
        cache.insert("first".to_string(), cached(&[1, 2, 3]));
        cache.insert("second".to_string(), cached(&[4, 5]));

        assert_eq!(cache.get("first").unwrap().body.as_ref(), &[1, 2, 3]);
        assert_eq!(
            cache.order.iter().map(String::as_str).collect::<Vec<_>>(),
            ["second", "first"]
        );

        cache.insert("first".to_string(), cached(&[9]));
        assert_eq!(cache.bytes, 3);
        assert_eq!(cache.get("first").unwrap().body.as_ref(), &[9]);
    }
}
