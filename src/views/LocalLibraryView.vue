<script setup lang="ts">
import { onMounted, ref } from "vue";

import GalleryCard from "@/components/GalleryCard.vue";
import EmptyState from "@/components/EmptyState.vue";
import { localScan } from "@/api";
import { useDownloadedStore } from "@/stores/downloaded";
import { useScrollCache } from "@/composables/useScrollCache";
import type { LocalGallery, SimpleGallery } from "@/types";

const items = ref<LocalGallery[]>([]);
const scanning = ref(false);
const viewRef = ref<HTMLElement | null>(null);
useScrollCache(viewRef);
const downloaded = useDownloadedStore();

function toSimple(l: LocalGallery): SimpleGallery {
  return {
    id: l.id,
    media_id: l.media_id,
    title: l.title || `#${l.id}`,
    thumbnail: l.thumbnail_path ?? null,
    language: "all",
    tags: [],
    num_pages: l.num_pages,
  };
}

async function scan() {
  scanning.value = true;
  try {
    items.value = await localScan();
    // A scan may pick up galleries newly placed in the download dir (or
    // remove deleted ones), so re-sync the "downloaded" id set that badges
    // online covers.
    await downloaded.refresh();
  } finally {
    scanning.value = false;
  }
}

onMounted(scan);
</script>

<template>
  <div ref="viewRef" class="view">
    <div class="view-header">
      <div class="view-title">Local Library</div>
      <div class="toolbar">
        <button class="btn" :disabled="scanning" @click="scan">
          {{ scanning ? "Scanning…" : "Rescan" }}
        </button>
      </div>
    </div>

    <div v-if="items.length" class="grid">
      <GalleryCard
        v-for="l in items"
        :key="l.folder"
        :gallery="toSimple(l)"
        local
        :thumbnail-override="l.thumbnail_path"
      />
    </div>
    <EmptyState
      v-else
      title="No local galleries"
      hint="Download galleries and they'll appear here automatically. You can also click Rescan to refresh."
    />
  </div>
</template>

<style scoped>
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 12px;
}
/* Match GalleryGrid: two covers per row on phones. */
@media (max-width: 560px) {
  .grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
  }
}
</style>
