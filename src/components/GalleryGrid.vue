<script setup lang="ts">
import { nextTick, onActivated, onDeactivated, onUnmounted, watch } from "vue";

import GalleryCard from "./GalleryCard.vue";
import EmptyState from "./EmptyState.vue";
import {
  loadImageObjectUrl,
  pinImageObjectSource,
} from "@/composables/useImageObjectCache";
import { usePriorityPreloadQueue } from "@/composables/usePriorityPreloadQueue";
import type { SimpleGallery } from "@/types";

const props = defineProps<{
  galleries: SimpleGallery[];
  loading?: boolean;
  emptyTitle?: string;
  emptyHint?: string;
  selectable?: boolean;
  selected?: Set<number>;
}>();

const emit = defineEmits<{
  (e: "select", id: number): void;
  (e: "deselect", id: number): void;
}>();

const coverPreloader = usePriorityPreloadQueue(async (index) => {
  const source = props.galleries[index]?.thumbnail;
  if (source) await loadImageObjectUrl(source);
}, 3);

let scheduleGeneration = 0;
let coverPinsActive = true;
let releasePinnedCovers: Array<() => void> = [];

function releaseCoverPins() {
  releasePinnedCovers.forEach((release) => release());
  releasePinnedCovers = [];
}

function syncCoverPins() {
  releaseCoverPins();
  if (!coverPinsActive) return;

  const sources = new Set(
    props.galleries
      .map((gallery) => gallery.thumbnail)
      .filter((source): source is string => !!source),
  );
  releasePinnedCovers = [...sources].map(pinImageObjectSource);
}

function prioritizeCover(id: number) {
  const index = props.galleries.findIndex((gallery) => gallery.id === id);
  if (index >= 0) coverPreloader.enqueue([index], true);
}

async function scheduleCoverLoading() {
  const generation = ++scheduleGeneration;
  coverPreloader.reset();

  // Let cards mount and report visible/nearby covers first. The remainder
  // then enters the same queue in page order and fills while the user reads.
  await nextTick();
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      if (generation !== scheduleGeneration || !coverPinsActive) return;
      coverPreloader.enqueue(props.galleries.map((_, index) => index));
    });
  });
}

watch(
  () => props.galleries.map((gallery) => `${gallery.id}:${gallery.thumbnail ?? ""}`).join("|"),
  () => {
    syncCoverPins();
    if (coverPinsActive) void scheduleCoverLoading();
  },
  { immediate: true },
);

onActivated(() => {
  coverPinsActive = true;
  syncCoverPins();
  void scheduleCoverLoading();
});

onDeactivated(() => {
  coverPinsActive = false;
  releaseCoverPins();
  scheduleGeneration++;
  coverPreloader.reset();
});

onUnmounted(() => {
  coverPinsActive = false;
  releaseCoverPins();
  scheduleGeneration++;
});
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
      :selectable="selectable"
      :selected="selected?.has(g.id)"
      managed-cover-loading
      @select="emit('select', $event)"
      @deselect="emit('deselect', $event)"
      @cover-priority="prioritizeCover"
    />
  </div>
</template>

<style scoped>
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 12px;
}
/* On phones force exactly two covers per row so they're large enough to read
   but still pack two across. The minmax above would otherwise collapse to a
   single column on very narrow viewports. */
@media (max-width: 560px) {
  .grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
  }
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
