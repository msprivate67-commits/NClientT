<script setup lang="ts">
import { ChevronLeft, ChevronRight } from "@lucide/vue";

defineProps<{
  currentPage: number;
  total: number;
}>();

function selectPage(event: Event) {
  const page = Number((event.target as HTMLInputElement).value);
  if (Number.isFinite(page)) {
    emit("select", page);
  }
}

const emit = defineEmits<{
  previous: [];
  next: [];
  select: [page: number];
}>();
</script>

<template>
  <footer class="reader-bar glass-surface glass-surface--dark">
    <button class="reader-btn" @click="$emit('previous')">
      <ChevronLeft :size="16" /> {{ $t('reader.prev') }}
    </button>
    <input
      class="reader-range"
      type="range"
      min="1"
      :max="Math.max(1, total)"
      :value="currentPage"
      @change="selectPage"
    />
    <button class="reader-btn" @click="$emit('next')">
      {{ $t('reader.next') }} <ChevronRight :size="16" />
    </button>
  </footer>
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
.reader-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
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
.reader-range {
  flex: 1;
  min-width: 60px;
}
</style>
