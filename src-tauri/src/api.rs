//! nhentai API v2 client.
//!
//! Port of NClientV3's `InspectorV3`, `SimpleGallery.fromV2ListItem`,
//! `GalleryData.parseJSON`, `CommentsFetcher` and `loginapi.User`.
//!
//! Endpoints (relative to `https://<mirror>/api/v2/`):
//! - `galleries?page=N`                       list / popular
//! - `galleries/random`                       random gallery
//! - `galleries/<id>?include=related,favorite` single gallery detail
//! - `search?query=...&page=N&sort=...`       search by query + tags
//! - `favorites?page=N&q=...`                 user favorites (requires auth)
//! - `favorites/random`                       random favorite
//! - `user`                                   current user (requires auth)
//! - `galleries/<id>/comments`                gallery comments
//! - `tags/popular`                           popular tags

use std::sync::Arc;

use chrono::TimeZone;
use serde_json::Value;

use crate::config::{ConfigStore, Language, SortType};
use crate::error::{AppError, AppResult};
use crate::http::HttpClient;
use crate::models::*;

/// The API client. Cheap to clone.
#[derive(Clone)]
pub struct ApiClient {
    pub http: Arc<HttpClient>,
    pub config: Arc<ConfigStore>,
}

impl ApiClient {
    pub fn new(http: Arc<HttpClient>, config: Arc<ConfigStore>) -> Self {
        Self { http, config }
    }

    fn settings(&self) -> crate::config::Settings {
        self.config.get()
    }

    fn api_url(&self, suffix: &str) -> String {
        format!("{}{}", self.config.api_base_url(), suffix.trim_start_matches('/'))
    }

    // ---------------------------------------------------------------------
    // List / browse
    // ---------------------------------------------------------------------

    /// Browse home page. Mirrors `InspectorV3#basicInspector` /
    /// `ApiRequestType.BYALL` / `BYSEARCH` with sort.
    pub async fn browse(&self, page: u32, sort: SortType) -> AppResult<SearchPage> {
        let s = self.settings();
        let lang_tag = language_tag_id(s.only_language);
        // If no sort and no language filter, use the simple `galleries` list.
        if sort.url_addition().is_none() && lang_tag.is_none() {
            let url = self.api_url(&format!("galleries?page={}", page));
            return self.fetch_search(&url).await;
        }
        // Otherwise build a search query with the requested sort.
        let mut q = SearchQuery {
            page,
            sort,
            only_language: s.only_language,
            ..Default::default()
        };
        if let Some(id) = lang_tag {
            q.accepted_tag_ids.push(id);
        }
        self.search(&q).await
    }

