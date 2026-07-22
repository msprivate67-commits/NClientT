<script setup lang="ts">
import { onMounted, ref } from "vue";
import { Pause, Play, X, Trash2 } from "lucide-vue-next";

import DownloadItem from "@/components/DownloadItem.vue";
import EmptyState from "@/components/EmptyState.vue";
import { openPath } from "@/api";
import { useDownloadsStore } from "@/stores/downloads";
import { useScrollCache } from "@/composables/useScrollCache";

const downloads = useDownloadsStore();
const viewRef = ref<HTMLElement | null>(null);
useScrollCache(viewRef);

onMounted(() => downloads.init());
</script>

<template>
  <div ref="viewRef" class="view">
    <div class="view-header">
      <div class="view-title">{{ $t('downloads.title') }}</div>
      <div class="toolbar">
        <button class="btn" @click="downloads.refresh()">{{ $t('downloads.refresh') }}</button>
        <button class="btn" @click="downloads.clear()">{{ $t('downloads.clear_finished') }}</button>
      </div>
    </div>

    <!-- batch action bar -->
    <div v-if="downloads.selectedCount > 0" class="batch-bar">
      <span class="batch-label">{{ $t('downloads.selected_count', { n: downloads.selectedCount }) }}</span>
      <button class="btn sm" @click="downloads.selectAll()">
        {{ downloads.allSelected ? $t('downloads.deselect_all') : $t('downloads.select_all') }}
      </button>
      <div class="spacer"></div>
      <button class="btn sm" :title="$t('downloads.pause')" @click="downloads.batchPause()">
        <Pause :size="14" />
        <span>{{ $t('downloads.pause') }}</span>
      </button>
      <button class="btn sm" :title="$t('downloads.resume')" @click="downloads.batchResume()">
        <Play :size="14" />
        <span>{{ $t('downloads.resume') }}</span>
      </button>
      <button class="btn sm" :title="$t('downloads.cancel')" @click="downloads.batchCancel()">
        <X :size="14" />
        <span>{{ $t('downloads.cancel') }}</span>
      </button>
      <button class="btn sm danger" :title="$t('downloads.delete')" @click="downloads.batchDelete()">
        <Trash2 :size="14" />
        <span>{{ $t('downloads.delete') }}</span>
      </button>
    </div>

    <div v-if="downloads.items.length" class="list">
      <DownloadItem
        v-for="item in downloads.items"
        :key="item.id"
        :entry="item"
        :selected="downloads.selected.has(item.id)"
        :show-checkbox="true"
        @pause="downloads.pause"
        @resume="downloads.resume"
        @cancel="downloads.cancel"
        @delete="downloads.deleteDownload"
        @open="openPath"
        @toggle-select="downloads.toggleSelect"
      />
    </div>
    <EmptyState v-else :title="$t('downloads.no_downloads')" :hint="$t('downloads.no_downloads_hint')" />
  </div>
</template>

<style scoped>
.list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-width: 720px;
}
.batch-bar {
  display: flex;
  align-items: center;
  gap: 6px;
  max-width: 720px;
  padding: 8px 12px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  margin-bottom: 8px;
}
.batch-label {
  font-size: 0.8rem;
  color: var(--text-dim);
}
.spacer {
  flex: 1;
}
.btn.sm {
  font-size: 0.72rem;
  padding: 3px 8px;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.btn.danger {
  color: #ff8e8e;
  border-color: #ff8e8e55;
}
.btn.danger:hover {
  background: #ff8e8e11;
}
</style>
