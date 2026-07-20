<script setup lang="ts">
import { onMounted } from "vue";

import DownloadItem from "@/components/DownloadItem.vue";
import EmptyState from "@/components/EmptyState.vue";
import { openPath } from "@/api";
import { useDownloadsStore } from "@/stores/downloads";

const downloads = useDownloadsStore();

onMounted(() => downloads.init());
</script>

<template>
  <div class="view">
    <div class="view-header">
      <div class="view-title">Downloads</div>
      <div class="toolbar">
        <button class="btn" @click="downloads.refresh()">Refresh</button>
        <button class="btn" @click="downloads.clear()">Clear finished</button>
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
    <EmptyState v-else title="No downloads" hint="Start a download from any gallery page." />
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
