// Thin wrapper around Tauri's `invoke` for every backend command.
// Each function maps 1:1 to a `#[tauri::command]` in `src-tauri/src/commands.rs`.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AuthCredentials,
  AuthStatus,
  Comment,
  CommentsPage,
  DownloadEntry,
  DownloadProgress,
  DownloadRequest,
  DownloadRow,
  FavoriteRow,
  FavoritesPage,
  Gallery,
  HistoryEntry,
  Language,
  LocalGallery,
  Page,
  SearchPage,
  SearchQuery,
  Settings,
  SimpleGallery,
  SortType,
  Tag,
  TagStatus,
  TagType,
  User,
} from "@/types";

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

export const settingsGet = (): Promise<Settings> => invoke("settings_get");
export const settingsSet = (s: Settings): Promise<Settings> => invoke("settings_set", { newSettings: s });
export const settingsGetPaths = (): Promise<{ app_data: string; log_dir: string | null }> => invoke("settings_get_paths");
export const settingsPickDirectory = (): Promise<string | null> => invoke("settings_pick_directory");
export const settingsClearCookies = (): Promise<void> => invoke("settings_clear_cookies");

// ---------------------------------------------------------------------------
// Auth + Cloudflare
// ---------------------------------------------------------------------------

export const authGet = (): Promise<AuthCredentials> => invoke("auth_get");
export const authSetApiKey = (apiKey: string): Promise<Settings> => invoke("auth_set_api_key", { apiKey });
export const authClear = (): Promise<Settings> => invoke("auth_clear");
export const authStatus = (): Promise<AuthStatus> => invoke("auth_status");
export const cloudflareCheck = (): Promise<boolean> => invoke("cloudflare_check");
export const cloudflareOpenChallenge = (): Promise<void> => invoke("cloudflare_open_challenge");
export const cloudflareIsSolved = (): Promise<boolean> => invoke("cloudflare_is_solved");

// ---------------------------------------------------------------------------
// API
// ---------------------------------------------------------------------------

export const apiBrowse = (page: number, sort: SortType): Promise<SearchPage> =>
  invoke("api_browse", { page, sort });

export const apiSearch = (query: SearchQuery): Promise<SearchPage> =>
  invoke("api_search", { query });

export const apiRandom = (): Promise<Gallery> => invoke("api_random");

export const apiGetGallery = (id: number): Promise<Gallery> => invoke("api_get_gallery", { id });

export const apiGetUser = (): Promise<User> => invoke("api_get_user");

export const apiGetComments = (galleryId: number): Promise<CommentsPage> =>
  invoke("api_get_comments", { galleryId });

export const apiGetFavoritesPage = (page: number, query?: string): Promise<FavoritesPage> =>
  invoke("api_get_favorites_page", { page, query });

export const apiGetTags = (typeFilter?: TagType | null): Promise<Tag[]> =>
  invoke("api_get_tags", { typeFilter });

export const apiGetPopularTags = (): Promise<Tag[]> => invoke("api_get_popular_tags");

// ---------------------------------------------------------------------------
// Favorites (local DB)
// ---------------------------------------------------------------------------

export const favAdd = (
  id: number,
  title: string,
  mediaId: number,
  thumbnail: string,
): Promise<void> => invoke("fav_add", { id, title, mediaId, thumbnail });

export const favRemove = (id: number): Promise<void> => invoke("fav_remove", { id });

export const favIsFavorite = (id: number): Promise<boolean> => invoke("fav_is_favorite", { id });

export const favList = (limit?: number, offset?: number): Promise<FavoriteRow[]> =>
  invoke("fav_list", { limit, offset });

// ---------------------------------------------------------------------------
// Tags
// ---------------------------------------------------------------------------

export const tagsGetAll = (): Promise<Tag[]> => invoke("tags_get_all");
export const tagsGetByType = (typeFilter?: TagType | null): Promise<Tag[]> =>
  invoke("tags_get_by_type", { typeFilter });
