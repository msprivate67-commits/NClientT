<script setup lang="ts">
import { ArrowLeftRight, ArrowUpDown, X } from "@lucide/vue";

import type { ReaderDirection, ReaderFitMode } from "@/composables/useReaderNavigation";

defineProps<{
  currentPage: number;
  total: number;
  fitMode: ReaderFitMode;
  direction: ReaderDirection;
}>();

defineEmits<{
  close: [];
  "update:fitMode": [mode: ReaderFitMode];
  "update:direction": [direction: ReaderDirection];
}>();
</script>

<template>
  <header class="reader-bar glass-surface glass-surface--dark">
    <button class="reader-btn" @click="$emit('close')"><X :size="16" /></button>
    <span class="reader-counter">{{ currentPage }} / {{ total || "?" }}</span>
    <div class="reader-fit">
      <button
        class="reader-btn small icon-only"
        :title="direction === 'vertical' ? $t('reader.horizontal') : $t('reader.vertical')"
        @click="$emit('update:direction', direction === 'vertical' ? 'horizontal' : 'vertical')"
      >
        <ArrowLeftRight v-if="direction === 'vertical'" :size="14" />
        <ArrowUpDown v-else :size="14" />
      </button>
      <button
        v-for="mode in (['height', 'width', 'original'] as ReaderFitMode[])"
        :key="mode"
        class="reader-btn small"
        :class="{ primary: fitMode === mode }"
        @click="$emit('update:fitMode', mode)"
      >
        {{ $t(`reader.fit_${mode}`) }}
      </button>
    </div>
    <slot name="actions" />
  </header>
</template>

<style scoped>
.reader-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 14px;
  background: rgba(0, 0, 0, 0.85);
  color: #fff;
  flex-shrink: 0;
  z-index: 2;
  flex-wrap: wrap;
}
.reader-counter {
  font-size: 0.85rem;
  white-space: nowrap;
}
.reader-fit {
  margin-left: auto;
  display: flex;
  gap: 4px;
}
.reader-btn {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: #fff;
  padding: 6px 14px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.85rem;
}
.reader-btn:hover {
  background: rgba(255, 255, 255, 0.15);
}
.reader-btn.primary {
  background: var(--accent);
  border-color: var(--accent);
}
.reader-btn.small {
  padding: 2px 8px;
  font-size: 0.72rem;
}
.reader-btn.icon-only {
  padding: 4px 6px;
}
</style>
