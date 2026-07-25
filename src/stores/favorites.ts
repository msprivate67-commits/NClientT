import { defineStore } from "pinia";
import { computed, ref } from "vue";

import {
  apiAddFavorite,
  apiGetFavoritesPage,
  apiRemoveFavorite,
  apiSyncLocalFavorites,
  authStatus,
  favAdd,
  favIsFavorite,
  favList,
  favRemove,
} from "@/api";
import type { FavoriteRow, SimpleGallery } from "@/types";

export const useFavoritesStore = defineStore("favorites", () => {
  const items = ref<FavoriteRow[]>([]);
  const galleries = ref<SimpleGallery[]>([]);
  const page = ref(1);
  const numPages = ref(0);
  const online = ref(false);
  const loaded = ref(false);
  // Quick lookup of favorited gallery IDs.
  const ids = computed(() => {
    const set = new Set(items.value.map((f) => f.id));
    for (const gallery of galleries.value) set.add(gallery.id);
    return set;
  });

  function localRowsToGalleries(rows: FavoriteRow[]): SimpleGallery[] {
    return rows.map((f) => ({
      id: f.id,
      media_id: f.media_id,
      title: f.title,
      thumbnail: f.thumbnail || null,
      language: "all",
      tags: [],
      num_pages: 0,
    }));
  }

  async function load(nextPage = 1) {
    const status = await authStatus();
    online.value = status.has_credentials;
    items.value = await favList(500, 0);
    if (online.value) {
      await apiSyncLocalFavorites();
      const result = await apiGetFavoritesPage(nextPage);
      page.value = result.page;
      numPages.value = result.num_pages;
      galleries.value = result.galleries;
    } else {
      page.value = 1;
      numPages.value = 0;
      galleries.value = localRowsToGalleries(items.value);
    }
    loaded.value = true;
    return galleries.value;
  }

  async function isFavorite(id: number) {
    if (!loaded.value) await load();
    return ids.value.has(id);
  }

  async function add(g: {
    id: number;
    title: string;
    media_id: number;
    thumbnail: string;
  }) {
    await favAdd(g.id, g.title, g.media_id, g.thumbnail);
    if (online.value) {
      await apiAddFavorite(g.id);
    }
    if (!items.value.some((f) => f.id === g.id)) {
      items.value.unshift({
        id: g.id,
        title: g.title,
        media_id: g.media_id,
        thumbnail: g.thumbnail,
        added_at: new Date().toISOString(),
      });
    }
    if (!galleries.value.some((f) => f.id === g.id)) {
      galleries.value.unshift({
        id: g.id,
        media_id: g.media_id,
        title: g.title,
        thumbnail: g.thumbnail || null,
        language: "all",
        tags: [],
        num_pages: 0,
      });
    }
  }

  async function remove(id: number) {
    if (online.value) {
      await apiRemoveFavorite(id);
    }
    await favRemove(id);
    items.value = items.value.filter((f) => f.id !== id);
    galleries.value = galleries.value.filter((f) => f.id !== id);
  }

  async function toggle(g: {
    id: number;
    title: string;
    media_id: number;
    thumbnail: string;
  }) {
    if (!loaded.value) await load();
    // Optimistic check; refresh from DB on disagreement.
    const fav = ids.value.has(g.id);
    if (fav) {
      await remove(g.id);
    } else {
      await add(g);
    }
    return !fav;
  }

  async function syncAfterLogin() {
    online.value = true;
    await apiSyncLocalFavorites();
    await load(1);
  }

  return {
    items,
    galleries,
    page,
    numPages,
    online,
    ids,
    loaded,
    load,
    isFavorite,
    add,
    remove,
    toggle,
    syncAfterLogin,
  };
});

// Re-export favIsFavorite for one-off checks.
export { favIsFavorite };
