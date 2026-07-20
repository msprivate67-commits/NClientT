<script setup lang="ts">
import GalleryCard from "./GalleryCard.vue";
import EmptyState from "./EmptyState.vue";
import type { SimpleGallery } from "@/types";

defineProps<{
  galleries: SimpleGallery[];
  loading?: boolean;
  emptyTitle?: string;
  emptyHint?: string;
}>();
</script>

<template>
  <div v-if="loading && galleries.length === 0" class="grid loading">
    <div v-for="i in 12" :key="i" class="skeleton"></div>
  </div>
  <div v-else-if="galleries.length === 0">
    <EmptyState :title="emptyTitle" :hint="emptyHint" />
  </div>
  <div v-else class="grid">
    <GalleryCard
      v-for="g in galleries"
      :key="g.id"
      :gallery="g"
    />
  </div>
</template>

<style scoped>
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 12px;
}
.loading .skeleton {
  aspect-ratio: 3 / 4;
  background: var(--surface);
  border-radius: 8px;
  animation: pulse 1.4s ease-in-out infinite;
}
@keyframes pulse {
  0%, 100% { opacity: 0.6; }
  50% { opacity: 1; }
}
</style>
