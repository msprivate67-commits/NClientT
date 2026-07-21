<script setup lang="ts">
import { computed, onMounted, ref } from "vue";

import GalleryGrid from "@/components/GalleryGrid.vue";
import GalleryCard from "@/components/GalleryCard.vue";
import Pagination from "@/components/Pagination.vue";
import EmptyState from "@/components/EmptyState.vue";
import { apiGetFavoritesPage } from "@/api";
import { useFavoritesStore } from "@/stores/favorites";
import { useScrollCache } from "@/composables/useScrollCache";
import type { SimpleGallery } from "@/types";

const favorites = useFavoritesStore();

const mode = ref<"local" | "online">("local");
const page = ref(1);
const numPages = ref(0);
const onlineItems = ref<SimpleGallery[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);
const viewRef = ref<HTMLElement | null>(null);
useScrollCache(viewRef);

const localItems = computed<SimpleGallery[]>(
  () =>
    favorites.items.map((f) => ({
      id: f.id,
      media_id: f.media_id,
      title: f.title,
      thumbnail: f.thumbnail || null,
      language: "all",
      tags: [],
      num_pages: 0,
    })),
);

async function loadOnline() {
  loading.value = true;
  error.value = null;
  try {
    const result = await apiGetFavoritesPage(page.value);
    onlineItems.value = result.galleries;
    numPages.value = result.num_pages;
  } catch (e: any) {
    error.value = String(e?.message ?? e);
  } finally {
    loading.value = false;
  }
}

onMounted(async () => {
  await favorites.load();
});

function switchMode(m: "local" | "online") {
  mode.value = m;
  if (m === "online" && onlineItems.value.length === 0) loadOnline();
}

async function refresh() {
  if (mode.value === "online") {
    await loadOnline();
  } else {
    await favorites.load();
  }
}
</script>

<template>
  <div ref="viewRef" class="view">
    <div class="view-header">
      <div class="view-title">Favorites</div>
      <div class="toolbar">
        <button class="btn" :class="{ primary: mode === 'local' }" @click="switchMode('local')">Local</button>
        <button class="btn" :class="{ primary: mode === 'online' }" @click="switchMode('online')">Online</button>
        <button class="btn" :disabled="loading" @click="refresh" title="Reload favorites">
          {{ loading ? "Refreshing…" : "🔄 Refresh" }}
        </button>
      </div>
    </div>

    <div v-if="error" class="error">{{ error }}</div>

    <template v-if="mode === 'local'">
      <div v-if="localItems.length" class="grid">
        <GalleryCard v-for="g in localItems" :key="g.id" :gallery="g" />
      </div>
      <EmptyState v-else title="No favorites yet" hint="Click ★ on any gallery to add it here." />
    </template>

    <template v-else>
      <GalleryGrid
        :galleries="onlineItems"
        :loading="loading"
        empty-title="No online favorites"
        empty-hint="Online favorites require an API key. Add one in Settings."
      />
      <Pagination :page="page" :num-pages="numPages" @change="page = $event; loadOnline()" />
    </template>
  </div>
</template>

<style scoped>
.error {
  padding: 12px 14px;
  background: rgba(255, 80, 80, 0.1);
  border: 1px solid rgba(255, 80, 80, 0.4);
  border-radius: 8px;
  color: #ff9e9e;
  margin-bottom: 14px;
  font-size: 0.85rem;
}
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 12px;
}
</style>
