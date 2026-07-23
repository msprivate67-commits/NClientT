import { invoke } from "@tauri-apps/api/core";
import type {
  CommentsPage,
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
export const apiGetComments = (galleryId: number): Promise<CommentsPage> =>
  invoke("api_get_comments", { galleryId });
export const apiGetFavoritesPage = (page: number, query?: string): Promise<FavoritesPage> =>
  invoke("api_get_favorites_page", { page, query });
export const apiGetTags = (typeFilter?: TagType | null): Promise<Tag[]> =>
  invoke("api_get_tags", { typeFilter });
export const apiGetPopularTags = (): Promise<Tag[]> => invoke("api_get_popular_tags");
