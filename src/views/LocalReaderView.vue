<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, nextTick, watch } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";

import { imageProxyUrl, localList, localDelete, readProgressGet } from "@/api";
import { X, ArrowLeftRight, ArrowUpDown, AlertTriangle, ChevronLeft, ChevronRight } from "lucide-vue-next";
import { useReadProgressStore } from "@/stores/readProgress";
import { useSettingsStore } from "@/stores/settings";
import type { LocalGallery } from "@/types";

const props = defineProps<{ folder: number | string; overlay?: boolean }>();
const emit = defineEmits<{ back: [] }>();
const router = useRouter();

const local = ref<LocalGallery | null>(null);
const settings = useSettingsStore();
const fitMode = ref<"width" | "height" | "original">(
  (settings.settings.reader_fit_mode as "width" | "height" | "original") || "height",
);
const scrollMode = ref<"vertical" | "horizontal">(
  (settings.settings.reader_direction as "vertical" | "horizontal") || "vertical",
);
const readProgress = useReadProgressStore();
const { t } = useI18n();

const pages = computed(() => local.value?.page_files ?? []);
const total = computed(() => pages.value.length);

const scrollRef = ref<HTMLElement | null>(null);
const currentPage = ref(1);
const failedPages = ref(new Set<number>());
const retries = ref(new Map<number, number>());

// Track how many page images have loaded so we don't record spurious
// page numbers while the layout is still settling.
const pagesLoaded = ref(0);
function onPageLoaded(i: number) {
  pagesLoaded.value = Math.max(pagesLoaded.value, i + 1);
}

function pageSrc(i: number): string {
  const url = imageProxyUrl(pages.value[i] ?? "");
  const r = retries.value.get(i);
  if (r && r > 0 && url) {
    const sep = url.includes("?") ? "&" : "?";
    return `${url}${sep}_retry=${r}`;
  }
  return url;
}

function onImageError(i: number) {
  const s = new Set(failedPages.value);
  s.add(i);
  failedPages.value = s;
}

function reloadPage(i: number) {
  const s = new Set(failedPages.value);
  s.delete(i);
  failedPages.value = s;
  const m = new Map(retries.value);
  m.set(i, (m.get(i) ?? 0) + 1);
  retries.value = m;
}

function computeCurrentPage() {
  if (!scrollRef.value || !total.value) return;
  const container = scrollRef.value;
  const isH = scrollMode.value === "horizontal";
  const viewCenter = isH
    ? container.scrollLeft + container.clientWidth / 2
    : container.scrollTop + container.clientHeight / 2;

  let best = 0;
  let bestDist = Infinity;
  const wraps = container.querySelectorAll<HTMLElement>(".page-wrap");
  for (let i = 0; i < wraps.length; i++) {
    const el = wraps[i];
    const pos = isH ? el.offsetLeft : el.offsetTop;
    const size = isH ? el.offsetWidth : el.offsetHeight;
    const center = pos + size / 2;
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
    reportProgress();
  }, 150);
}

/**
 * Report the page the user is actually looking at. We avoid the
 * high-water-mark approach used by the online reader because in the
 * local reader `computeCurrentPage()` can be inaccurate before page
 * images finish loading (empty elements collapse to zero height).
 * Instead we only report after nearby images are confirmed loaded.
 */
function reportProgress() {
  const totalVal = total.value;
  if (!totalVal || !local.value) return;
  const page = currentPage.value;
  const gid = local.value.id;
  if (gid <= 0 || page <= 0) return;
  // Don't report if the image we think we're on hasn't loaded yet.
  if (pagesLoaded.value < page) return;
  void readProgress.report(gid, page, totalVal);
}

function scrollToPage(idx: number, smooth = true) {
  if (!scrollRef.value || idx < 0 || idx >= total.value) return;
  const el = scrollRef.value.querySelectorAll<HTMLElement>(".page-wrap")[idx];
  if (!el) return;
  const container = scrollRef.value;
  if (scrollMode.value === "horizontal") {
    container.scrollTo({ left: el.offsetLeft, behavior: smooth ? "smooth" : "auto" });
  } else {
    container.scrollTo({ top: el.offsetTop, behavior: smooth ? "smooth" : "auto" });
  }
}

function prev() {
  scrollToPage(currentPage.value - 2);
}
function next() {
  scrollToPage(currentPage.value);
}

function onKey(e: KeyboardEvent) {
  if (e.key === "ArrowRight" || e.key === "ArrowDown") {
    e.preventDefault();
    next();
  } else if (e.key === "ArrowLeft" || e.key === "ArrowUp") {
    e.preventDefault();
    prev();
  } else if (e.key === "Escape") {
    props.overlay ? emit("back") : router.back();
  }
}

async function load() {
  const all = await localList();
  local.value =
    all.find((l) => String(l.id) === String(props.folder)) ?? null;
  failedPages.value.clear();
  retries.value.clear();
  pagesLoaded.value = 0;
  await nextTick();
  currentPage.value = 1;
  if (scrollRef.value) {
    scrollRef.value.scrollTop = 0;
  }

  if (local.value && local.value.id > 0) {
    try {
      const progress = await readProgressGet(local.value.id);
      if (progress && progress.last_page > 1 && progress.last_page <= total.value) {
        await nextTick();
        currentPage.value = progress.last_page;
        scrollToPage(currentPage.value - 1, false);
      }
    } catch { /* ignore — resume is best-effort */ }
  }
}

onMounted(() => {
  load();
  window.addEventListener("keydown", onKey);
});

