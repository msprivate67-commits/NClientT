import { invoke } from "@tauri-apps/api/core";
import type {
  CommentsPage,
  FavoriteStatus,
  FavoritesPage,
  Gallery,
  SearchPage,
  SearchQuery,
  SortType,
  Tag,
  TagType,
  User,
} from "@/types";

export const apiBrowse = (page: number, sort: SortType): Promise<SearchPage> =>
  invoke("api_browse", { page, sort });
export const apiSearch = (query: SearchQuery): Promise<SearchPage> =>
  invoke("api_search", { query });
export const apiRandom = (): Promise<Gallery> => invoke("api_random");
export const apiGetGallery = (id: number): Promise<Gallery> => invoke("api_get_gallery", { id });
export const apiGetUser = (): Promise<User> => invoke("api_get_user");
export const apiGetComments = (galleryId: number, page = 1): Promise<CommentsPage> =>
  invoke("api_get_comments", { galleryId, page });
export const apiGetFavoritesPage = (page: number, query?: string): Promise<FavoritesPage> =>
  invoke("api_get_favorites_page", { page, query });
export const apiCheckFavorite = (galleryId: number): Promise<FavoriteStatus> =>
  invoke("api_check_favorite", { galleryId });
export const apiAddFavorite = (galleryId: number): Promise<FavoriteStatus> =>
  invoke("api_add_favorite", { galleryId });
export const apiRemoveFavorite = (galleryId: number): Promise<FavoriteStatus> =>
  invoke("api_remove_favorite", { galleryId });
export const apiSyncLocalFavorites = (): Promise<number> => invoke("api_sync_local_favorites");
export const apiGetTags = (typeFilter?: TagType | null): Promise<Tag[]> =>
  invoke("api_get_tags", { typeFilter });
export const apiGetPopularTags = (): Promise<Tag[]> => invoke("api_get_popular_tags");
