<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useI18n } from "vue-i18n";

import GalleryGrid from "@/components/GalleryGrid.vue";
import Pagination from "@/components/Pagination.vue";
import TagChip from "@/components/TagChip.vue";
import { RefreshCw, X, CheckSquare } from "lucide-vue-next";
import { tagsSearch } from "@/api";
import { useGalleryStore } from "@/stores/gallery";
import { useSettingsStore } from "@/stores/settings";
import { useDownloadsStore } from "@/stores/downloads";
import { useTagsStore } from "@/stores/tags";
import { useScrollCache } from "@/composables/useScrollCache";
import type { Language, SimpleGallery, SortType, Tag } from "@/types";

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const gallery = useGalleryStore();
const settings = useSettingsStore();
const downloads = useDownloadsStore();
const tagsStore = useTagsStore();

const query = ref(String(route.query.q ?? ""));
const lang = ref<Language>((route.query.lang as Language) ?? settings.settings.only_language);
const sort = ref<SortType>((route.query.sort as SortType) ?? settings.settings.sort_type);
const selected = ref<Tag[]>(parseTagsParam(String(route.query.tags ?? "")));

const page = ref(Number(route.query.page ?? 1));
const numPages = ref(0);
const items = ref<SimpleGallery[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);
const viewRef = ref<HTMLElement | null>(null);
useScrollCache(viewRef);

const selectMode = ref(false);
const selectedIds = ref(new Set<number>());

function toggleSelectMode() {
  selectMode.value = !selectMode.value;
  selectedIds.value.clear();
}

function toggleSelect(id: number) {
  const s = selectedIds.value;
  if (s.has(id)) {
    s.delete(id);
  } else {
    s.add(id);
  }
  selectedIds.value = new Set(s);
}

function selectAllIds() {
  selectedIds.value = new Set(items.value.map((g) => g.id));
}

function deselectAllIds() {
  selectedIds.value.clear();
}

async function downloadSelected() {
  if (selectedIds.value.size === 0) return;
  for (const id of selectedIds.value) {
    await downloads.enqueue({ gallery_id: id });
  }
  selectMode.value = false;
  selectedIds.value.clear();
}

const tagQuery = ref("");
const suggestions = ref<Tag[]>([]);

const langs: { value: Language; label: string }[] = [
  { value: "all", label: "All" },
  { value: "english", label: "EN" },
  { value: "japanese", label: "JP" },
  { value: "chinese", label: "CN" },
];
const sorts: { value: SortType; label: string }[] = [
  { value: "recent_all_time", label: "Recent" },
  { value: "popular_all_time", label: "Popular" },
  { value: "popular_week", label: "Week" },
  { value: "popular_day", label: "Day" },
];

const hasQuery = computed(
  () => query.value.trim() || selected.value.length || lang.value !== "all",
);

function parseTagsParam(s: string): Tag[] {
  if (!s) return [];
  const out: Tag[] = [];
  for (const part of s.split(",")) {
    const segments = part.split(":");
    const id = Number(segments[0]);
    if (!id) continue;
    const status = (segments[1] as Tag["status"]) ?? "accepted";
    const name = segments.length > 2 ? decodeURIComponent(segments[2]) : "";
    const type = (segments.length > 3 ? decodeURIComponent(segments[3]) : "tag") as Tag["type"];
    out.push({
      id,
      name,
      type,
      count: 0,
      status,
    });
  }
  return out;
}

