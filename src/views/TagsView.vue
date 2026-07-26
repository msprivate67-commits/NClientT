<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { Ban, Plus, RefreshCw, ShieldCheck } from "@lucide/vue";

import TagChip from "@/components/TagChip.vue";
import EmptyState from "@/components/EmptyState.vue";
import { useTagsStore } from "@/stores/tags";
import { useSettingsStore } from "@/stores/settings";
import { useScrollCache } from "@/composables/useScrollCache";
import type { Tag, TagType } from "@/types";

const router = useRouter();
const { t: tagsI18n } = useI18n();
const tags = useTagsStore();
const settings = useSettingsStore();

const filter = ref<TagType | "all">("all");
const query = ref("");
const scope = ref<"all" | "accepted" | "avoided" | "blacklisted">("all");
const suggestions = ref<Tag[]>([]);
const loadingBlacklist = ref(false);
const actionError = ref("");
const hasApiKey = computed(() => Boolean(settings.settings.auth.api_key.trim()));
const viewRef = ref<HTMLElement | null>(null);
useScrollCache(viewRef);

const filtered = computed(() => {
  let list = tags.tags;
  if (scope.value === "accepted") list = list.filter((t) => t.status === "accepted");
  if (scope.value === "avoided") list = list.filter((t) => t.status === "avoided");
  if (scope.value === "blacklisted") list = list.filter((t) => t.blacklisted);
  if (filter.value !== "all") list = list.filter((t) => t.type === filter.value);
  if (query.value.trim()) {
    const q = query.value.toLowerCase();
    list = list.filter((t) => t.name.toLowerCase().includes(q));
  }
  return list.slice().sort((a, b) => b.count - a.count);
});

const types: (TagType | "all")[] = [
  "all",
  "tag",
  "artist",
  "character",
  "parody",
  "group",
  "language",
  "category",
];
const scopes = ["all", "accepted", "avoided", "blacklisted"] as const;

function onClick(t: any) {
  const name = encodeURIComponent(t.name);
  const type = encodeURIComponent(t.type);
  router.push({ name: "search", query: { tags: `${t.id}:accepted:${name}:${type}` } });
}

async function cycle(t: any) {
  await tags.cycle(t.id);
}

async function findTags() {
  actionError.value = "";
  try {
    suggestions.value = query.value.trim()
      ? await tags.search(query.value, 30)
      : [];
  } catch (cause) {
    actionError.value = String(cause);
  }
}

async function toggleBlacklist(tag: Tag) {
  actionError.value = "";
  if (!hasApiKey.value) {
    actionError.value = String(tagsI18n("tags.blacklist_requires_api_key"));
    return;
  }
  tags.merge([tag]);
  try {
    if (tags.blacklistedIds.has(tag.id)) {
      await tags.removeBlacklist(tag.id);
    } else {
      await tags.addBlacklist(tag.id);
    }
  } catch (cause) {
    actionError.value = String(cause);
  }
}

async function refreshBlacklist() {
  if (!hasApiKey.value) return;
  loadingBlacklist.value = true;
  actionError.value = "";
  try {
    await tags.loadBlacklist(true);
  } catch (cause) {
    actionError.value = String(cause);
  } finally {
    loadingBlacklist.value = false;
  }
}

onMounted(async () => {
  await tags.load();
  if (hasApiKey.value) await refreshBlacklist();
});
</script>

<template>
  <div ref="viewRef" class="view">
    <div class="view-header">
      <div class="view-title">{{ $t('tags.title') }}</div>
      <div class="toolbar">
        <input
          v-model="query"
          type="text"
          :placeholder="$t('tags.filter')"
          @input="findTags"
        />
        <button
          v-for="t in types"
          :key="t"
          class="btn"
          :class="{ primary: filter === t }"
          @click="filter = t"
        >{{ $t('tags.tag_type_' + t) }}</button>
        <button
          class="btn"
          :class="{ refreshing: loadingBlacklist }"
          :disabled="loadingBlacklist || !hasApiKey"
          type="button"
          @click="refreshBlacklist"
        >
          <RefreshCw :size="14" /> {{ $t('tags.sync_blacklist') }}
        </button>
      </div>
    </div>

    <div v-if="!hasApiKey" class="hint">{{ $t('tags.blacklist_requires_api_key') }}</div>
    <div class="toolbar scope-toolbar">
      <button
        v-for="value in scopes"
        :key="value"
        class="btn"
        :class="{ primary: scope === value }"
        type="button"
        @click="scope = value"
      >{{ $t('tags.scope_' + value) }}</button>
    </div>

    <div v-if="suggestions.length" class="suggestions">
      <div class="section-label">{{ $t('tags.search_results') }}</div>
      <span v-for="tag in suggestions" :key="tag.id" class="tag-row">
        <TagChip :tag="tag" show-type @click="onClick(tag)" />
        <button
          class="btn small"
          type="button"
          @click="toggleBlacklist(tag)"
        >
          <ShieldCheck v-if="tags.blacklistedIds.has(tag.id)" :size="13" />
          <Plus v-else :size="13" />
          {{ tags.blacklistedIds.has(tag.id)
            ? $t('tags.remove_blacklist')
            : $t('tags.add_blacklist') }}
        </button>
      </span>
    </div>

    <div v-if="actionError" class="error">{{ actionError }}</div>

    <div v-if="filtered.length" class="tag-list">
      <span v-for="t in filtered" :key="t.id" class="tag-row">
        <TagChip
          :tag="t"
          show-type
          @click="onClick(t)"
          @cycle="cycle(t)"
        />
        <button
          class="blacklist-btn"
          :class="{ active: tags.blacklistedIds.has(t.id) }"
          type="button"
          :title="tags.blacklistedIds.has(t.id)
            ? $t('tags.remove_blacklist')
            : $t('tags.add_blacklist')"
          @click="toggleBlacklist(t)"
        >
          <ShieldCheck v-if="tags.blacklistedIds.has(t.id)" :size="13" />
          <Ban v-else :size="13" />
        </button>
      </span>
    </div>
    <EmptyState v-else :title="$t('tags.no_tags')" :hint="$t('tags.no_tags_hint')" />
  </div>
</template>

<style scoped>
.toolbar input {
  background: var(--surface);
  border: 1px solid var(--border);
  color: var(--text);
  border-radius: 6px;
  padding: 4px 10px;
}
.scope-toolbar {
  margin-bottom: 12px;
}
.tag-list,
.suggestions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.suggestions {
  margin-bottom: 14px;
  padding: 10px;
  border: 1px solid var(--border);
  border-radius: 8px;
}
.section-label {
  width: 100%;
  color: var(--text-dim);
  font-size: 0.8rem;
}
.tag-row {
  display: inline-flex;
  align-items: center;
}
.blacklist-btn {
  display: inline-grid;
  place-items: center;
  width: 24px;
  height: 24px;
  margin-left: -4px;
  border: 1px solid var(--border);
  border-radius: 50%;
  background: var(--surface);
  color: var(--text-dim);
  cursor: pointer;
}
.blacklist-btn:hover,
.blacklist-btn.active {
  color: #ff8e8e;
  border-color: #e05252;
}
.error {
  margin-bottom: 12px;
  color: #ff9e9e;
}
.hint {
  margin-bottom: 12px;
  color: var(--text-dim);
  font-size: 0.8rem;
}
.btn.small {
  padding: 2px 8px;
  font-size: 0.72rem;
}
</style>
