<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";

import TagChip from "@/components/TagChip.vue";
import EmptyState from "@/components/EmptyState.vue";
import { useTagsStore } from "@/stores/tags";
import { useScrollCache } from "@/composables/useScrollCache";
import type { TagType } from "@/types";

const router = useRouter();
const tags = useTagsStore();

const filter = ref<TagType | "all">("all");
const query = ref("");
const viewRef = ref<HTMLElement | null>(null);
useScrollCache(viewRef);

const filtered = computed(() => {
  let list = tags.tags;
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

function onClick(t: any) {
  const name = encodeURIComponent(t.name);
  const type = encodeURIComponent(t.type);
  router.push({ name: "search", query: { tags: `${t.id}:accepted:${name}:${type}` } });
}

async function cycle(t: any) {
  await tags.cycle(t.id);
}

onMounted(() => tags.load());
</script>

<template>
  <div ref="viewRef" class="view">
    <div class="view-header">
      <div class="view-title">Tags</div>
      <div class="toolbar">
        <input v-model="query" type="text" placeholder="Filter…" />
        <button
          v-for="t in types"
          :key="t"
          class="btn"
          :class="{ primary: filter === t }"
          @click="filter = t"
        >{{ t }}</button>
      </div>
    </div>

    <div v-if="filtered.length" class="chips">
      <TagChip
        v-for="t in filtered"
        :key="t.id"
        :tag="t"
        show-type
        @click="onClick(t)"
        @cycle="cycle(t)"
      />
    </div>
    <EmptyState v-else title="No tags loaded" hint="Browse some galleries first to populate the tag cache." />
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
.chips {
  gap: 6px;
}
</style>