    /// Search by query + tags + ranges. Mirrors `InspectorV3#searchInspector`.
    pub async fn search(&self, q: &SearchQuery) -> AppResult<SearchPage> {
        let s = self.settings();
        let base = self.api_url("search?query=");

        // Collect non-empty query tokens, then join with `+`. nhentai rejects a
        // leading `+` (HTTP 400 "query should have at least 1 character"), so we
        // must not prefix the first token — especially when there is no text
        // query and the only filter is a language tag.
        let mut parts: Vec<String> = Vec::new();
        let text = url_encoded(&q.query);
        if !text.is_empty() {
            parts.push(text);
        }

        // Accepted tags (status ACCEPTED): add as `tag:"name"`.
        for t in &q.tags {
            if t.status != TagStatus::Avoided {
                parts.push(percent_encode(&t.to_query_tag_with(TagStatus::Accepted)));
            }
        }
        for t in &q.tags {
            if t.status == TagStatus::Avoided {
                parts.push(percent_encode(&t.to_query_tag_with(TagStatus::Avoided)));
            }
        }
        // Tag IDs from the local DB.
        if !q.accepted_tag_ids.is_empty() || !q.avoided_tag_ids.is_empty() {
            let all = self.fetch_tags_by_ids(
                &q.accepted_tag_ids
                    .iter()
                    .chain(q.avoided_tag_ids.iter())
                    .copied()
                    .collect::<Vec<_>>(),
            )
            .await
            .unwrap_or_default();
            for t in &all {
                let accepted = q.accepted_tag_ids.contains(&t.id);
                parts.push(percent_encode(&t.to_query_tag_with(if accepted {
                    TagStatus::Accepted
                } else {
                    TagStatus::Avoided
                })));
            }
        }
        // Language filter.
        if let Some(id) = language_tag_id(q.only_language.or_all(s.only_language)) {
            // Skip if already covered by accepted_tag_ids (e.g. via browse()).
            if !q.accepted_tag_ids.contains(&id) {
                if let Ok(tags) = self.fetch_tags_by_ids(&[id]).await {
                    if let Some(t) = tags.into_iter().next() {
                        parts.push(percent_encode(&t.to_query_tag_with(TagStatus::Accepted)));
                    }
                }
            }
        }
        // Page range filter (mirrors `Ranges`).
        match (q.from_page, q.to_page) {
            (Some(a), Some(b)) if a == b => {
                parts.push(format!("pages%3A{}", a));
            }
            (Some(a), Some(b)) => {
                parts.push(format!("pages%3A%3E%3D{}+pages%3A%3C%3D{}", a, b));
            }
            (Some(a), None) => parts.push(format!("pages%3A%3E%3D{}", a)),
            (None, Some(b)) => parts.push(format!("pages%3A%3C%3D{}", b)),
            (None, None) => {}
        }

        let mut query = parts.join("+");
        // If the query is empty (no text, no tags, no language filter, no page
        // range), nhentai rejects the request with HTTP 400 "query should have
        // at least 1 character".  Use a dummy negative tag that matches every
        // gallery — exactly what NClientV3 does in `tryByAllPopular()`.
        // See InspectorV3.java:267-272.
        if query.is_empty() {
            query = "-nclientv3".to_string();
        }
        let mut url = format!("{}{}&page={}", base, query, q.page);
        if let Some(srt) = q.sort.url_addition() {
            url.push_str(&format!("&sort={}", srt));
        }
        self.fetch_search(&url).await
    }

    /// Fetch a search URL and parse its result list.
    async fn fetch_search(&self, url: &str) -> AppResult<SearchPage> {
        let s = self.settings();
        let (body, _) = self.http.get_text(url, true, &s).await?;
        let v: Value = serde_json::from_str(&body)?;
        let results = v
            .get("result")
            .ok_or(AppError::InvalidResponse)?
            .as_array()
            .ok_or(AppError::InvalidResponse)?;
        let galleries = results
            .iter()
            .map(|j| simple_gallery_from_v2_list(j, &s.mirror))
            .collect::<Vec<_>>();
        let page = v
            .get("page")
            .and_then(|x| x.as_u64())
            .map(|x| x as u32)
            .unwrap_or(1);
        let num_pages = v
            .get("num_pages")
            .and_then(|x| x.as_u64())
            .map(|x| x as u32)
            .unwrap_or(page.max(1));
        let per_page = v
            .get("per_page")
            .and_then(|x| x.as_u64())
            .map(|x| x as u32)
            .unwrap_or(25);
        Ok(SearchPage {
            galleries,
            page,
            num_pages,
            per_page,
        })
    }

    // ---------------------------------------------------------------------
    // Single gallery detail
    // ---------------------------------------------------------------------

    /// Fetch full gallery detail. Mirrors `InspectorV3#doSingleV2`.
    pub async fn gallery(&self, id: i64) -> AppResult<Gallery> {
        let s = self.settings();
        let url = self.api_url(&format!("galleries/{}?include=related,favorite", id));
        let (body, _) = self.http.get_text(&url, true, &s).await?;
        parse_gallery(&body, &s.mirror)
    }

    /// Random gallery. Mirrors `ApiRequestType.RANDOM`.
    pub async fn random(&self) -> AppResult<Gallery> {
        let s = self.settings();
        let url = self.api_url("galleries/random");
        let (body, _) = self.http.get_text(&url, true, &s).await?;
        // Random endpoint returns just `{"id": N}`; fetch detail.
        let v: Value = serde_json::from_str(&body)?;
        if let Some(id) = v.get("id").and_then(|i| i.as_i64()) {
            return self.gallery(id).await;
        }
        parse_gallery(&body, &s.mirror)
    }

