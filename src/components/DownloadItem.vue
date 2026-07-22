<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { Pause, Play, X, Folder } from "lucide-vue-next";

import type { DownloadEntry } from "@/types";

const { t: $t } = useI18n();

const props = defineProps<{ entry: DownloadEntry }>();
const emit = defineEmits<{
  (e: "pause", id: number): void;
  (e: "resume", id: number): void;
  (e: "cancel", id: number): void;
  (e: "open", folder: string): void;
}>();

const pct = computed(() => {
  if (props.entry.total_pages <= 0) return 0;
  return Math.round((props.entry.done_pages / props.entry.total_pages) * 100);
});

const statusLabel = computed(() => {
  switch (props.entry.status) {
    case "downloading":
      return $t("downloads.status_downloading");
    case "paused":
      return $t("downloads.status_paused");
    case "finished":
      return $t("downloads.status_finished");
    case "canceled":
      return $t("downloads.status_canceled");
    case "failed":
      return $t("downloads.status_failed");
    default:
      return $t("downloads.status_queued");
  }
});

const speedLabel = computed(() => {
  if (props.entry.status !== "downloading" || props.entry.bytes_per_second == null) return "";
  const bps = props.entry.bytes_per_second;
  if (bps >= 1024 * 1024) return `${(bps / (1024 * 1024)).toFixed(1)} MB/s`;
  if (bps >= 1024) return `${(bps / 1024).toFixed(0)} KB/s`;
  return `${bps.toFixed(0)} B/s`;
});
</script>

<template>
  <div class="item">
    <div class="info">
      <div class="title" :title="entry.title">{{ entry.title }}</div>
      <div class="sub">
        <span class="status" :data-status="entry.status">{{ statusLabel }}</span>
        <span v-if="speedLabel" class="speed">{{ speedLabel }}</span>
        <span class="pages">{{ entry.done_pages }}/{{ entry.total_pages }}</span>
      </div>
      <div class="bar">
        <div class="fill" :style="{ width: pct + '%' }" :data-status="entry.status"></div>
      </div>
    </div>
    <div class="actions">
      <button
        v-if="entry.status === 'downloading' || entry.status === 'pending'"
        :title="$t('downloads.pause')"
        @click="emit('pause', entry.id)"
      ><Pause :size="14" /></button>
      <button
        v-if="entry.status === 'paused' || entry.status === 'failed'"
        :title="$t('downloads.resume')"
        @click="emit('resume', entry.id)"
      ><Play :size="14" /></button>
      <button
        v-if="entry.status !== 'finished' && entry.status !== 'canceled'"
        :title="$t('downloads.cancel')"
        @click="emit('cancel', entry.id)"
      ><X :size="14" /></button>
      <button
        v-if="entry.status === 'finished'"
        :title="$t('downloads.open_folder')"
        @click="emit('open', entry.folder)"
      ><Folder :size="14" /></button>
    </div>
  </div>
</template>

<style scoped>
.item {
  display: flex;
  gap: 10px;
  padding: 10px 12px;
  background: var(--surface);
  border-radius: 8px;
  border: 1px solid var(--border);
}
.info {
  flex: 1;
  min-width: 0;
}
.title {
  font-size: 0.85rem;
  color: var(--text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.sub {
  display: flex;
  gap: 10px;
  margin: 2px 0 6px;
  font-size: 0.72rem;
  color: var(--text-dim);
}
.status[data-status="finished"] {
  color: #6ec16e;
}
.status[data-status="failed"] {
  color: #ff8e8e;
}
.speed {
  color: var(--accent);
  font-variant-numeric: tabular-nums;
}
.bar {
  height: 4px;
  background: var(--surface-3);
  border-radius: 2px;
  overflow: hidden;
}
.fill {
  height: 100%;
  background: var(--accent);
  transition: width 0.2s ease;
}
.fill[data-status="finished"] {
  background: #6ec16e;
}
.fill[data-status="failed"] {
  background: #ff8e8e;
}
.actions {
  display: flex;
  gap: 4px;
  align-items: center;
}
.actions button {
  background: var(--surface-2);
  border: 1px solid var(--border);
  color: var(--text);
  border-radius: 6px;
  width: 30px;
  height: 30px;
  cursor: pointer;
}
.actions button:hover {
  background: var(--surface-3);
}
</style>
