<script setup lang="ts">
import { computed, onMounted, ref } from "vue";

import GalleryCard from "@/components/GalleryCard.vue";
import EmptyState from "@/components/EmptyState.vue";
import { ArrowUp, ArrowDown } from "lucide-vue-next";
import { localScan, localDelete } from "@/api";
import { useDownloadedStore } from "@/stores/downloaded";
import { useScrollCache } from "@/composables/useScrollCache";
import type { LocalGallery, SimpleGallery } from "@/types";

const items = ref<LocalGallery[]>([]);
const scanning = ref(false);
const viewRef = ref<HTMLElement | null>(null);
useScrollCache(viewRef);
const downloaded = useDownloadedStore();

type SortField = "name" | "date";
const sortField = ref<SortField>("date");
const sortAsc = ref(false);
const selectMode = ref(false);
const selectedIds = ref(new Set<number>());

const sorted = computed(() => {
  const arr = [...items.value];
  arr.sort((a, b) => {
    let cmp: number;
    if (sortField.value === "name") {
      cmp = (a.title || "").localeCompare(b.title || "");
    } else {
      cmp = (a.scanned_at || "").localeCompare(b.scanned_at || "");
    }
    return sortAsc.value ? cmp : -cmp;
  });
  return arr;
});

const allSelected = computed(() =>
  sorted.value.length > 0 && sorted.value.every((l) => selectedIds.value.has(l.id)),
);

function toggleSelectMode() {
  selectMode.value = !selectMode.value;
  if (!selectMode.value) {
    selectedIds.value = new Set();
  }
}

function toggleSelectAll() {
  if (allSelected.value) {
    selectedIds.value = new Set();
  } else {
    selectedIds.value = new Set(sorted.value.map((l) => l.id));
  }
}

async function deleteSelected() {
  const idsToDelete = [...selectedIds.value];
  if (!idsToDelete.length) return;
  // Find the corresponding folder for each selected ID.
  const folders = idsToDelete
    .map((id) => items.value.find((l) => l.id === id)?.folder)
    .filter(Boolean) as string[];
  selectedIds.value = new Set();
  // Delete one by one — the backend removes the folder from disk.
  for (const folder of folders) {
    try {
      await localDelete(folder);
    } catch { /* ignore individual failures */ }
  }
  // Remove deleted items from the local list and re-sync the downloaded set.
  const deletedSet = new Set(folders);
  items.value = items.value.filter((l) => !deletedSet.has(l.folder));
  await downloaded.refresh();
}

function onSelect(id: number) {
  selectedIds.value = new Set(selectedIds.value).add(id);
}

function onDeselect(id: number) {
  const next = new Set(selectedIds.value);
  next.delete(id);
  selectedIds.value = next;
}

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
    selectedIds.value = new Set();
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
      <div class="view-title">{{ $t('localLibrary.title') }}</div>
      <div class="toolbar">
        <button class="btn" :disabled="scanning" @click="scan">
          {{ scanning ? $t('localLibrary.scanning') : $t('localLibrary.rescan') }}
        </button>
      </div>
    </div>

    <div class="sort-row">
      <div class="sort-group">
        <span class="sort-label">{{ $t('localLibrary.sort_label') }}</span>
        <button
          class="btn small"
          :class="{ active: sortField === 'name' }"
          @click="sortField = 'name'"
        >{{ $t('localLibrary.sort_name') }}</button>
        <button
          class="btn small"
          :class="{ active: sortField === 'date' }"
          @click="sortField = 'date'"
        >{{ $t('localLibrary.sort_date') }}</button>
        <button class="btn small icon" @click="sortAsc = !sortAsc" :title="sortAsc ? $t('localLibrary.ascending') : $t('localLibrary.descending')">
          <ArrowUp v-if="sortAsc" :size="14" />
          <ArrowDown v-else :size="14" />
        </button>
      </div>
      <div class="select-group">
        <button
          class="btn small"
          :class="{ active: selectMode }"
          @click="toggleSelectMode"
        >{{ selectMode ? $t('localLibrary.cancel') : $t('localLibrary.select') }}</button>
        <template v-if="selectMode">
          <button class="btn small" @click="toggleSelectAll">
            {{ allSelected ? $t('localLibrary.deselect_all') : $t('localLibrary.select_all') }}
          </button>
          <button
            class="btn small danger"
            :disabled="selectedIds.size === 0"
            @click="deleteSelected"
          >
            {{ $t('localLibrary.delete_selected', { n: selectedIds.size }) }}
          </button>
        </template>
      </div>
    </div>

    <div v-if="sorted.length" class="grid">
      <GalleryCard
        v-for="l in sorted"
        :key="l.folder"
        :gallery="toSimple(l)"
        local
        :thumbnail-override="l.thumbnail_path"
        :selectable="selectMode"
        :selected="selectedIds.has(l.id)"
        @select="onSelect"
        @deselect="onDeselect"
      />
    </div>
    <EmptyState
      v-else
      :title="$t('localLibrary.no_local')"
      :hint="$t('localLibrary.no_local_hint')"
    />
  </div>
</template>

<style scoped>
.sort-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 14px;
}
.sort-group,
.select-group {
  display: flex;
  align-items: center;
  gap: 6px;
}
.sort-label {
  font-size: 0.82rem;
  color: var(--text-dim);
  margin-right: 2px;
}
.btn.small {
  padding: 3px 10px;
  font-size: 0.78rem;
  border-radius: 5px;
}
.btn.small.icon {
  padding: 3px 8px;
  font-size: 0.85rem;
  line-height: 1;
}
.btn.small.active {
  background: var(--accent);
  color: #fff;
  border-color: var(--accent);
}
.btn.small.danger {
  background: rgba(220, 60, 60, 0.15);
  border-color: rgba(220, 60, 60, 0.5);
  color: #f08080;
}
.btn.small.danger:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 12px;
}
@media (max-width: 560px) {
  .grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
  }
}
</style>