    /// Random favorite. Requires auth.
    pub async fn random_favorite(&self) -> AppResult<Gallery> {
        let s = self.settings();
        let url = self.api_url("favorites/random");
        let (body, _) = self.http.get_text(&url, true, &s).await?;
        let v: Value = serde_json::from_str(&body)?;
        if let Some(id) = v.get("id").and_then(|i| i.as_i64()) {
            return self.gallery(id).await;
        }
        Err(AppError::InvalidResponse)
    }

    // ---------------------------------------------------------------------
    // Favorites
    // ---------------------------------------------------------------------

    /// Fetch a page of online favorites. Mirrors `ApiRequestType.FAVORITE`.
    pub async fn favorites_page(
        &self,
        page: u32,
        query: Option<&str>,
    ) -> AppResult<FavoritesPage> {
        let s = self.settings();
        let mut url = format!("{}favorites?page={}", self.config.api_base_url(), page);
        if let Some(q) = query {
            if !q.trim().is_empty() {
                url.push_str(&format!("&q={}", percent_encode(q.trim())));
            }
        }
        let (body, _) = self.http.get_text(&url, true, &s).await?;
        let v: Value = serde_json::from_str(&body)?;
        let results = v
            .get("result")
            .and_then(|x| x.as_array())
            .ok_or(AppError::InvalidResponse)?;
        let galleries = results
            .iter()
            .map(|j| simple_gallery_from_v2_list(j, &s.mirror))
            .collect::<Vec<_>>();
        let num_pages = v
            .get("num_pages")
            .and_then(|x| x.as_u64())
            .map(|x| x as u32)
            .unwrap_or(page.max(1));
        Ok(FavoritesPage {
            galleries,
            page,
            num_pages,
        })
    }

    // ---------------------------------------------------------------------
    // User / comments
    // ---------------------------------------------------------------------

    /// Current user. Mirrors `loginapi.User.createUser`.
    pub async fn user(&self) -> AppResult<User> {
        let s = self.settings();
        let url = self.api_url("user");
        let (body, _) = self.http.get_text(&url, true, &s).await?;
        let v: Value = serde_json::from_str(&body)?;
        Ok(User {
            id: v
                .get("id")
                .and_then(|x| x.as_i64())
                .ok_or(AppError::InvalidResponse)?,
            username: v
                .get("username")
                .and_then(|x| x.as_str())
                .ok_or(AppError::InvalidResponse)?
                .to_string(),
            slug: v.get("slug").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            avatar_url: v
                .get("avatar_url")
                .and_then(|x| x.as_str())
                .map(String::from),
            is_superuser: v.get("is_superuser").and_then(|x| x.as_bool()).unwrap_or(false),
            is_staff: v.get("is_staff").and_then(|x| x.as_bool()).unwrap_or(false),
        })
    }

    /// Gallery comments. Mirrors `CommentsFetcher`.
    pub async fn comments(&self, gallery_id: i64) -> AppResult<CommentsPage> {
        let s = self.settings();
        let url = self.api_url(&format!("galleries/{}/comments", gallery_id));
        let (body, _) = self.http.get_text(&url, true, &s).await?;
        let v: Value = serde_json::from_str(&body)?;
        let arr = v.as_array().cloned().unwrap_or_default();
        let comments = arr.iter().map(parse_comment).collect::<Vec<_>>();
        Ok(CommentsPage {
            comments,
            gallery_id,
        })
    }

    // ---------------------------------------------------------------------
    // Tags
    // ---------------------------------------------------------------------

    /// Popular tags (cached on first run via `tags.json` shipped with the app).
    pub async fn popular_tags(&self) -> AppResult<Vec<Tag>> {
        let s = self.settings();
        let url = self.api_url("tags/popular");
        let (body, _) = match self.http.get_text(&url, true, &s).await {
            Ok(x) => x,
            Err(e) => {
                log::warn!("popular tags fetch failed: {e}");
                return Ok(vec![]);
            }
        };
        let v: Value = serde_json::from_str(&body).unwrap_or(Value::Array(vec![]));
        let arr = match v {
            Value::Array(a) => a,
            Value::Object(_) => vec![v],
            _ => vec![],
        };
        Ok(arr.iter().map(parse_tag).collect())
    }

