<script setup lang="ts">
import { ChevronDown, ChevronUp } from "@lucide/vue";

import TagChip from "@/components/TagChip.vue";
import type { Tag } from "@/types";

defineProps<{
  groups: Map<string, Tag[]>;
  expanded: boolean;
}>();

defineEmits<{
  "update:expanded": [expanded: boolean];
  select: [tag: Tag];
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
          <TagChip
            v-for="tag in tags"
            :key="tag.id"
            :tag="tag"
            show-type
            @click="$emit('select', tag)"
          />
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
</style>
