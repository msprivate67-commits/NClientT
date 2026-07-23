//! HTTP client with persistent cookies, custom User-Agent, API key auth,
//! and Cloudflare challenge detection.
//!
//! This module plays the role of NClientV3's `Global#initHttpClient()` +
//! `ApiAuthInterceptor` + `CustomCookieJar` (which uses
//! `franmontiel/PersistentCookieJar`). Cookies are persisted to a JSON file so
//! that `cf_clearance` and session cookies survive restarts.

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, REFERER};
use reqwest::{Client, ClientBuilder, StatusCode};
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};

use crate::config::{ConfigStore, ProxyType};
use crate::error::{AppError, AppResult};

/// Headers returned alongside a response body, useful for diagnostics.
#[derive(Debug, Clone, Default)]
pub struct ResponseInfo {
    pub status: u16,
    pub url: String,
    pub final_url: String,
}

/// The application-wide HTTP client. Cheaply cloneable internally because
/// both `reqwest::Client` and the cookie store are `Arc`-wrapped.
pub struct HttpClient {
    inner: RwLock<Client>,
    cookie_store: Arc<CookieStoreMutex>,
    cookie_path: PathBuf,
    /// Most recent mirror / UA / timeout / proxy, so we know when to rebuild the client.
    fingerprint: RwLock<ClientFingerprint>,
}

/// Fields that, when changed, require the HTTP client to be rebuilt.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ClientFingerprint {
    mirror: String,
    user_agent: String,
    timeout_secs: u64,
    proxy_type: ProxyType,
    proxy_host: String,
    proxy_port: u16,
    proxy_username: String,
    proxy_password: String,
}

impl HttpClient {
    /// Create the client, loading (or creating) the persistent cookie jar at
    /// `<app_data>/cookies.json`.
    pub fn new(config: &ConfigStore) -> Self {
        let cookie_path = config
            .path_of_setting()
            .and_then(|p| p.parent().map(Path::to_path_buf))
            .map(|d| d.join("cookies.json"))
            .unwrap_or_else(|| PathBuf::from("cookies.json"));

        let cookie_store = load_or_create_cookie_store(&cookie_path);
        let s = config.get();
        let ua = effective_ua(&s.user_agent);
        let client = build_client(&cookie_store, &ua, s.request_timeout_secs, &s);

        Self {
            inner: RwLock::new(client),
            cookie_store,
            cookie_path,
            fingerprint: RwLock::new(ClientFingerprint::from_settings(&s)),
        }
    }

    /// Rebuild the underlying client (mirror / UA / timeout / proxy changed).
    /// The cookie jar is preserved so cf_clearance survives.
    pub fn rebuild(&self, settings: &crate::config::Settings) {
        let new_fp = ClientFingerprint::from_settings(settings);
        {
            let mut fp = self.fingerprint.write().unwrap();
            if *fp == new_fp {
                return;
            }
            *fp = new_fp;
        }
        let ua = effective_ua(&settings.user_agent);
        let client = build_client(
            &self.cookie_store,
            &ua,
            settings.request_timeout_secs,
            settings,
        );
        *self.inner.write().unwrap() = client;
        log::info!("http client rebuilt for host={}", settings.mirror);
    }

    /// Build a `GET` request with the right headers. `is_api` should be true
    /// when the URL targets `/api/v2/` so the `Authorization: Key <api_key>`
    /// header (mirroring `ApiAuthInterceptor`) is attached.
    pub fn request(
        &self,
        method: reqwest::Method,
        url: &str,
        is_api: bool,
        settings: &crate::config::Settings,
    ) -> reqwest::RequestBuilder {
        let client = self.inner.read().unwrap().clone();
        let mut builder = client.request(method, url);

        let mut headers = HeaderMap::new();
        headers.insert(
            REFERER,
            HeaderValue::try_from(settings.mirror.as_str())
                .unwrap_or(HeaderValue::from_static("https://nhentai.net/")),
        );
        if is_api {
            if let Some(auth) = settings.auth.authorization_header() {
                if let Ok(val) = HeaderValue::from_str(&auth) {
                    headers.insert(AUTHORIZATION, val);
                }
            }
        }
        builder = builder.headers(headers);
        builder
    }