    /// Fetch full tag objects for a list of IDs. The list endpoint does not
    /// support multi-fetch; we use a search trick (`include tag` would be
    /// ideal but is rate-limited). Falls back to the local DB when available.
    pub async fn fetch_tags_by_ids(&self, ids: &[i64]) -> AppResult<Vec<Tag>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        // Try local DB first (most reliable, no network).
        let db_tags = crate::db::tags_get_by_ids(ids).unwrap_or_default();
        if db_tags.len() == ids.len() {
            return Ok(db_tags);
        }
        // Fetch each missing tag from the API.
        let mut out = db_tags;
        let known: std::collections::HashSet<i64> = out.iter().map(|t| t.id).collect();
        for id in ids {
            if known.contains(id) {
                continue;
            }
            if let Ok(t) = self.fetch_tag_by_id(*id).await {
                out.push(t);
            }
        }
        Ok(out)
    }

    pub async fn fetch_tag_by_id(&self, id: i64) -> AppResult<Tag> {
        let s = self.settings();
        let url = self.api_url(&format!("tags/{}", id));
        let (body, _) = self.http.get_text(&url, true, &s).await?;
        let v: Value = serde_json::from_str(&body)?;
        Ok(parse_tag(&v))
    }

    /// All tags (shipped data file fallback). Used by the tag picker UI.
    pub async fn all_tags(&self) -> AppResult<Vec<Tag>> {
        // The full tag list is huge (~8MB); on desktop we just expose the
        // popular ones plus whatever is cached locally.
        self.popular_tags().await
    }

    /// Search tags by name. Mirrors NClientV3's tag autocomplete.
    pub async fn search_tags(&self, query: &str, limit: usize) -> AppResult<Vec<Tag>> {
        let q = query.trim().to_ascii_lowercase();
        if q.is_empty() {
            return self.popular_tags().await;
        }
        // Try the local DB first (instant).
        let local = crate::db::tags_search(&q, limit).unwrap_or_default();
        if local.len() >= limit.min(5) {
            return Ok(local.into_iter().take(limit).collect());
        }
        // Fall back to API search.
        let s = self.settings();
        let url = self.api_url(&format!("tags?search={}&page=1", percent_encode(&q)));
        match self.http.get_text(&url, true, &s).await {
            Ok((body, _)) => {
                let v: Value = serde_json::from_str(&body).unwrap_or(Value::Null);
                let arr = v.get("result").and_then(|x| x.as_array());
                if let Some(arr) = arr {
                    let tags: Vec<Tag> = arr.iter().map(parse_tag).collect();
                    for t in &tags {
                        let _ = crate::db::tag_upsert(t);
                    }
                    Ok(tags.into_iter().take(limit).collect())
                } else {
                    Ok(local)
                }
            }
            Err(_) => Ok(local),
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers / parsers
// ---------------------------------------------------------------------------

/// Build a `SimpleGallery` from a v2 search/list item.
/// Mirrors `SimpleGallery.fromV2ListItem`.
pub fn simple_gallery_from_v2_list(j: &Value, host: &str) -> SimpleGallery {
    let id = j.get("id").and_then(|x| x.as_i64()).unwrap_or(0);
    let media_id = j
        .get("media_id")
        .and_then(|x| x.as_str())
        .and_then(|s| s.parse::<i64>().ok())
        .or_else(|| j.get("media_id").and_then(|x| x.as_i64()))
        .unwrap_or(0);

    // Title: prefer `title.pretty` -> `title.english` -> `english_title` -> `japanese_title`.
    let title = if let Some(t) = j.get("title") {
        let pretty = t.get("pretty").and_then(|x| x.as_str()).unwrap_or("");
        let english = t.get("english").and_then(|x| x.as_str()).unwrap_or("");
        let japanese = t.get("japanese").and_then(|x| x.as_str()).unwrap_or("");
        let pick = if !pretty.is_empty() {
            pretty
        } else if !english.is_empty() {
            english
        } else {
            japanese
        };
        pick.to_string()
    } else {
        let en = j.get("english_title").and_then(|x| x.as_str()).unwrap_or("");
        let jp = j.get("japanese_title").and_then(|x| x.as_str()).unwrap_or("");
        if !en.is_empty() {
            en.to_string()
        } else {
            jp.to_string()
        }
    };

    let thumb_path = j
        .get("thumbnail")
        .and_then(|x| {
            x.as_str()
                .map(String::from)
                .or_else(|| x.get("path").and_then(|p| p.as_str()).map(String::from))
        })
        .unwrap_or_default();
    let thumbnail = if thumb_path.is_empty() {
        None
    } else if thumb_path.starts_with("http") {
        Some(thumb_path)
    } else {
        Some(format!("https://t1.{}/{}", host, thumb_path.trim_start_matches('/')))
    };

    let num_pages = j
        .get("num_pages")
        .and_then(|x| x.as_u64())
        .map(|x| x as usize)
        .unwrap_or(0);

    // Tags: detail/related items may carry full tag objects; list items only
    // carry `tag_ids`. We resolve names lazily.
    let mut tags = Vec::new();
    if let Some(arr) = j.get("tags").and_then(|x| x.as_array()) {
        for t in arr {
            tags.push(parse_tag(t));
        }
    }
    let language = infer_language_from_tags(&tags);

    SimpleGallery {
        id,
        media_id,
        title,
        thumbnail,
        language,
        tags,
        num_pages,
    }
}

/// Parse a full gallery JSON object. Mirrors `GalleryData#parseJSON`.
pub fn parse_gallery(body: &str, host: &str) -> AppResult<Gallery> {
    let v: Value = serde_json::from_str(body)?;
    if v.get("error").is_some() {
        return Err(AppError::NotFound);
    }
    let id = v
        .get("id")
        .and_then(|x| x.as_i64())
        .ok_or(AppError::InvalidResponse)?;
    let media_id = v
        .get("media_id")
        .and_then(|x| x.as_str())
        .and_then(|s| s.parse::<i64>().ok())
        .or_else(|| v.get("media_id").and_then(|x| x.as_i64()))
        .unwrap_or(0);

    let titles = Titles {
        english: v
            .pointer("/title/english")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string(),
        pretty: v
            .pointer("/title/pretty")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string(),
        japanese: v
            .pointer("/title/japanese")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string(),
    };

    let upload_date = v
        .get("upload_date")
        .and_then(|x| x.as_i64())
        .and_then(|s| chrono::Utc.timestamp_opt(s, 0).single());
    let num_favorites = v
        .get("num_favorites")
        .and_then(|x| x.as_i64())
        .unwrap_or(0);

    let mut tags = Vec::new();
    if let Some(arr) = v.get("tags").and_then(|x| x.as_array()) {
        for t in arr {
            tags.push(parse_tag(t));
        }
    }

    let cover = parse_page(v.get("cover"), host, "cover", "t1");
    let thumbnail = parse_page(v.get("thumbnail"), host, "thumbnail", "t1");

    let mut pages = Vec::new();
    if let Some(arr) = v.get("pages").and_then(|x| x.as_array()) {
        for (i, p) in arr.iter().enumerate() {
            pages.push(parse_page(Some(p), host, &format!("page {}", i), "i1").with_index(i));
        }
    }
    let num_pages = v
        .get("num_pages")
        .and_then(|x| x.as_u64())
        .map(|x| x as usize)
        .unwrap_or(pages.len());

    let is_favorited = v
        .get("is_favorited")
        .and_then(|x| x.as_bool())
        .unwrap_or(false);

    let related = v
        .get("related")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .map(|j| simple_gallery_from_v2_list(j, host))
                .collect()
        })
        .unwrap_or_default();

    Ok(Gallery {
        id,
        media_id,
        upload_date,
        num_favorites,
        num_pages,
        titles,
        tags,
        cover,
        thumbnail,
        pages,
        is_favorited,
        related,
    })
}

