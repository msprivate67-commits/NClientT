<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch, nextTick } from "vue";
import { useRoute, useRouter } from "vue-router";

import { imageProxyUrl } from "@/api";
import { useGalleryStore } from "@/stores/gallery";
import { useSettingsStore } from "@/stores/settings";
import { useOverlayStore } from "@/stores/overlay";

const props = defineProps<{ id: number | string; overlay?: boolean }>();
const emit = defineEmits<{ back: [] }>();
const route = useRoute();
const router = useRouter();
const gallery = useGalleryStore();
const settings = useSettingsStore();
const overlay = useOverlayStore();

const id = computed(() => Number(props.id));
const fitMode = ref<"width" | "height" | "original">(
  fitFromZoom(settings.settings.default_zoom_pct),
);

function fitFromZoom(z: number): "width" | "height" | "original" {
  if (z <= 50) return "height";
  if (z >= 150) return "original";
  return "width";
}

const pages = computed(() => gallery.current?.pages ?? []);
const total = computed(() => pages.value.length);
const rtl = computed(() => settings.settings.use_rtl);

const scrollRef = ref<HTMLElement | null>(null);
const currentPage = ref(1);

function pageSrc(i: number): string {
  return imageProxyUrl(pages.value[i]?.path ?? "");
}

const preloaded = new Set<number>();
const PRELOAD_BUF = 3;

function preloadNearby() {
  const start = Math.max(0, currentPage.value - PRELOAD_BUF);
  const end = Math.min(total.value, currentPage.value + PRELOAD_BUF);
  for (let i = start; i < end; i++) {
    if (preloaded.has(i)) continue;
    preloaded.add(i);
    const p = pages.value[i]?.path;
    if (p) {
      const img = new Image();
      img.src = imageProxyUrl(p);
      img.onerror = () => preloaded.delete(i);
    }
  }
}

function computeCurrentPage() {
  if (!scrollRef.value || !total.value) return;
  const container = scrollRef.value;
  const viewCenter = container.scrollTop + container.clientHeight / 2;

  let best = 0;
  let bestDist = Infinity;
  const imgs = container.querySelectorAll<HTMLElement>(".page-img");
  for (let i = 0; i < imgs.length; i++) {
    const el = imgs[i];
    const top = el.offsetTop;
    const center = top + el.offsetHeight / 2;
    const dist = Math.abs(viewCenter - center);
    if (dist < bestDist) {
      bestDist = dist;
      best = i;
    }
  }
  currentPage.value = best + 1;
}

let scrollTimer: ReturnType<typeof setTimeout> | null = null;
function onScroll() {
  if (scrollTimer) clearTimeout(scrollTimer);
  scrollTimer = setTimeout(() => {
    computeCurrentPage();
    preloadNearby();
  }, 150);
}

function scrollToPage(idx: number, smooth = true) {
  if (!scrollRef.value || idx < 0 || idx >= total.value) return;
  const el = scrollRef.value.querySelectorAll<HTMLElement>(".page-img")[idx];
  if (!el) return;
  el.scrollIntoView({ behavior: smooth ? "smooth" : "auto", block: "start" });
}

function goPage(delta: number) {
  const next = currentPage.value - 1 + delta;
  if (next < 0 || next >= total.value) return;
  scrollToPage(next);
}

function prev() {
  goPage(rtl.value ? 1 : -1);
}
function next() {
  goPage(rtl.value ? -1 : 1);
}

function onKey(e: KeyboardEvent) {
  if (e.key === "ArrowRight") {
    e.preventDefault();
    rtl.value ? prev() : next();
  } else if (e.key === "ArrowLeft") {
    e.preventDefault();
    rtl.value ? next() : prev();
  } else if (e.key === "Escape") {
    if (props.overlay) {
      emit("back");
    } else {
      router.back();
    }
  }
}

async function load() {
  if (!gallery.current || gallery.current.id !== id.value) {
    await gallery.load(id.value);
  }
  const start = props.overlay
    ? overlay.readerPage
    : Number(route.query.page) || null;
  preloaded.clear();
  await nextTick();
  if (start && start > 0 && start <= total.value) {
    currentPage.value = start;
    scrollToPage(start - 1, false);
  } else {
    currentPage.value = 1;
    if (scrollRef.value) {
      scrollRef.value.scrollTop = 0;
    }
  }
  preloadNearby();
  if (props.overlay) overlay.readerPage = null;
}

onMounted(() => {
  load();
  window.addEventListener("keydown", onKey);
});
onUnmounted(() => window.removeEventListener("keydown", onKey));
watch(id, load);
watch(fitMode, () => {
  nextTick(() => {
    if (scrollRef.value && currentPage.value > 1) {
      scrollToPage(currentPage.value - 1, false);
    }
  });
});
</script>

<template>
  <div class="reader" :class="[`fit-${fitMode}`, { rtl }]">
    <header class="bar">
      <button class="btn" @click="props.overlay ? emit('back') : router.back()">✕ Close</button>
      <span class="counter">{{ currentPage }} / {{ total || "?" }}</span>
      <div class="fit">
        <button
          class="btn small"
          :class="{ primary: fitMode === 'height' }"
          @click="fitMode = 'height'"
        >
          Fit H
        </button>
        <button
          class="btn small"
          :class="{ primary: fitMode === 'width' }"
          @click="fitMode = 'width'"
        >
          Fit W
        </button>
        <button
          class="btn small"
          :class="{ primary: fitMode === 'original' }"
          @click="fitMode = 'original'"
        >
          1:1
        </button>
      </div>
    </header>

    <div ref="scrollRef" class="scroll-strip" @scroll="onScroll">
      <div v-if="!total" class="loading">Loading…</div>
      <img
        v-for="(_p, i) in pages"
        :key="i"
        :src="pageSrc(i)"
        :alt="`page ${i + 1}`"
        loading="lazy"
        decoding="async"
        class="page-img"
      />
    </div>

    <footer class="bar">
      <button class="btn" @click="prev">‹ Prev</button>
      <input
        type="range"
        min="1"
        :max="Math.max(1, total)"
        v-model.number="currentPage"
        @change="scrollToPage(currentPage - 1, false)"
      />
      <button class="btn" @click="next">Next ›</button>
    </footer>
  </div>
</template>

<style scoped>
.reader {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  background: #000;
}
.bar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 14px;
  background: rgba(0, 0, 0, 0.85);
  color: #fff;
  flex-shrink: 0;
  z-index: 2;
}
.counter {
  font-size: 0.85rem;
  white-space: nowrap;
}
.fit {
  margin-left: auto;
  display: flex;
  gap: 4px;
}
.btn {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: #fff;
  padding: 6px 14px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.85rem;
}
.btn:hover {
  background: rgba(255, 255, 255, 0.15);
}
.btn.primary {
  background: var(--accent);
  border-color: var(--accent);
}
.btn.small {
  padding: 2px 8px;
  font-size: 0.72rem;
}

.scroll-strip {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  scroll-behavior: auto;
  background: #000;
}

.scroll-strip img {
  display: block;
  margin: 0 auto 2px;
  min-height: 1px;
}

.fit-height .scroll-strip img {
  height: 100%;
  width: auto;
  max-width: 100%;
  object-fit: contain;
}

.fit-width .scroll-strip img {
  width: 100%;
  height: auto;
}

.fit-original .scroll-strip {
  overflow-x: auto;
}
.fit-original .scroll-strip img {
  max-width: none;
  max-height: none;
}

.loading {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #888;
  font-size: 1rem;
}

input[type="range"] {
  flex: 1;
  min-width: 60px;
}
</style>
