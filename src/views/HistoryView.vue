<script setup lang="ts">
import {
  nextTick,
  onActivated,
  onDeactivated,
  onMounted,
  onUnmounted,
  ref,
  watch,
} from "vue";
import { useI18n } from "vue-i18n";

import EmptyState from "@/components/EmptyState.vue";
import { RefreshCw } from "@lucide/vue";
import { historyClear, historyList } from "@/api";
import {
  cachedImageObjectUrl,
  loadImageObjectUrl,
} from "@/composables/useImageObjectCache";
import { usePriorityPreloadQueue } from "@/composables/usePriorityPreloadQueue";
import { useOverlayStore } from "@/stores/overlay";
import { useScrollCache } from "@/composables/useScrollCache";
import type { HistoryEntry } from "@/types";

const { t } = useI18n();
const overlay = useOverlayStore();
const items = ref<HistoryEntry[]>([]);
const viewRef = ref<HTMLElement | null>(null);
useScrollCache(viewRef);

let viewActive = true;
let scheduleGeneration = 0;
let coverObserver: IntersectionObserver | null = null;

const coverPreloader = usePriorityPreloadQueue(async (index) => {
  const source = items.value[index]?.thumbnail;
  if (source) await loadImageObjectUrl(source);
}, 2);

function coverSrc(source: string): string {
  return cachedImageObjectUrl(source);
}

function coverLoadingPaused(): boolean {
  return !viewActive || overlay.hasAny();
}

function stopCoverLoading() {
  scheduleGeneration++;
  coverObserver?.disconnect();
  coverObserver = null;
  coverPreloader.reset();
}

async function scheduleCoverLoading() {
  stopCoverLoading();
  if (coverLoadingPaused()) return;

  const generation = scheduleGeneration;
  await nextTick();
  const root = viewRef.value;
  if (!root || generation !== scheduleGeneration || coverLoadingPaused()) return;

  coverObserver = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        const index = Number((entry.target as HTMLElement).dataset.historyIndex);
        if (Number.isInteger(index)) coverPreloader.enqueue([index], true);
      }
    },
    { root, rootMargin: "400px 0px", threshold: 0.01 },
  );

  root.querySelectorAll<HTMLElement>("[data-history-index]")
    .forEach((element) => coverObserver?.observe(element));
  coverPreloader.enqueue(items.value.map((_, index) => index));
}

async function load() {
  items.value = await historyList(200);
}

function open(id: number) {
  overlay.openGallery(id);
}

async function clear() {
  if (!confirm(t("history.confirm_clear"))) return;
  await historyClear();
  items.value = [];
}

onMounted(load);
watch(
  () => items.value.map((item) => `${item.gallery_id}:${item.thumbnail}`).join("|"),
  () => void scheduleCoverLoading(),
);
watch(
  () => overlay.hasAny(),
  (covered) => {
    if (covered) stopCoverLoading();
    else void scheduleCoverLoading();
  },
);
onActivated(() => {
  viewActive = true;
  void scheduleCoverLoading();
});
onDeactivated(() => {
  viewActive = false;
  stopCoverLoading();
});
onUnmounted(stopCoverLoading);
</script>

<template>
  <div ref="viewRef" class="view">
    <div class="view-header">
      <div class="view-title">{{ $t('history.title') }}</div>
      <div class="toolbar">
        <button class="btn" @click="load" :title="$t('history.reload_history')"><RefreshCw :size="14" /> {{ $t('common.refresh') }}</button>
        <button v-if="items.length" class="btn danger" @click="clear">{{ $t('history.clear') }}</button>
      </div>
    </div>
    <div v-if="items.length" class="list">
      <div
        v-for="(h, index) in items"
        :key="h.gallery_id"
        class="row"
        :data-history-index="index"
        @click="open(h.gallery_id)"
      >
        <div class="thumb">
          <img v-if="coverSrc(h.thumbnail)" :src="coverSrc(h.thumbnail)" :alt="h.title" />
        </div>
        <div class="info">
          <div class="title">{{ h.title || `#${h.gallery_id}` }}</div>
          <div class="time">{{ new Date(h.visited_at).toLocaleString() }}</div>
        </div>
      </div>
    </div>
    <EmptyState v-else :title="$t('history.no_history')" :hint="$t('history.no_history_hint')" />
  </div>
</template>

<style scoped>
.list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.row {
  display: flex;
  gap: 12px;
  padding: 6px;
  border-radius: 8px;
  cursor: pointer;
}
.row:hover {
  background: var(--surface);
}
.thumb {
  width: 48px;
  height: 64px;
  background: var(--surface-2);
  border-radius: 4px;
  overflow: hidden;
  flex-shrink: 0;
}
.thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.info {
  min-width: 0;
  align-self: center;
}
.title {
  font-size: 0.88rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.time {
  font-size: 0.74rem;
  color: var(--text-dim);
  margin-top: 2px;
}
</style>
