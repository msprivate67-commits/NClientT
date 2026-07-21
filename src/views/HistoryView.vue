<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";

import EmptyState from "@/components/EmptyState.vue";
import { historyClear, historyList, imageProxyUrl } from "@/api";
import { useScrollCache } from "@/composables/useScrollCache";
import type { HistoryEntry } from "@/types";

const router = useRouter();
const items = ref<HistoryEntry[]>([]);
const viewRef = ref<HTMLElement | null>(null);
useScrollCache(viewRef);

async function load() {
  items.value = await historyList(200);
}

function open(id: number) {
  router.push({ name: "gallery", params: { id } });
}

async function clear() {
  if (!confirm("Clear all history?")) return;
  await historyClear();
  items.value = [];
}

onMounted(load);
</script>

<template>
  <div ref="viewRef" class="view">
    <div class="view-header">
      <div class="view-title">History</div>
      <div class="toolbar">
        <button class="btn" @click="load" title="Reload history">🔄 Refresh</button>
        <button v-if="items.length" class="btn danger" @click="clear">Clear</button>
      </div>
    </div>
    <div v-if="items.length" class="list">
      <div v-for="h in items" :key="h.gallery_id" class="row" @click="open(h.gallery_id)">
        <div class="thumb">
          <img v-if="h.thumbnail" :src="imageProxyUrl(h.thumbnail)" :alt="h.title" />
        </div>
        <div class="info">
          <div class="title">{{ h.title || `#${h.gallery_id}` }}</div>
          <div class="time">{{ new Date(h.visited_at).toLocaleString() }}</div>
        </div>
      </div>
    </div>
    <EmptyState v-else title="No history" hint="Read a gallery and it'll show up here." />
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