fn parse_tag(v: &Value) -> Tag {
    Tag {
        id: v.get("id").and_then(|x| x.as_i64()).unwrap_or(0),
        name: v
            .get("name")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string(),
        tag_type: TagType::from_name(
            v.get("type")
                .and_then(|x| x.as_str())
                .unwrap_or("tag"),
        ),
        count: v.get("count").and_then(|x| x.as_i64()).unwrap_or(0),
        status: TagStatus::Default,
    }
}

fn parse_comment(v: &Value) -> Comment {
    let poster = v
        .get("poster")
        .map(|p| CommentUser {
            id: p.get("id").and_then(|x| x.as_i64()).unwrap_or(0),
            username: p
                .get("username")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            slug: p.get("slug").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            avatar_url: p
                .get("avatar_url")
                .and_then(|x| x.as_str())
                .map(String::from),
            is_superuser: p.get("is_superuser").and_then(|x| x.as_bool()).unwrap_or(false),
            is_staff: p.get("is_staff").and_then(|x| x.as_bool()).unwrap_or(false),
        })
        .unwrap_or_else(|| CommentUser {
            id: 0,
            username: "anonymous".into(),
            slug: String::new(),
            avatar_url: None,
            is_superuser: false,
            is_staff: false,
        });
    Comment {
        id: v.get("id").and_then(|x| x.as_i64()).unwrap_or(0),
        gallery_id: v.get("gallery_id").and_then(|x| x.as_i64()).unwrap_or(0),
        poster,
        body: v
            .get("body")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string(),
        create_date: v
            .get("create_date")
            .and_then(|x| x.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.with_timezone(&chrono::Utc)),
        post_date: v
            .get("post_date")
            .and_then(|x| x.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.with_timezone(&chrono::Utc)),
        vote: v.get("vote").and_then(|x| x.as_i64()).map(|x| x as i32),
    }
}