    /// Convenience GET that returns the body as text. Returns
    /// [`AppError::Cloudflare`] when a CF interstitial is detected.
    pub async fn get_text(
        &self,
        url: &str,
        is_api: bool,
        settings: &crate::config::Settings,
    ) -> AppResult<(String, ResponseInfo)> {
        let resp = self
            .request(reqwest::Method::GET, url, is_api, settings)
            .send()
            .await?;
        let status = resp.status();
        let final_url = resp.url().to_string();

        // Cloudflare detection: 503 with a server header, or a 403 with the
        // challenge body, or `cf-mitigated: challenge`.
        if is_cloudflare(&resp) {
            return Err(AppError::Cloudflare);
        }
        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            // Auth failures on API endpoints surface as Unauthorized so the
            // frontend can prompt to re-enter the API key.
            if is_api {
                return Err(AppError::Unauthorized);
            }
        }

        let body = resp.text().await?;
        if status == StatusCode::NOT_FOUND {
            return Err(AppError::NotFound);
        }
        if !status.is_success() {
            return Err(AppError::Http {
                status: status.as_u16(),
                body,
            });
        }
        Ok((
            body,
            ResponseInfo {
                status: status.as_u16(),
                url: url.to_string(),
                final_url,
            },
        ))
    }

    /// Convenience GET that streams the response. Used by the download manager.
    pub async fn get_stream(
        &self,
        url: &str,
        settings: &crate::config::Settings,
    ) -> AppResult<reqwest::Response> {
        let resp = self
            .request(reqwest::Method::GET, url, false, settings)
            .send()
            .await?;
        if is_cloudflare(&resp) {
            return Err(AppError::Cloudflare);
        }
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Http {
                status: status.as_u16(),
                body,
            });
        }
        Ok(resp)
    }

    pub fn cookie_store(&self) -> Arc<CookieStoreMutex> {
        self.cookie_store.clone()
    }

    /// Wipe the cookie jar and persist it. Mirrors the "clear cookies" UI.
    pub fn clear_cookies(&self) -> AppResult<()> {
        {
            let mut store = self.cookie_store.lock().unwrap();
            store.clear();
        }
        self.persist_cookies();
        Ok(())
    }

    /// Persist the cookie jar to disk (no-op if path missing).
    pub fn persist_cookies(&self) {
        let store = self.cookie_store.lock().unwrap();
        save_cookie_store(&store, &self.cookie_path);
    }

    /// Inject a raw cookie string (e.g. captured by the Cloudflare webview)
    /// into the jar for the given URL. `cookie_str` may be either a single
    /// `name=value` pair or a full `Set-Cookie:` header.
    pub fn set_cookie_str(&self, url: &str, cookie_str: &str) -> AppResult<()> {
        let request_url = url::Url::parse(url)?;
        let raw = cookie_str.trim();
        // Strip an optional leading `Set-Cookie:`.
        let body = raw
            .strip_prefix("Set-Cookie:")
            .or_else(|| raw.strip_prefix("set-cookie:"))
            .unwrap_or(raw)
            .trim();
        let pair = match body.split_once(';') {
            Some((p, _)) => p.trim(),
            None => body,
        };
        let mut store = self.cookie_store.lock().unwrap();
        if let Err(e) = store.parse(pair, &request_url) {
            log::warn!("failed to parse cookie '{}': {}", pair, e);
        }
        drop(store);
        self.persist_cookies();
        Ok(())
    }

    /// Snapshot all cookies for a URL as `name1=value1; name2=value2`.
    pub fn cookies_for(&self, url: &str) -> String {
        let Ok(u) = url::Url::parse(url) else {
            return String::new();
        };
        let store = self.cookie_store.lock().unwrap();
        // `CookieStore::get_request_values` returns `Vec<(name, value)>` which
        // we join into a Cookie header-style string.
        store
            .get_request_values(&u)
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("; ")
    }
}

