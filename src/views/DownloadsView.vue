<script setup lang="ts">
import { onMounted, ref } from "vue";

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

    <div v-if="downloads.items.length" class="list">
      <DownloadItem
        v-for="item in downloads.items"
        :key="item.id"
        :entry="item"
        @pause="downloads.pause"
        @resume="downloads.resume"
        @cancel="downloads.cancel"
        @open="openPath"
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
</style>
