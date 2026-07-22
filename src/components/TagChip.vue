<script setup lang="ts">
import { computed } from "vue";

import type { Tag, TagStatus } from "@/types";

const props = withDefaults(
  defineProps<{
    tag: Tag;
    clickable?: boolean;
    showType?: boolean;
  }>(),
  { clickable: true, showType: false },
);

const emit = defineEmits<{
  (e: "click", tag: Tag): void;
  (e: "cycle", tag: Tag): void;
}>();

const status = computed<TagStatus>(() => props.tag.status ?? "default");
const statusClass = computed(() => `status-${status.value}`);

const typeLabel = computed(() => {
  const t = props.tag.type;
  return t === "unknown" ? "" : t;
});

function onClick() {
  if (!props.clickable) return;
  emit("click", props.tag);
}

function onContextmenu(e: MouseEvent) {
  if (!props.clickable) return;
  e.preventDefault();
  emit("cycle", props.tag);
}
</script>

<template>
  <span
    class="chip"
    :class="[statusClass, { clickable }]"
    :title="$t('tags.cycle_hint', { type: tag.type, name: tag.name, count: tag.count })"
    @click="onClick"
    @contextmenu="onContextmenu"
  >
    <span v-if="showType && typeLabel" class="type">{{ typeLabel }}:</span>
    <span class="name">{{ tag.name }}</span>
    <span v-if="tag.count" class="count">{{ formatCount(tag.count) }}</span>
  </span>
</template>

<script lang="ts">
function formatCount(n: number): string {
  if (n >= 1000) return (n / 1000).toFixed(1).replace(/\.0$/, "") + "k";
  return String(n);
}
</script>

<style scoped>
.chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 0.78rem;
  background: var(--surface-2);
  color: var(--text);
  border: 1px solid var(--border);
  user-select: none;
  white-space: nowrap;
}
.chip.clickable {
  cursor: pointer;
}
.chip.clickable:hover {
  background: var(--surface-3);
}
.type {
  color: var(--text-dim);
  font-size: 0.72rem;
}
.count {
  color: var(--text-dim);
  font-size: 0.7rem;
}
.status-accepted {
  background: var(--accent-soft);
  border-color: var(--accent);
  color: var(--accent);
}
.status-avoided {
  background: rgba(220, 50, 50, 0.15);
  border-color: rgba(220, 50, 50, 0.6);
  color: #ff8e8e;
  text-decoration: line-through;
}
</style>
