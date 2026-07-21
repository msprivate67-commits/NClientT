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
} from "@/api";
import type {
  Comment,
  FavoritesPage,
  Gallery,
  SearchPage,
  SearchQuery,
  SortType,
  User,
} from "@/types";

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