/// Build a `reqwest::Client` with our cookie jar and headers.
fn build_client(
    cookie_store: &Arc<CookieStoreMutex>,
    user_agent: &str,
    timeout_secs: u64,
    settings: &crate::config::Settings,
) -> Client {
    let ua = effective_ua(user_agent);
    let mut builder = ClientBuilder::new()
        .cookie_provider(cookie_store.clone())
        // The persistent store handles cookie persistence; do not also use the
        // in-memory-only `cookies(true)` shortcut.
        .user_agent(&ua)
        .timeout(Duration::from_secs(timeout_secs))
        .connect_timeout(Duration::from_secs(20))
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(32)
        .gzip(true)
        .brotli(true)
        .deflate(true);

    // Apply proxy if configured explicitly by the user.
    if let Some(proxy) = build_proxy(settings) {
        builder = builder.proxy(proxy);
    } else if let Some(proxy) = detect_system_proxy() {
        // No explicit proxy in settings: fall back to the OS-level proxy
        // (Windows Internet Settings / `HTTP_PROXY` env). Without this,
        // reqwest only consults env vars and ignores the Windows registry,
        // so users behind a system-wide VPN/proxy silently fail to connect.
        builder = builder.proxy(proxy);
    }

    // We embed rustls so the app does not depend on platform OpenSSL.
    builder.build().expect("failed to build reqwest client")
}

/// Resolve the effective User-Agent: override if non-empty, otherwise default.
fn effective_ua(raw: &str) -> String {
    if raw.trim().is_empty() {
        crate::config::DEFAULT_UA.to_string()
    } else {
        raw.trim().to_string()
    }
}

/// Build a `reqwest::Proxy` from settings, returning `None` when proxy is disabled.
fn build_proxy(settings: &crate::config::Settings) -> Option<reqwest::Proxy> {
    use crate::config::ProxyType;

    if matches!(settings.proxy_type, ProxyType::None) {
        return None;
    }
    let host = settings.proxy_host.trim();
    if host.is_empty() {
        return None;
    }
    let port = settings.proxy_port;

    // Build proxy URL: scheme://[user:pass@]host:port
    let scheme = match settings.proxy_type {
        ProxyType::Http => "http",
        ProxyType::Socks5 => "socks5h",
        ProxyType::None => unreachable!(),
    };

    let user = settings.proxy_username.trim();
    let pass = settings.proxy_password.trim();
    let authority = if !user.is_empty() {
        if !pass.is_empty() {
            format!("{}:{}@{}:{}", user, pass, host, port)
        } else {
            format!("{}@{}:{}", user, host, port)
        }
    } else {
        format!("{}:{}", host, port)
    };

    let proxy_url = format!("{}://{}", scheme, authority);
    match reqwest::Proxy::all(&proxy_url) {
        Ok(p) => {
            log::info!("using proxy: {}://{}:{}", scheme, host, port);
            Some(p)
        }
        Err(e) => {
            log::warn!("invalid proxy URL '{}': {}", proxy_url, e);
            None
        }
    }
}

/// Detect an OS-level proxy when the user has not configured one in Settings.
///
/// reqwest only honours the `HTTP_PROXY` / `HTTPS_PROXY` / `ALL_PROXY` env
/// vars by default; on Windows most proxy tools (V2Ray, Xray, Clash, ...)
/// expose themselves via the IE/WinINET registry settings instead. Users in
/// regions where the target site is blocked rely on those tools, so we read
/// the registry ourselves and fall back to env vars on non-Windows platforms.
fn detect_system_proxy() -> Option<reqwest::Proxy> {
    if let Some(url) = read_windows_system_proxy() {
        return build_sys_proxy(&url, "Windows Internet Settings");
    }
    // Non-Windows / registry unavailable: reqwest already wires up env-var
    // proxies via its default `with_system_proxy` behaviour, so we have nothing
    // to add here.
    None
}

/// Build a `reqwest::Proxy` from a raw system proxy URL, logging the source.
fn build_sys_proxy(raw: &str, source: &str) -> Option<reqwest::Proxy> {
    let url = raw.trim();
    if url.is_empty() {
        return None;
    }
    match reqwest::Proxy::all(url) {
        Ok(p) => {
            log::info!("using system proxy ({}): {}", source, url);
            Some(p)
        }
        Err(e) => {
            log::warn!("invalid system proxy '{}' ({}): {}", url, source, e);
            None
        }
    }
}

