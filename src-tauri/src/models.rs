//! Serializable data structures mirrored from NClientV3's `api/components` and
//! `api/enums` packages.
//!
//! These types are used for two things:
//! 1. Deserializing nhentai's API v2 JSON responses.
//! 2. Returning data to the frontend via Tauri commands.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::{Language, SortType, TitleType};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Mirrors `TagType`. The `single` form is what nhentai uses for both tag
/// queries (`tag:"foo"`) and JSON `"type"` fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TagType {
    Unknown,
    Parody,
    Character,
    Tag,
    Artist,
    Group,
    Language,
    Category,
}

impl TagType {
    pub fn from_name(name: &str) -> Self {
        match name {
            "parody" => Self::Parody,
            "character" => Self::Character,
            "tag" => Self::Tag,
            "artist" => Self::Artist,
            "group" => Self::Group,
            "language" => Self::Language,
            "category" => Self::Category,
            _ => Self::Unknown,
        }
    }

    pub fn single(self) -> &'static str {
        match self {
            Self::Unknown => "",
            Self::Parody => "parody",
            Self::Character => "character",
            Self::Tag => "tag",
            Self::Artist => "artist",
            Self::Group => "group",
            Self::Language => "language",
            Self::Category => "category",
        }
    }
}

/// Mirrors `TagStatus`. Stored in the local DB so users can mark tags as
/// always-include / always-exclude.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TagStatus {
    #[default]
    Default,
    Accepted,
    Avoided,
}

/// Special tag IDs from `SpecialTagIds`.
pub mod special_tag_ids {
    pub const LANGUAGE_JAPANESE: i64 = 6346;
    pub const LANGUAGE_ENGLISH: i64 = 12227;
    pub const LANGUAGE_CHINESE: i64 = 29963;
    pub const INVALID_ID: i64 = -1;
}

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// A tag (artist / character / language / ...). Mirrors `Tag`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub tag_type: TagType,
    pub count: i64,
    /// Local-only: a user-set status. Defaults to `default`.
    #[serde(default)]
    pub status: TagStatus,
}

impl Tag {
    /// Mirrors `Tag#toQueryTag(status)`. Produces e.g. `tag:"foo"` or
    /// `-tag:"foo"` when avoided.
    pub fn to_query_tag_with(&self, status: TagStatus) -> String {
        let prefix = if status == TagStatus::Avoided { "-" } else { "" };
        format!(
            "{}{}:\"{}\"",
            prefix,
            self.tag_type.single(),
            self.name
        )
    }

    pub fn to_query_tag(&self) -> String {
        self.to_query_tag_with(self.status)
    }
}

/// A single page image. Mirrors `Page`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub index: usize,
    pub path: Option<String>,
    pub thumbnail: Option<String>,
    pub width: i64,
    pub height: i64,
}

impl Page {
    pub fn thumbnail_or_path(&self) -> Option<&str> {
        self.thumbnail.as_deref().or(self.path.as_deref())
    }
}

/// Gallery titles in three languages. Mirrors `TitleType` / `GalleryData#titles`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Titles {
    #[serde(default)]
    pub english: String,
    #[serde(default)]
    pub pretty: String,
    #[serde(default)]
    pub japanese: String,
}

impl Titles {
    /// Pick the best title given the user preference. Mirrors
    /// `Gallery#getTitle()` fallback chain.
    pub fn best(&self, pref: TitleType) -> String {
        let pick = |s: &str| -> bool { s.trim().len() > 2 };
        match pref {
            TitleType::Pretty if pick(&self.pretty) => self.pretty.clone(),
            TitleType::English if pick(&self.english) => self.english.clone(),
            TitleType::Japanese if pick(&self.japanese) => self.japanese.clone(),
            _ => {
                if pick(&self.pretty) {
                    self.pretty.clone()
                } else if pick(&self.english) {
                    self.english.clone()
                } else if pick(&self.japanese) {
                    self.japanese.clone()
                } else {
                    "Unnamed".to_string()
                }
            }
        }
    }
}

/// Full gallery detail. Mirrors `Gallery` + `GalleryData`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gallery {
    pub id: i64,
    pub media_id: i64,
    pub upload_date: Option<DateTime<Utc>>,
    pub num_favorites: i64,
    pub num_pages: usize,
    pub titles: Titles,
    pub tags: Vec<Tag>,
    pub cover: Page,
    pub thumbnail: Page,
    pub pages: Vec<Page>,
    /// True when the gallery is favorited on the remote site.
    #[serde(default)]
    pub is_favorited: bool,
    /// Related galleries (from `?include=related`).
    #[serde(default)]
    pub related: Vec<SimpleGallery>,
}

impl Gallery {
    pub fn best_title(&self, pref: TitleType) -> String {
        self.titles.best(pref)
    }