fn parse_page(v: Option<&Value>, host: &str, _debug: &str, prefix_if_missing: &str) -> Page {
    let (path, thumb, w, h) = match v {
        Some(obj) => {
            let raw_path = obj.get("path").and_then(|x| x.as_str()).unwrap_or("");
            let path = absolutize(raw_path, host, prefix_if_missing);
            let thumb = obj
                .get("thumbnail")
                .and_then(|x| x.as_str())
                .map(|s| absolutize(s, host, "t1"));
            let w = obj.get("width").and_then(|x| x.as_i64()).unwrap_or(0);
            let h = obj.get("height").and_then(|x| x.as_i64()).unwrap_or(0);
            (Some(path), thumb, w, h)
        }
        None => (None, None, 0, 0),
    };
    Page {
        index: 0,
        path,
        thumbnail: thumb,
        width: w,
        height: h,
    }
}

impl Page {
    fn with_index(mut self, index: usize) -> Self {
        self.index = index;
        self
    }
}

fn absolutize(path: &str, host: &str, prefix: &str) -> String {
    if path.starts_with("http") {
        path.to_string()
    } else {
        format!("https://{}.{}/{}", prefix, host, path.trim_start_matches('/'))
    }
}

fn infer_language_from_tags(tags: &[Tag]) -> Language {
    for t in tags {
        if t.tag_type == TagType::Language {
            match t.name.to_ascii_lowercase().as_str() {
                "japanese" => return Language::Japanese,
                "english" => return Language::English,
                "chinese" => return Language::Chinese,
                _ => {}
            }
        }
    }
    Language::All
}

fn language_tag_id(lang: Language) -> Option<i64> {
    match lang {
        Language::All => None,
        Language::English => Some(special_tag_ids::LANGUAGE_ENGLISH),
        Language::Japanese => Some(special_tag_ids::LANGUAGE_JAPANESE),
        Language::Chinese => Some(special_tag_ids::LANGUAGE_CHINESE),
    }
}

/// Encode a value for use in a URL query segment. Covers everything NClientV3
/// relies on (`+`, `"`, spaces, etc.).
fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

fn url_encoded(s: &str) -> String {
    let mut out = percent_encode(s);
    // nhentai search uses `+` for spaces; %20 also works but matches the
    // original app's behaviour.
    out = out.replace("%20", "+");
    out
}

/// Local helper trait to make language fallback read nicely.
trait LanguageExt {
    fn or_all(self, other: Language) -> Language;
}
impl LanguageExt for Language {
    fn or_all(self, other: Language) -> Language {
        if self == Language::All {
            other
        } else {
            self
        }
    }
}
