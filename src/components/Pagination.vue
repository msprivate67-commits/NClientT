<script setup lang="ts">
import { ChevronLeft, ChevronRight } from "@lucide/vue";

const props = defineProps<{ page: number; numPages: number }>();
const emit = defineEmits<{ (e: "change", page: number): void }>();

function go(p: number) {
  if (p < 1 || (props.numPages > 0 && p > props.numPages)) return;
  emit("change", p);
}
</script>

<template>
  <div class="pager">
    <button :disabled="page <= 1" @click="go(page - 1)"><ChevronLeft :size="16" /> {{ $t('common.prev') }}</button>
    <span class="info">
      <input
        v-if="numPages > 0"
        type="number"
        min="1"
        :max="numPages"
        :value="page"
        @change="go(Number(($event.target as HTMLInputElement).value))"
      />
      <span v-else>{{ page }}</span>
      <span v-if="numPages > 0" class="total">/ {{ numPages }}</span>
    </span>
    <button :disabled="numPages > 0 && page >= numPages" @click="go(page + 1)">{{ $t('common.next') }} <ChevronRight :size="16" /></button>
  </div>
</template>

<style scoped>
.pager {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 14px 0;
}
button {
  background: var(--surface);
  border: 1px solid var(--border);
  color: var(--text);
  padding: 6px 14px;
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 4px;
}
button:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
.info {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--text-dim);
  font-size: 0.85rem;
}
input {
  width: 56px;
  text-align: center;
  background: var(--surface);
  border: 1px solid var(--border);
  color: var(--text);
  border-radius: 6px;
  padding: 4px;
}
</style>
