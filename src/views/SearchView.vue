<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";

import GalleryGrid from "@/components/GalleryGrid.vue";
import Pagination from "@/components/Pagination.vue";
import TagChip from "@/components/TagChip.vue";
import { tagsSearch } from "@/api";
import { useGalleryStore } from "@/stores/gallery";
import { useSettingsStore } from "@/stores/settings";
import { useTagsStore } from "@/stores/tags";
import type { Language, SimpleGallery, SortType, Tag } from "@/types";

const route = useRoute();
const router = useRouter();
const gallery = useGalleryStore();
const settings = useSettingsStore();
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
    const [idStr, status] = part.split(":");
    const id = Number(idStr);
    if (!id) continue;
    out.push({
      id,
      name: "",
      type: "tag",
      count: 0,
      status: (status as Tag["status"]) ?? "accepted",
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
  if (/cloudflare/i.test(s)) return "Cloudflare challenge required. Open Settings → Cloudflare.";
  if (/401|403|unauthorized/i.test(s)) return "Authentication failed. Check your API key in Settings.";
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
  <div class="view">
    <div class="view-header">
      <div class="view-title">Search</div>
    </div>

    <form class="form" @submit.prevent="submit">
      <input v-model="query" type="text" placeholder="Title or keyword…" />
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
      <button type="submit" class="btn primary">Search</button>
      <button
        type="button"
        class="btn"
        :disabled="loading || !hasQuery"
        @click="load"
        title="Reload results"
      >
        {{ loading ? "Refreshing…" : "🔄 Refresh" }}
      </button>
    </form>

    <div class="field tag-input">
      <input
        v-model="tagQuery"
        type="text"
        placeholder="Add a tag filter…"
        @input="searchSuggestions"
        @keydown.enter.prevent="suggestions[0] && addTag(suggestions[0])"
      />
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
      >✕</button>
    </div>

    <div v-if="error" class="error">{{ error }}</div>

    <div v-if="hasQuery" class="results">
      <GalleryGrid :galleries="items" :loading="loading" empty-title="No matches" />
      <Pagination :page="page" :num-pages="numPages" @change="changePage" />
    </div>
    <div v-else class="hint-block">
      Type a query, pick a language, or add tags above to start searching.
    </div>
  </div>
</template>

<style scoped>
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