    pub fn language(&self) -> Language {
        for t in &self.tags {
            if t.tag_type == TagType::Language {
                match t.id {
                    special_tag_ids::LANGUAGE_JAPANESE => return Language::Japanese,
                    special_tag_ids::LANGUAGE_ENGLISH => return Language::English,
                    special_tag_ids::LANGUAGE_CHINESE => return Language::Chinese,
                    _ => {}
                }
            }
        }
        Language::All
    }

    /// Path-safe title. Mirrors `Gallery#getPathTitle`.
    pub fn path_title(&self, pref: TitleType) -> String {
        let title = self.best_title(pref);
        let mut s = title.replace('/', " ");
        let bad = ['|', '\\', '*', '"', '\'', '?', ':', '<', '>'];
        for c in bad {
            s = s.replace(c, " ");
        }
        while s.contains("  ") {
            s = s.replace("  ", " ");
        }
        s.trim().to_string()
    }

    /// Filesystem-safe name used for download folders.
    pub fn download_folder_name(&self, pref: TitleType) -> String {
        let title = self.path_title(pref);
        if title.is_empty() {
            format!("[{}]", self.id)
        } else {
            format!("[{}] {}", self.id, title)
        }
    }
}

/// Lightweight gallery used in listings. Mirrors `SimpleGallery`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleGallery {
    pub id: i64,
    pub media_id: i64,
    pub title: String,
    pub thumbnail: Option<String>,
    pub language: Language,
    #[serde(default)]
    pub tags: Vec<Tag>,
    #[serde(default)]
    pub num_pages: usize,
}

// ---------------------------------------------------------------------------
// Search results
// ---------------------------------------------------------------------------

/// A page of search/list results. Mirrors the v2 search response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPage {
    pub galleries: Vec<SimpleGallery>,
    pub page: u32,
    /// Total number of pages reported by the server, or `page` when unknown.
    pub num_pages: u32,
    pub per_page: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PopularTags {
    pub tags: Vec<Tag>,
}

// ---------------------------------------------------------------------------
// Favorites (online)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoriteGroup {
    pub id: i64,
    pub name: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoritesPage {
    pub galleries: Vec<SimpleGallery>,
    pub page: u32,
    pub num_pages: u32,
}

// ---------------------------------------------------------------------------
// Comments
// ---------------------------------------------------------------------------

/// Mirrors `api/comments/User`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentUser {
    pub id: i64,
    pub username: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub avatar_url: Option<String>,
    pub is_superuser: bool,
    pub is_staff: bool,
}

/// Mirrors `Comment`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: i64,
    pub gallery_id: i64,
    pub poster: CommentUser,
    pub body: String,
    pub create_date: Option<DateTime<Utc>>,
    pub post_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub vote: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentsPage {
    pub comments: Vec<Comment>,
    pub gallery_id: i64,
}

// ---------------------------------------------------------------------------
// User
// ---------------------------------------------------------------------------

/// Mirrors `loginapi/User`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub is_superuser: bool,
    #[serde(default)]
    pub is_staff: bool,
}

// ---------------------------------------------------------------------------
// Local library
// ---------------------------------------------------------------------------

/// A gallery present on disk under the download folder. Mirrors `LocalGallery`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalGallery {
    pub id: i64,
    pub title: String,
    pub thumbnail_path: Option<String>,
    pub folder: String,
    pub num_pages: usize,
    pub page_files: Vec<String>,
    pub media_id: i64,
    pub scanned_at: String,
}

/// Tag groupings for the UI: `tag -> tag` list, `artist -> tag` list, ...
pub type TagMap = BTreeMap<String, Vec<Tag>>;

// ---------------------------------------------------------------------------
// History
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub gallery_id: i64,
    pub title: String,
    pub media_id: i64,
    pub thumbnail: String,
    pub visited_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Misc request helpers
// ---------------------------------------------------------------------------

/// A search query. Mirrors the inputs to `InspectorV3#searchInspector`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchQuery {
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub tags: Vec<Tag>,
    #[serde(default)]
    pub accepted_tag_ids: Vec<i64>,
    #[serde(default)]
    pub avoided_tag_ids: Vec<i64>,
    pub page: u32,
    #[serde(default)]
    pub sort: SortType,
    #[serde(default)]
    pub only_language: Language,
    #[serde(default)]
    pub from_page: Option<i32>,
    #[serde(default)]
    pub to_page: Option<i32>,
}

/// Status returned by `auth_status`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatus {
    pub has_credentials: bool,
    pub api_key_valid: bool,
    pub cloudflare_solved: bool,
}

/// Cloudflare challenge state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CfState {
    #[default]
    Unknown,
    Needed,
    Pending,
    Solved,
    Failed,
}
