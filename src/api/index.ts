// Thin wrapper around Tauri's `invoke` for every backend command.
// Each function maps 1:1 to a `#[tauri::command]` in `src-tauri/src/commands.rs`.

import { invoke, convertFileSrc } from "@tauri-apps/api/core";
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
export const settingsListDownloadCandidates = (): Promise<[string, string][]> => invoke("settings_list_download_candidates");
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
// Read progress
// ---------------------------------------------------------------------------

export interface ReadProgressRow {
  gallery_id: number;
  last_page: number;
  total_pages: number;
  read: boolean;
  updated_at: string;
}

export const readProgressSet = (
  galleryId: number,
  lastPage: number,
  totalPages: number,
): Promise<void> =>
  invoke("read_progress_set", { galleryId, lastPage, totalPages });
export const readProgressReset = (galleryId: number): Promise<void> =>
  invoke("read_progress_reset", { galleryId });
export const readProgressGet = (
  galleryId: number,
): Promise<ReadProgressRow | null> =>
  invoke("read_progress_get", { galleryId });
export const readProgressIds = (): Promise<number[]> =>
  invoke("read_progress_ids");

// ---------------------------------------------------------------------------
// Local library
// ---------------------------------------------------------------------------

export const localScan = (): Promise<LocalGallery[]> => invoke("local_scan");
export const localList = (): Promise<LocalGallery[]> => invoke("local_list");
export const localIds = (): Promise<number[]> => invoke("local_ids");
export const localGet = (galleryId: number): Promise<LocalGallery | null> => invoke("local_get", { galleryId });
export const localSetTranslatedTitle = (galleryId: number, title: string): Promise<void> =>
  invoke("local_set_translated_title", { galleryId, title });
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
  // Remote images load directly; local paths go through Tauri's asset
  // protocol. `convertFileSrc` picks the correct scheme per platform
  // (`http://asset.localhost/` on Windows/Android, `asset://localhost/`
  // on macOS/Linux) and encodes the path — both of which a hardcoded
  // `asset://localhost/` got wrong on Windows/Android.
  if (url && url.startsWith("http")) return url;
  if (!url) return "";
  return convertFileSrc(url);
};
export const readLocalImage = (path: string): Promise<string | null> =>
  invoke("read_local_image", { path });
export const registerApp = (): Promise<void> => invoke("register_app");

// --- AI Translation ---

export async function translateTitle(
  baseUrl: string,
  model: string,
  apiKey: string,
  title: string,
  targetLang: string,
  thinking: boolean,
): Promise<string> {
  const url = `${baseUrl.replace(/\/+$/, "")}/v1/chat/completions`;
  const body: Record<string, unknown> = {
    model,
    messages: [
      {
        role: "system",
        content: `You are a translator. The following text is a hentai manga title. The original language is either Japanese or English. Translate it to ${targetLang}. Output ONLY the translated title, nothing else — no quotes, no extra words, no explanations.`,
      },
      { role: "user", content: title },
    ],
    temperature: 0.1,
  };
  if (thinking) {
    body["reasoning_effort"] = "medium";
  }
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  if (apiKey) {
    headers["Authorization"] = `Bearer ${apiKey}`;
  }
  const resp = await fetch(url, {
    method: "POST",
    headers,
    body: JSON.stringify(body),
  });
  if (!resp.ok) {
    const text = await resp.text();
    throw new Error(`Translation API error (${resp.status}): ${text}`);
  }
  const json = await resp.json();
  const content = json?.choices?.[0]?.message?.content;
  if (typeof content !== "string" || !content.trim()) {
    throw new Error("Empty translation response");
  }
  return content.trim();
}

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
