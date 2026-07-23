<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";

import { Dices, RefreshCw } from "@lucide/vue";
import GalleryGrid from "@/components/GalleryGrid.vue";
import Pagination from "@/components/Pagination.vue";
import { useGalleryStore } from "@/stores/gallery";
import { useSettingsStore } from "@/stores/settings";
import { useDownloadsStore } from "@/stores/downloads";
import { useOverlayStore } from "@/stores/overlay";
import { useScrollCache } from "@/composables/useScrollCache";
import type { Language, SimpleGallery, SortType } from "@/types";
import { useI18n } from "vue-i18n";

const gallery = useGalleryStore();
const settings = useSettingsStore();
const downloads = useDownloadsStore();
const overlay = useOverlayStore();
const { t } = useI18n();

const page = ref(1);
const numPages = ref(0);
const items = ref<SimpleGallery[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);
const viewRef = ref<HTMLElement | null>(null);
useScrollCache(viewRef);

const selectMode = ref(false);
const selected = ref(new Set<number>());

function toggleSelectMode() {
  selectMode.value = !selectMode.value;
  selected.value.clear();
}

function toggleSelect(id: number) {
  const s = selected.value;
  if (s.has(id)) {
    s.delete(id);
  } else {
    s.add(id);
  }
  selected.value = new Set(s);
}

function selectAll() {
  selected.value = new Set(items.value.map((g) => g.id));
}

function deselectAll() {
  selected.value.clear();
}

async function downloadSelected() {
  if (selected.value.size === 0) return;
  for (const id of selected.value) {
    await downloads.enqueue({ gallery_id: id });
  }
  selectMode.value = false;
  selected.value.clear();
}

const sorts = computed(() => [
  { value: "recent_all_time" as SortType, label: t("home.sort_recent") },
  { value: "popular_all_time" as SortType, label: t("home.sort_popular_all") },
  { value: "popular_week" as SortType, label: t("home.sort_popular_week") },
  { value: "popular_day" as SortType, label: t("home.sort_popular_day") },
  { value: "popular_month" as SortType, label: t("home.sort_popular_month") },
]);

const langs = computed(() => [
  { value: "all" as Language, label: t("home.lang_all") },
  { value: "english" as Language, label: t("home.lang_en") },
  { value: "japanese" as Language, label: t("home.lang_jp") },
  { value: "chinese" as Language, label: t("home.lang_cn") },
]);

async function load() {
  loading.value = true;
  error.value = null;
  try {
    const sort = settings.settings.sort_type;
    const result = await gallery.browse(page.value, sort);
    items.value = result.galleries;
    numPages.value = result.num_pages;
  } catch (e: any) {
    error.value = humanizeError(e);
  } finally {
    loading.value = false;
  }
}

function humanizeError(e: any): string {
  const s = String(e?.message ?? e);
  if (/cloudflare/i.test(s)) return t("home.cf_error");
  if (/401|403|unauthorized/i.test(s)) return t("home.auth_error");
  return s;
}

function changeSort(s: SortType) {
  settings.save({ sort_type: s }).then(load);
}

function changeLanguage(l: Language) {
  if (settings.settings.only_language === l) return;
  // Reset to the first page so we don't land beyond the filtered total.
  page.value = 1;
  settings.save({ only_language: l }).then(load);
}

function changePage(nextPage: number) {
  page.value = nextPage;
  viewRef.value?.scrollTo({ top: 0, behavior: "auto" });
}

onMounted(load);
watch(page, load);
</script>

<template>
  <div ref="viewRef" class="view">
    <div class="view-header">
      <div class="view-title">{{ $t('home.title') }}</div>
      <div class="toolbar">
        <button
          class="btn"
          :class="{ primary: selectMode }"
          @click="toggleSelectMode"
        >
          {{ selectMode ? $t('common.cancel') : $t('common.select') }}
        </button>
        <template v-if="selectMode">
          <button class="btn" @click="selectAll">{{ $t('common.select_all') }}</button>
          <button class="btn" @click="deselectAll">{{ $t('common.deselect_all') }}</button>
          <button class="btn primary" :disabled="selected.size === 0" @click="downloadSelected">
            {{ $t('common.download') }} ({{ selected.size }})
          </button>
        </template>
        <template v-else>
        <button
          v-for="s in sorts"
          :key="s.value"
          class="btn"
          :class="{ primary: settings.settings.sort_type === s.value }"
          @click="changeSort(s.value)"
        >
          {{ s.label }}
        </button>
        <button class="btn" @click="gallery.random().then((g) => overlay.openGallery(g.id))">
          <Dices :size="14" /> {{ $t('home.random') }}
        </button>
        <button
          class="btn"
          :class="{ refreshing: loading }"
          :disabled="loading"
          :aria-busy="loading"
          @click="load"
          :title="$t('home.reload_galleries')"
        >
          {{ loading ? $t('common.refreshing') : '' }}<RefreshCw v-if="!loading" :size="14" /> {{ $t('common.refresh') }}
        </button>
        <div class="lang-group">
          <span class="lang-label">{{ $t('home.lang_label') }}</span>
          <button
            v-for="l in langs"
            :key="l.value"
            class="btn small"
            :class="{ primary: settings.settings.only_language === l.value }"
            :title="l.value === 'all' ? $t('home.all_languages') : l.label"
            @click="changeLanguage(l.value)"
          >
            {{ l.label }}
          </button>
        </div>
        </template>
      </div>
    </div>

    <div v-if="error" class="error">{{ error }}</div>

    <GalleryGrid
      :galleries="items"
      :loading="loading"
      :empty-title="$t('home.no_galleries')"
      :empty-hint="$t('home.no_galleries_hint')"
      :selectable="selectMode"
      :selected="selected"
      @select="toggleSelect"
      @deselect="toggleSelect"
    />

    <Pagination :page="page" :num-pages="numPages" @change="changePage" />
  </div>
</template>

<style scoped>
.error {
  padding: 12px 14px;
  background: rgba(255, 80, 80, 0.1);
  border: 1px solid rgba(255, 80, 80, 0.4);
  border-radius: 8px;
  color: #ff9e9e;
  margin-bottom: 14px;
  font-size: 0.85rem;
}
.lang-group {
  display: flex;
  align-items: center;
  gap: 4px;
  padding-left: 8px;
  margin-left: 4px;
  border-left: 1px solid var(--border);
}
.lang-label {
  font-size: 0.78rem;
  color: var(--text-dim);
  margin-right: 2px;
}
.btn.small {
  padding: 2px 8px;
  font-size: 0.72rem;
}
</style>
