//! Image protocol shared by local-library and remote gallery images.
//!
//! Renderer-side HTTP requests do not use the proxy configured for reqwest,
//! and the built-in asset protocol is not consistent across Linux WebKitGTK
//! versions. Routing both kinds of images through one custom protocol keeps
//! image loading on the Rust side, where proxy, cookies and headers are known.

use std::path::PathBuf;

use reqwest::header::CONTENT_TYPE;
use tauri::{http, Manager, UriSchemeContext, UriSchemeResponder, Wry};
use tokio::sync::Semaphore;

use crate::AppState;

pub const SCHEME: &str = "nclient-image";
const MAX_REMOTE_IMAGE_REQUESTS: usize = 3;
static REMOTE_IMAGE_LIMIT: Semaphore = Semaphore::const_new(MAX_REMOTE_IMAGE_REQUESTS);

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
            let _permit = REMOTE_IMAGE_LIMIT
                .acquire()
                .await
                .expect("remote image semaphore closed");
            let settings = config.get();
            match http
                .request(reqwest::Method::GET, &source, false, &settings)
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
                        Ok(body) => build_response(status.as_u16(), &content_type, body.to_vec()),
                        Err(error) => error_response(502, &error.to_string()),
                    }
                }
                Err(error) => error_response(502, &error.to_string()),
            }
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
    use super::{local_path, percent_decode};

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
}
