import { defineStore } from "pinia";
import { computed, ref } from "vue";

import { favAdd, favIsFavorite, favList, favRemove } from "@/api";
import type { FavoriteRow } from "@/types";

export const useFavoritesStore = defineStore("favorites", () => {
  const items = ref<FavoriteRow[]>([]);
  const loaded = ref(false);
  // Quick lookup of favorited gallery IDs.
  const ids = computed(() => new Set(items.value.map((f) => f.id)));

  async function load() {
    items.value = await favList(500, 0);
    loaded.value = true;
    return items.value;
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
    if (!items.value.some((f) => f.id === g.id)) {
      items.value.unshift({
        id: g.id,
        title: g.title,
        media_id: g.media_id,
        thumbnail: g.thumbnail,
        added_at: new Date().toISOString(),
      });
    }
  }

  async function remove(id: number) {
    await favRemove(id);
    items.value = items.value.filter((f) => f.id !== id);
  }

  async function toggle(g: {
    id: number;
    title: string;
    media_id: number;
    thumbnail: string;
  }) {
    // Optimistic check; refresh from DB on disagreement.
    const fav = ids.value.has(g.id);
    if (fav) {
      await remove(g.id);
    } else {
      await add(g);
    }
    return !fav;
  }

  return { items, ids, loaded, load, isFavorite, add, remove, toggle };
});

// Re-export favIsFavorite for one-off checks.
export { favIsFavorite };