watch(() => props.folder, () => {
  load();
});
onUnmounted(() => {
  window.removeEventListener("keydown", onKey);
  // Always flush the exact page on exit, bypassing the loaded guard.
  if (local.value && local.value.id > 0 && currentPage.value > 0) {
    void readProgress.report(local.value.id, currentPage.value, total.value);
  }
});

watch(fitMode, () => {
  nextTick(() => {
    if (scrollRef.value && currentPage.value > 1) {
      scrollToPage(currentPage.value - 1, false);
    }
  });
  settings.save({ reader_fit_mode: fitMode.value });
});
watch(scrollMode, () => {
  settings.save({ reader_direction: scrollMode.value });
});

async function remove() {
  if (!local.value) return;
  if (!confirm(t("reader.confirm_delete", { title: local.value.title }))) return;
  await localDelete(local.value.folder);
  if (props.overlay) {
    emit("back");
  } else {
    router.push({ name: "local" });
  }
}
</script>

<template>
  <div class="reader" :class="[`fit-${fitMode}`, `direction-${scrollMode}`]">
    <header class="bar">
      <button class="btn" @click="props.overlay ? emit('back') : router.back()"><X :size="16" /></button>
      <span class="counter">{{ currentPage }} / {{ total || "?" }}</span>
      <button
        class="btn small icon-only"
        :title="scrollMode === 'vertical' ? $t('reader.horizontal') : $t('reader.vertical')"
        @click="scrollMode = scrollMode === 'vertical' ? 'horizontal' : 'vertical'"
      >
        <ArrowLeftRight v-if="scrollMode === 'vertical'" :size="14" />
        <ArrowUpDown v-if="scrollMode === 'horizontal'" :size="14" />
      </button>
      <div class="fit">
        <button
          class="btn small"
          :class="{ primary: fitMode === 'height' }"
          @click="fitMode = 'height'"
        >
          {{ $t('reader.fit_height') }}
        </button>
        <button
          class="btn small"
          :class="{ primary: fitMode === 'width' }"
          @click="fitMode = 'width'"
        >
          {{ $t('reader.fit_width') }}
        </button>
        <button
          class="btn small"
          :class="{ primary: fitMode === 'original' }"
          @click="fitMode = 'original'"
        >
          {{ $t('reader.fit_original') }}
        </button>
      </div>
      <button class="btn danger" @click="remove">{{ $t('common.delete') }}</button>
    </header>

    <div ref="scrollRef" class="scroll-strip" @scroll="onScroll">
      <div v-if="!total" class="loading">{{ $t('reader.no_pages') }}</div>
      <div
        v-for="(_p, i) in pages"
        :key="i"
        class="page-wrap"
      >
        <img
          :src="pageSrc(i)"
          :alt="$t('common.page_n', { n: i + 1 })"
          loading="lazy"
          decoding="async"
          class="page-thumb"
        />
        <img
          :src="pageSrc(i)"
          :alt="$t('common.page_n', { n: i + 1 })"
          loading="lazy"
          decoding="async"
          class="page-img"
          @error="onImageError(i)"
             @load="(e) => { (e.target as HTMLImageElement).classList.add('loaded'); onPageLoaded(i); }"
        />
        <div v-if="failedPages.has(i)" class="page-error">
          <AlertTriangle :size="20" />
          <button class="btn" @click="reloadPage(i)">{{ $t('reader.reload') }}</button>
        </div>
      </div>
    </div>

    <footer class="bar">
      <button class="btn" @click="prev"><ChevronLeft :size="16" /> {{ $t('reader.prev') }}</button>
      <input
        type="range"
        min="1"
        :max="Math.max(1, total)"
        v-model.number="currentPage"
        @change="scrollToPage(currentPage - 1, false)"
      />
      <button class="btn" @click="next">{{ $t('reader.next') }} <ChevronRight :size="16" /></button>
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
  /* On narrow windows the buttons + delete don't all fit on one line; wrap
     them down instead of overflowing horizontally. */
  flex-wrap: wrap;
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
.btn.icon-only {
  padding: 4px 6px;
}
.btn.danger {
  color: #ff8e8e;
}

.scroll-strip {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  scroll-behavior: auto;
  background: #000;
}

.page-wrap {
  position: relative;
  margin-bottom: 2px;
}

.page-thumb {
  display: block;
  margin: 0 auto;
  min-height: 1px;
}

.page-img {
  position: absolute;
  top: 0;
  left: 50%;
  transform: translateX(-50%);
  display: block;
  margin: 0 auto;
  opacity: 0;
  transition: opacity 0.3s ease;
  z-index: 1;
}
.page-img.loaded {
  opacity: 1;
}

.page-error {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  background: rgba(0, 0, 0, 0.7);
  color: #ff9e9e;
  font-size: 1.5rem;
}
.page-error .btn {
  font-size: 0.8rem;
  padding: 4px 12px;
}

.fit-height .page-wrap {
  height: 100%;
}
.fit-height .page-thumb,
.fit-height .page-img {
  height: 100%;
  width: auto;
  max-width: 100%;
  object-fit: contain;
}

.fit-width .page-thumb,
.fit-width .page-img {
  width: 100%;
  height: auto;
}

.fit-original .scroll-strip {
  overflow-x: auto;
}
.fit-original .page-thumb,
.fit-original .page-img {
  max-width: none;
  max-height: none;
}

.direction-horizontal .scroll-strip {
  display: flex;
  flex-direction: row;
  overflow-y: hidden;
  overflow-x: auto;
}
.direction-horizontal .page-wrap {
  flex-shrink: 0;
  width: 100%;
  height: 100%;
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
