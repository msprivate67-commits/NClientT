<script setup lang="ts">
import { ChevronDown, ChevronUp } from "@lucide/vue";

import GalleryGrid from "@/components/GalleryGrid.vue";
import type { SimpleGallery } from "@/types";

withDefaults(defineProps<{
  galleries: SimpleGallery[];
  collapsible?: boolean;
  expanded?: boolean;
}>(), {
  collapsible: true,
  expanded: true,
});

defineEmits<{
  "update:expanded": [expanded: boolean];
}>();
</script>

<template>
  <section class="related detail-card">
    <div class="section-toggle-bar">
      <div class="section-title">{{ $t('gallery.section_related') }}</div>
      <button
        v-if="collapsible"
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
    <div v-show="!collapsible || expanded" class="related-content">
      <GalleryGrid :galleries="galleries" />
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
.related-content {
  margin-top: 12px;
}
</style>
