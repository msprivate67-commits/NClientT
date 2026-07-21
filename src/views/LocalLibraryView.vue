<script setup lang="ts">
import { onMounted, ref } from "vue";

import GalleryCard from "@/components/GalleryCard.vue";
import EmptyState from "@/components/EmptyState.vue";
import { localList, localScan } from "@/api";
import { useScrollCache } from "@/composables/useScrollCache";
import type { LocalGallery, SimpleGallery } from "@/types";

const items = ref<LocalGallery[]>([]);
const scanning = ref(false);
const viewRef = ref<HTMLElement | null>(null);
useScrollCache(viewRef);

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

async function load() {
  items.value = await localList();
}

async function scan() {
  scanning.value = true;
  try {
    items.value = await localScan();
  } finally {
    scanning.value = false;
  }
}

onMounted(load);
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
      hint="Download galleries, or click Rescan to pick up existing folders in your download dir."
    />
  </div>
</template>

<style scoped>
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 12px;
}
</style>