async function load() {
  loading.value = true;
  error.value = null;
  try {
    // Resolve tag names if missing.
    if (selected.value.some((t) => !t.name)) {
      const all = await tagsStore.load();
      for (const t of selected.value) {
        if (!t.name) {
          const found = all.find((x) => x.id === t.id);
          if (found) {
            t.name = found.name;
            t.type = found.type;
            t.count = found.count;
          }
        }
      }
    }
    const result = await gallery.search({
      query: query.value.trim(),
      tags: selected.value,
      page: page.value,
      sort: sort.value,
      only_language: lang.value,
    });
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
  if (/cloudflare/i.test(s)) return t("search.cf_error");
  if (/401|403|unauthorized/i.test(s)) return t("search.auth_error");
  return s;
}

async function searchSuggestions() {
  if (!tagQuery.value.trim()) {
    suggestions.value = [];
    return;
  }
  suggestions.value = await tagsSearch(tagQuery.value, 20);
}

function addTag(t: Tag) {
  if (!selected.value.some((x) => x.id === t.id)) {
    selected.value.push({ ...t, status: "accepted" });
  }
  tagQuery.value = "";
  suggestions.value = [];
  syncUrl();
  load();
}

function cycleTag(t: Tag) {
  const item = selected.value.find((x) => x.id === t.id);
  if (!item) return;
  item.status = item.status === "accepted" ? "avoided" : "accepted";
  syncUrl();
  load();
}

function removeTag(t: Tag) {
  selected.value = selected.value.filter((x) => x.id !== t.id);
  syncUrl();
  load();
}

function syncUrl() {
  const params: Record<string, string> = {};
  if (query.value.trim()) params.q = query.value.trim();
  if (lang.value !== "all") params.lang = lang.value;
  if (sort.value !== "recent_all_time") params.sort = sort.value;
  if (selected.value.length) params.tags = selected.value.map((t) => `${t.id}:${t.status}`).join(",");
  if (page.value > 1) params.page = String(page.value);
  router.replace({ query: params });
}

function submit() {
  page.value = 1;
  syncUrl();
  load();
}

function changePage(p: number) {
  page.value = p;
  viewRef.value?.scrollTo({ top: 0, behavior: "auto" });
  syncUrl();
  load();
}

onMounted(load);
watch(() => route.query, () => {
  // Sync from URL when navigated externally (e.g. from the search box).
  query.value = String(route.query.q ?? "");
  lang.value = (route.query.lang as Language) ?? lang.value;
  sort.value = (route.query.sort as SortType) ?? sort.value;
  selected.value = parseTagsParam(String(route.query.tags ?? ""));
  page.value = Number(route.query.page ?? 1);
  load();
});
</script>

<template>
  <div ref="viewRef" class="view">
    <div class="view-header">
      <div class="view-title">{{ $t('search.title') }}</div>
    </div>

    <div v-if="query.trim()" class="current-query">
      {{ $t('search.searching') }} <strong>{{ query }}</strong>
    </div>
    <div class="form">
      <div class="toolbar">
        <button
          v-for="l in langs"
          :key="l.value"
          class="btn"
          :class="{ primary: lang === l.value }"
          type="button"
          @click="lang = l.value; submit()"
        >{{ l.label }}</button>
      </div>
      <div class="toolbar">
        <button
          v-for="s in sorts"
          :key="s.value"
          class="btn"
          :class="{ primary: sort === s.value }"
          type="button"
          @click="sort = s.value; submit()"
        >{{ s.label }}</button>
      </div>
      <button
        type="button"
        class="btn"
        :class="{ refreshing: loading }"
        :disabled="loading || !hasQuery"
        :aria-busy="loading"
        @click="load"
        :title="$t('search.reload_results')"
      >
        {{ loading ? $t('common.refreshing') : '' }}<RefreshCw v-if="!loading" :size="14" /> {{ $t('common.refresh') }}
      </button>
    </div>

    <div class="field tag-input">
      <div class="tag-input-row">
        <input
          v-model="tagQuery"
          type="text"
          :placeholder="$t('search.add_tag')"
          @input="searchSuggestions"
          @keydown.enter.prevent="suggestions[0] && addTag(suggestions[0])"
        />
        <button
          type="button"
          class="btn primary small"
          :disabled="suggestions.length === 0"
          @click="suggestions[0] && addTag(suggestions[0])"
        >
          {{ $t('search.add') }}
        </button>
      </div>
      <div v-if="suggestions.length" class="suggest">
        <TagChip
          v-for="t in suggestions"
          :key="t.id"
          :tag="t"
          show-type
          @click="addTag(t)"
        />
      </div>
    </div>

    <div v-if="selected.length" class="chips selected">
      <TagChip
        v-for="t in selected"
        :key="t.id"
        :tag="t"
        show-type
        @click="cycleTag(t)"
      />
      <button
        v-for="t in selected"
        :key="'rm-' + t.id"
        class="btn small"
        type="button"
        @click="removeTag(t)"
      ><X :size="12" /></button>
    </div>

    <div v-if="error" class="error">{{ error }}</div>

    <div v-if="hasQuery" class="results">
      <div class="toolbar" style="margin-bottom: 10px;">
        <button
          class="btn"
          :class="{ primary: selectMode }"
          type="button"
          @click="toggleSelectMode"
        >
          {{ selectMode ? '' : '' }}<X v-if="selectMode" :size="14" /> {{ selectMode ? $t('common.cancel') : '' }}<CheckSquare v-if="!selectMode" :size="14" /> {{ !selectMode ? $t('common.select') : '' }}
        </button>
        <template v-if="selectMode">
          <button class="btn" type="button" @click="selectAllIds">{{ $t('common.select_all') }}</button>
          <button class="btn" type="button" @click="deselectAllIds">{{ $t('common.deselect_all') }}</button>
          <button class="btn primary" type="button" :disabled="selectedIds.size === 0" @click="downloadSelected">
            {{ $t('common.download') }} ({{ selectedIds.size }})
          </button>
        </template>
      </div>
      <GalleryGrid
        :galleries="items"
        :loading="loading"
        :empty-title="$t('search.no_matches')"
        :selectable="selectMode"
        :selected="selectedIds"
        @select="toggleSelect"
        @deselect="toggleSelect"
      />
      <Pagination :page="page" :num-pages="numPages" @change="changePage" />
    </div>
    <div v-else class="hint-block">
      {{ $t('search.search_hint') }}
    </div>
  </div>
</template>

<style scoped>
.current-query {
  margin-bottom: 10px;
  font-size: 0.9rem;
  color: var(--text-dim);
}
.form {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 12px;
}
.form input[type="text"] {
  flex: 1;
  min-width: 200px;
  background: var(--surface);
  border: 1px solid var(--border);
  color: var(--text);
  border-radius: 6px;
  padding: 6px 10px;
}
.tag-input {
  margin-bottom: 8px;
}
.tag-input-row {
  display: flex;
  gap: 6px;
}
.tag-input-row input {
  flex: 1;
  min-width: 150px;
  background: var(--surface);
  border: 1px solid var(--border);
  color: var(--text);
  border-radius: 6px;
  padding: 6px 10px;
}
.suggest {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 4px;
}
.selected {
  margin-bottom: 12px;
  align-items: center;
}
.btn.small {
  padding: 2px 8px;
  font-size: 0.72rem;
}
.error {
  padding: 12px 14px;
  background: rgba(255, 80, 80, 0.1);
  border: 1px solid rgba(255, 80, 80, 0.4);
  border-radius: 8px;
  color: #ff9e9e;
  margin-bottom: 14px;
  font-size: 0.85rem;
}
.hint-block {
  padding: 40px 20px;
  text-align: center;
  color: var(--text-dim);
}
.results {
  margin-top: 14px;
}
</style>
