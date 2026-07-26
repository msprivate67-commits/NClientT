<script setup lang="ts">
import { Ban, ChevronDown, ChevronUp, ShieldCheck } from "@lucide/vue";

import TagChip from "@/components/TagChip.vue";
import type { Tag } from "@/types";

defineProps<{
  groups: Map<string, Tag[]>;
  expanded: boolean;
  blacklistedIds?: Set<number>;
}>();

defineEmits<{
  "update:expanded": [expanded: boolean];
  select: [tag: Tag];
  toggleBlacklist: [tag: Tag];
}>();
</script>

<template>
  <section class="detail-card tags-card">
    <div class="section-toggle-bar tag-toggle-bar">
      <div class="section-title">{{ $t('tags.title') }}</div>
      <button
        class="btn small"
        type="button"
        :aria-expanded="expanded"
        @click="$emit('update:expanded', !expanded)"
      >
        <ChevronUp v-if="expanded" :size="14" />
        <ChevronDown v-else :size="14" />
        {{ expanded ? $t('gallery.collapse') : $t('gallery.expand') }}
      </button>
    </div>
    <div v-show="expanded" class="tags-content">
      <section v-for="[type, tags] in groups" :key="type" class="tag-group">
        <div class="section-title">{{ type }}</div>
        <div class="chips">
          <span v-for="tag in tags" :key="tag.id" class="tag-with-action">
            <TagChip
              :tag="{ ...tag, blacklisted: blacklistedIds?.has(tag.id) ?? tag.blacklisted }"
              show-type
              @click="$emit('select', tag)"
            />
            <button
              class="blacklist-action"
              type="button"
              :class="{ active: blacklistedIds?.has(tag.id) ?? tag.blacklisted }"
              :title="(blacklistedIds?.has(tag.id) ?? tag.blacklisted)
                ? $t('tags.remove_blacklist')
                : $t('tags.add_blacklist')"
              @click="$emit('toggleBlacklist', tag)"
            >
              <ShieldCheck v-if="blacklistedIds?.has(tag.id) ?? tag.blacklisted" :size="12" />
              <Ban v-else :size="12" />
            </button>
          </span>
        </div>
      </section>
    </div>
  </section>
</template>

<style scoped>
.section-toggle-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.section-toggle-bar .section-title {
  margin: 0;
  margin-right: auto;
}
.tags-content {
  margin-top: 12px;
}
.tag-group {
  margin-bottom: 14px;
}
.tag-group:last-child {
  margin-bottom: 0;
}
.tag-with-action {
  display: inline-flex;
  align-items: center;
}
.blacklist-action {
  display: inline-grid;
  place-items: center;
  width: 22px;
  height: 22px;
  margin-left: -4px;
  border: 1px solid var(--border);
  border-radius: 50%;
  background: var(--surface);
  color: var(--text-dim);
  cursor: pointer;
}
.blacklist-action:hover,
.blacklist-action.active {
  border-color: #e05252;
  color: #ff8e8e;
}
</style>
