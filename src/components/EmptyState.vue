<script setup lang="ts">
defineProps<{ title?: string; hint?: string }>();
</script>

<template>
  <div class="empty">
    <div class="title">{{ title ?? "Nothing here yet" }}</div>
    <div v-if="hint" class="hint">{{ hint }}</div>
    <slot />
  </div>
</template>

<style scoped>
.empty {
  padding: 60px 20px;
  text-align: center;
  color: var(--text-dim);
}
.title {
  font-size: 1rem;
  color: var(--text);
  margin-bottom: 6px;
}
.hint {
  font-size: 0.85rem;
}
</style>