export const tagsSetStatus = (id: number, status: TagStatus): Promise<void> =>
  invoke("tags_set_status", { id, status });
export const tagsAddBlacklist = (id: number): Promise<void> =>
  invoke("tags_add_blacklist", { id });
export const tagsRemoveBlacklist = (id: number): Promise<void> =>
  invoke("tags_remove_blacklist", { id });
export const tagsSearch = (query: string, limit?: number): Promise<Tag[]> =>
  invoke("tags_search", { query, limit });
export const tagsGetPopular = (): Promise<Tag[]> => invoke("tags_get_popular");

// ---------------------------------------------------------------------------
// History
// ---------------------------------------------------------------------------

export const historyAdd = (
  id: number,
  title: string,
  mediaId: number,
  thumbnail: string,
): Promise<void> => invoke("history_add", { id, title, mediaId, thumbnail });
export const historyList = (limit?: number): Promise<HistoryEntry[]> =>
  invoke("history_list", { limit });
export const historyClear = (): Promise<void> => invoke("history_clear");

// ---------------------------------------------------------------------------
// Local library
// ---------------------------------------------------------------------------

export const localScan = (): Promise<LocalGallery[]> => invoke("local_scan");
export const localList = (): Promise<LocalGallery[]> => invoke("local_list");
export const localDelete = (folder: string): Promise<void> => invoke("local_delete", { folder });

// ---------------------------------------------------------------------------
// Downloader
// ---------------------------------------------------------------------------

export const downloadGallery = (req: DownloadRequest): Promise<DownloadEntry> =>
  invoke("download_gallery", { req });

export const downloadRange = (
  galleryId: number,
  fromPage?: number,
  toPage?: number,
): Promise<DownloadEntry> =>
  invoke("download_range", { galleryId, fromPage, toPage });

export const downloadList = (): Promise<DownloadEntry[]> => invoke("download_list");
export const downloadRows = (): Promise<DownloadRow[]> => invoke("download_rows");
export const downloadCancel = (id: number): Promise<void> => invoke("download_cancel", { id });
export const downloadPause = (id: number): Promise<void> => invoke("download_pause", { id });
export const downloadResume = (id: number): Promise<void> => invoke("download_resume", { id });
export const downloadClear = (): Promise<void> => invoke("download_clear");

export function onDownloadProgress(cb: (p: DownloadProgress) => void): Promise<UnlistenFn> {
  return listen<DownloadProgress>("download:progress", (e) => cb(e.payload));
}

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

export const exportPdf = (folder: string, out?: string): Promise<string> =>
  invoke("export_pdf", { folder, out });
export const exportZip = (folder: string, out?: string): Promise<string> =>
  invoke("export_zip", { folder, out });

// ---------------------------------------------------------------------------
// Misc
// ---------------------------------------------------------------------------

export const openInBrowser = (path: string): Promise<void> => invoke("open_in_browser", { path });
export const openPath = (path: string): Promise<void> => invoke("open_path", { path });
export const resolveAsset = (path: string): Promise<string> => invoke("resolve_asset", { path });
export const imageProxyUrl = (url: string): string => {
  // This is a sync command but we expose a sync helper that mirrors the
  // backend logic directly for fast image rendering.
  if (url && url.startsWith("http")) return url;
  return "asset://localhost/" + url.replace(/\\/g, "/").replace(/^\/+/, "");
};
export const readLocalImage = (path: string): Promise<string | null> =>
  invoke("read_local_image", { path });
export const registerApp = (): Promise<void> => invoke("register_app");

// Re-exports for convenience.
export type {
  AuthCredentials,
  AuthStatus,
  Comment,
  CommentsPage,
  DownloadEntry,
  DownloadProgress,
  DownloadRequest,
  DownloadRow,
  FavoriteRow,
  FavoritesPage,
  Gallery,
  HistoryEntry,
  Language,
  LocalGallery,
  Page,
  SearchPage,
  SearchQuery,
  Settings,
  SimpleGallery,
  SortType,
  Tag,
  TagStatus,
  TagType,
  User,
};
