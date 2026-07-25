<script setup lang="ts">
import { onMounted, ref } from "vue";

import GalleryCard from "@/components/GalleryCard.vue";
import Pagination from "@/components/Pagination.vue";
import EmptyState from "@/components/EmptyState.vue";
import { RefreshCw } from "@lucide/vue";
import { useFavoritesStore } from "@/stores/favorites";
import { useScrollCache } from "@/composables/useScrollCache";

const favorites = useFavoritesStore();

const loading = ref(false);
const error = ref<string | null>(null);
const viewRef = ref<HTMLElement | null>(null);
useScrollCache(viewRef);

async function load(page = 1) {
  loading.value = true;
  error.value = null;
  try {
    await favorites.load(page);
  } catch (e: any) {
    error.value = String(e?.message ?? e);
  } finally {
    loading.value = false;
  }
}

onMounted(async () => {
  await load();
});

async function refresh() {
  await load(favorites.page);
}
</script>

<template>
  <div ref="viewRef" class="view">
    <div class="view-header">
      <div class="view-title">{{ $t('favorites.title') }}</div>
      <div class="toolbar">
        <button class="btn" :disabled="loading" @click="refresh" :title="$t('favorites.reload_favorites')">
          {{ loading ? $t('common.refreshing') : '' }}<RefreshCw v-if="!loading" :size="14" /> {{ $t('common.refresh') }}
        </button>
      </div>
    </div>

    <div v-if="error" class="error">{{ error }}</div>

    <div v-if="favorites.galleries.length" class="grid">
      <GalleryCard v-for="g in favorites.galleries" :key="g.id" :gallery="g" />
    </div>
    <EmptyState v-else :title="$t('favorites.no_favorites')" :hint="$t('favorites.no_favorites_hint')" />
    <Pagination
      v-if="favorites.online"
      :page="favorites.page"
      :num-pages="favorites.numPages"
      @change="load($event)"
    />
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
/* On phones force exactly two covers per row, matching GalleryGrid, so the
   auto-fill above doesn't collapse to a single column on narrow viewports. */
@media (max-width: 560px) {
  .grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
  }
}
</style>