/// Read the Windows WinINET proxy setting from the registry.
///
/// Returns `http://host:port` when `ProxyEnable == 1` and `ProxyServer` is set.
/// Mirrors what `WinINET` itself does, and what Chrome/Edge/Firefox read as the
/// "system proxy". On non-Windows targets this is a compile-time no-op.
#[cfg(windows)]
fn read_windows_system_proxy() -> Option<String> {
    use windows_registry::CURRENT_USER;

    let key = CURRENT_USER
        .open("Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings")
        .ok()?;

    // ProxyEnable (DWORD): 1 => a manual proxy is configured.
    let enabled: u32 = key
        .get_value("ProxyEnable")
        .ok()
        .and_then(|v| v.try_into().ok())
        .unwrap_or(0);
    if enabled != 1 {
        return None;
    }

    // ProxyServer (REG_SZ): "host:port" or "http=host:port;https=host:port".
    let raw: String = key
        .get_value("ProxyServer")
        .ok()
        .and_then(|v| v.try_into().ok())?;
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    // WinINET lets users specify per-scheme proxies as "http=h:port;https=h:port".
    // Pick the `http=`/`https=` entry if present, otherwise treat the whole
    // value as a single "host:port" string.
    let chosen = raw
        .split(';')
        .map(str::trim)
        .find(|entry| entry.starts_with("http=") || entry.starts_with("https="))
        .map(|entry| entry.split('=').nth(1).unwrap_or("").trim())
        .unwrap_or(raw);

    if chosen.is_empty() {
        return None;
    }
    // Normalise to a full URL. The registry rarely stores the scheme, so add
    // `http://` when only `host:port` is present. `socks=` entries are skipped
    // because reqwest needs an explicit scheme we cannot reliably guess.
    if chosen.contains("://") {
        Some(chosen.to_string())
    } else {
        Some(format!("http://{}", chosen))
    }
}

#[cfg(not(windows))]
fn read_windows_system_proxy() -> Option<String> {
    None
}

impl ClientFingerprint {
    fn from_settings(s: &crate::config::Settings) -> Self {
        Self {
            mirror: s.mirror.clone(),
            user_agent: effective_ua(&s.user_agent),
            timeout_secs: s.request_timeout_secs,
            proxy_type: s.proxy_type,
            proxy_host: s.proxy_host.clone(),
            proxy_port: s.proxy_port,
            proxy_username: s.proxy_username.clone(),
            proxy_password: s.proxy_password.clone(),
        }
    }
}

/// Detect a Cloudflare interstitial. Mirrors NClientV3's behaviour, which
/// surfaces CF pages as `InvalidResponseException` and dispatches the WebView.
fn is_cloudflare(resp: &reqwest::Response) -> bool {
    if resp.headers().contains_key("cf-mitigated") {
        return true;
    }
    if let Some(server) = resp.headers().get(reqwest::header::SERVER) {
        if let Ok(s) = server.to_str() {
            if s.to_ascii_lowercase().contains("cloudflare")
                && (resp.status() == StatusCode::FORBIDDEN
                    || resp.status() == StatusCode::SERVICE_UNAVAILABLE
                    || resp.status() == StatusCode::TOO_MANY_REQUESTS)
            {
                return true;
            }
        }
    }
    false
}

/// Load or create the persistent cookie jar.
fn load_or_create_cookie_store(path: &Path) -> Arc<CookieStoreMutex> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let store = match std::fs::File::open(path) {
        Ok(f) => {
            let reader = std::io::BufReader::new(f);
            #[allow(deprecated)]
            CookieStore::load_json(reader).unwrap_or_else(|e| {
                log::warn!("failed to load cookies.json ({e}); starting fresh");
                CookieStore::default()
            })
        }
        Err(_) => CookieStore::default(),
    };
    Arc::new(CookieStoreMutex::new(store))
}

/// Persist the cookie jar to disk as JSON.
fn save_cookie_store(store: &CookieStore, path: &Path) {
    let mut buf = Vec::new();
    #[allow(deprecated)]
    if let Err(e) = store.save_json(&mut std::io::BufWriter::new(&mut buf)) {
        log::warn!("failed to serialize cookies: {e}");
        return;
    }
    let _ = std::fs::write(path, &buf);
}
