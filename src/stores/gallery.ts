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
  const commentsPage = ref(0);
  const commentsNumPages = ref(1);
  const commentsTotal = ref<number | null>(null);
  const commentsLoading = ref(false);
  const commentsError = ref<string | null>(null);
  const user = ref<User | null>(null);
  let loadRequestId = 0;

  function isTransientGalleryError(error: unknown): boolean {
    const message = error instanceof Error ? error.message : String(error);
    return /network error|timed? out|connection|dns|request error/i.test(message);
  }

  async function fetchGalleryWithStartupRetry(id: number): Promise<Gallery> {
    try {
      return await apiGetGallery(id);
    } catch (error: unknown) {
      if (!isTransientGalleryError(error)) throw error;
      // Android can report a transient DNS/socket error while its network is
      // becoming available immediately after a cold launch. One short retry
      // makes opening the first gallery deterministic without masking API,
      // authentication, or Cloudflare errors.
      await new Promise((resolve) => setTimeout(resolve, 350));
      return apiGetGallery(id);
    }
  }

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
    const requestId = ++loadRequestId;
    loading.value = true;
    error.value = null;
    current.value = null;
    comments.value = [];
    commentsPage.value = 0;
    commentsNumPages.value = 1;
    commentsTotal.value = null;
    commentsError.value = null;
    try {
      const local = await localGet(id);
      if (local && local.page_files.length > 0) {
        const result = localToGallery(local);
        if (requestId === loadRequestId) current.value = result;
        return result;
      }
      const result = await fetchGalleryWithStartupRetry(id);
      if (requestId === loadRequestId) current.value = result;
      return result;
    } catch (e: unknown) {
      if (requestId === loadRequestId) error.value = String(e);
      throw e;
    } finally {
      if (requestId === loadRequestId) loading.value = false;
    }
  }

  async function loadComments(galleryId: number, page = 1): Promise<Comment[]> {
    if (commentsLoading.value) return comments.value;
    commentsLoading.value = true;
    commentsError.value = null;
    try {
      const result = await apiGetComments(galleryId, page);
      comments.value = page === 1
        ? result.comments
        : [...comments.value, ...result.comments.filter(
            (incoming) => !comments.value.some((existing) => existing.id === incoming.id),
          )];
      commentsPage.value = result.page;
      commentsNumPages.value = result.num_pages;
      commentsTotal.value = result.total;
      return comments.value;
    } catch (error: unknown) {
      commentsError.value = error instanceof Error ? error.message : String(error);
      throw error;
    } finally {
      commentsLoading.value = false;
    }
  }

  async function loadMoreComments(galleryId: number): Promise<Comment[]> {
    if (commentsPage.value >= commentsNumPages.value) return comments.value;
    return loadComments(galleryId, commentsPage.value + 1);
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
    commentsPage,
    commentsNumPages,
    commentsTotal,
    commentsLoading,
    commentsError,
    user,
    browse,
    search,
    random,
    load,
    loadComments,
    loadMoreComments,
    loadUser,
    favorites,
  };
});
