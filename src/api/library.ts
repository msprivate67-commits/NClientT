import { invoke } from "@tauri-apps/api/core";
import type {
  FavoriteRow,
  Gallery,
  HistoryEntry,
  LocalGallery,
  Tag,
  TagStatus,
  TagType,
} from "@/types";

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

export const tagsGetAll = (): Promise<Tag[]> => invoke("tags_get_all");
export const tagsGetByType = (typeFilter?: TagType | null): Promise<Tag[]> =>
  invoke("tags_get_by_type", { typeFilter });
export const tagsSetStatus = (id: number, status: TagStatus): Promise<void> =>
  invoke("tags_set_status", { id, status });
export const tagsAddBlacklist = (id: number): Promise<void> => invoke("tags_add_blacklist", { id });
export const tagsRemoveBlacklist = (id: number): Promise<void> =>
  invoke("tags_remove_blacklist", { id });
export const tagsSearch = (query: string, limit?: number): Promise<Tag[]> =>
  invoke("tags_search", { query, limit });
export const tagsGetPopular = (): Promise<Tag[]> => invoke("tags_get_popular");

export const historyAdd = (
  id: number,
  title: string,
  mediaId: number,
  thumbnail: string,
): Promise<void> => invoke("history_add", { id, title, mediaId, thumbnail });
export const historyList = (limit?: number): Promise<HistoryEntry[]> =>
  invoke("history_list", { limit });
export const historyClear = (): Promise<void> => invoke("history_clear");

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
): Promise<void> => invoke("read_progress_set", { galleryId, lastPage, totalPages });
export const readProgressReset = (galleryId: number): Promise<void> =>
  invoke("read_progress_reset", { galleryId });
export const readProgressGet = (galleryId: number): Promise<ReadProgressRow | null> =>
  invoke("read_progress_get", { galleryId });
export const readProgressIds = (): Promise<number[]> => invoke("read_progress_ids");

export const localReaderProgressSet = (
  galleryId: number,
  page: number,
  totalPages: number,
): Promise<void> => invoke("local_reader_progress_set", { galleryId, page, totalPages });
export const localReaderProgressGet = (galleryId: number): Promise<number | null> =>
  invoke("local_reader_progress_get", { galleryId });

export const localScan = (): Promise<LocalGallery[]> => invoke("local_scan");
export const localList = (): Promise<LocalGallery[]> => invoke("local_list");
export const localIds = (): Promise<number[]> => invoke("local_ids");
export const localGet = (galleryId: number): Promise<LocalGallery | null> =>
  invoke("local_get", { galleryId });
export const localGetMeta = (galleryId: number): Promise<Gallery | null> =>
  invoke("local_get_meta", { galleryId });
export const localSetTranslatedTitle = (galleryId: number, title: string): Promise<void> =>
  invoke("local_set_translated_title", { galleryId, title });
export const localDelete = (folder: string): Promise<void> => invoke("local_delete", { folder });
