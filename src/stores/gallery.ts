import { defineStore } from "pinia";
import { ref } from "vue";

import {
  apiBrowse,
  apiGetComments,
  apiGetFavoritesPage,
  apiGetGallery,
  apiGetUser,
  apiRandom,
  apiSearch,
  localGet,
} from "@/api";
import type {
  Comment,
  FavoritesPage,
  Gallery,
  LocalGallery,
  Page,
  SearchPage,
  SearchQuery,
  SortType,
  User,
} from "@/types";

function localToGallery(lg: LocalGallery): Gallery {
  const pageFiles = lg.page_files || [];
  const pages: Page[] = pageFiles.map((path, i) => ({
    index: i + 1,
    path,
    thumbnail: path,
    width: 0,
    height: 0,
  }));
  const fallbackPage: Page = {
    index: 1,
    path: null,
    thumbnail: null,
    width: 0,
    height: 0,
  };
  return {
    id: lg.id,
    media_id: lg.media_id,
    upload_date: null,
    num_favorites: 0,
    num_pages: lg.num_pages,
    titles: {
      english: lg.title,
      pretty: lg.title,
      japanese: "",
    },
    tags: [],
    cover: lg.thumbnail_path
      ? { index: 1, path: lg.thumbnail_path, thumbnail: lg.thumbnail_path, width: 0, height: 0 }
      : pages[0] || fallbackPage,
    thumbnail: lg.thumbnail_path
      ? { index: 1, path: lg.thumbnail_path, thumbnail: lg.thumbnail_path, width: 0, height: 0 }
      : pages[0] || fallbackPage,
    pages,
    is_favorited: false,
    related: [],
  };
}

export const useGalleryStore = defineStore("gallery", () => {
  const loading = ref(false);
  const error = ref<string | null>(null);
  const current = ref<Gallery | null>(null);
  const comments = ref<Comment[]>([]);
  const user = ref<User | null>(null);

  async function browse(page: number, sort: SortType): Promise<SearchPage> {
    loading.value = true;
    error.value = null;
    try {
      return await apiBrowse(page, sort);
    } catch (e: any) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function search(q: SearchQuery): Promise<SearchPage> {
    loading.value = true;
    error.value = null;
    try {
      return await apiSearch(q);
    } catch (e: any) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function random(): Promise<Gallery> {
    loading.value = true;
    error.value = null;
    current.value = null;
    try {
      current.value = await apiRandom();
      return current.value;
    } finally {
      loading.value = false;
    }
  }

  async function load(id: number): Promise<Gallery> {
    loading.value = true;
    error.value = null;
    current.value = null;
    try {
      const local = await localGet(id);
      if (local && local.page_files.length > 0) {
        current.value = localToGallery(local);
        return current.value;
      }
      current.value = await apiGetGallery(id);
      return current.value;
    } catch (e: any) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function loadComments(galleryId: number): Promise<Comment[]> {
    const page = await apiGetComments(galleryId);
    comments.value = page.comments;
    return page.comments;
  }

  async function loadUser(): Promise<User | null> {
    try {
      user.value = await apiGetUser();
      return user.value;
    } catch {
      user.value = null;
      return null;
    }
  }

  async function favorites(page: number, query?: string): Promise<FavoritesPage> {
    loading.value = true;
    try {
      return await apiGetFavoritesPage(page, query);
    } finally {
      loading.value = false;
    }
  }

  return {
    loading,
    error,
    current,
    comments,
    user,
    browse,
    search,
    random,
    load,
    loadComments,
    loadUser,
    favorites,
  };
});
