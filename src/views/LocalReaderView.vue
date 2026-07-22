<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, nextTick, watch } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";

import {
  imageProxyUrl,
  localList,
  localDelete,
  localReaderProgressGet,
  localReaderProgressSet,
} from "@/api";
import { X, ArrowLeftRight, ArrowUpDown, AlertTriangle, ChevronLeft, ChevronRight } from "lucide-vue-next";
import { useReadProgressStore } from "@/stores/readProgress";
import { useSettingsStore } from "@/stores/settings";
import { useDownloadsStore } from "@/stores/downloads";
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
const downloads = useDownloadsStore();
const { t } = useI18n();

const pages = computed(() => local.value?.page_files ?? []);
const total = computed(() => pages.value.length);

// The active download entry for this gallery, if it is still being
// downloaded. The downloads store only retains non-finished entries, so
// presence here means the comic is incomplete on disk.
const activeDownload = computed(() =>
  local.value ? downloads.items.find((d) => d.id === local.value!.id) ?? null : null,
);
const isDownloading = computed(() => !!activeDownload.value);

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
 * Persist the user's reading position.
 *
 * Two separate concerns are recorded:
 *  1. `local_reader_progress` — the *exact* page the user is on (the resume
 *     point). Overwrites every time, so scrolling backwards is remembered.
 *     This is what `load()` reads to reopen at the right page.
 *  2. The shared `readProgress` store — a furthest-reached high-water mark
 *     that drives the "read" cover badge (>= 50%). Only moves forward.
 *
 * Both are guarded by `pagesLoaded`: page elements collapse to ~1px until
 * their image decodes, so `computeCurrentPage()` returns garbage before then.
 * Recording that bogus value is what made reopen jump to the wrong page.
 */
function reportProgress() {
  const totalVal = total.value;
  if (!totalVal || !local.value) return;
  const gid = local.value.id;
  const page = currentPage.value;
  if (gid <= 0 || page <= 0) return;
  // Refuse to persist until the page we think we're on has actually loaded —
  // otherwise we'd write a settle-time phantom page.
  if (pagesLoaded.value < page) return;
  void localReaderProgressSet(gid, page, totalVal);
  // High-water mark for the cover badge; the store enforces forward-only.
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
      const resume = await localReaderProgressGet(local.value.id);
      if (resume && resume > 1 && resume <= total.value) {
        // Jump to the saved page. Scroll first (instantly), then pin the
        // counter once layout has settled; the guard in reportProgress()
        // keeps us from overwriting the saved value with a settle-time
        // phantom until the target page's image has actually loaded.
        scrollToPage(resume - 1, false);
        await nextTick();
        currentPage.value = resume;
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
  // Flush the exact page on exit. We recompute from the live scroll position
  // (the debounce-based currentPage can lag) and only persist when we can
  // trust the geometry — i.e. the page we landed on has actually decoded.
  // This avoids both stale values and settle-time phantom pages, which were
  // why reopening jumped to the wrong spot.
  const gid = local.value?.id;
  if (gid && gid > 0 && total.value > 0) {
    computeCurrentPage();
    const page = currentPage.value;
    if (page > 0 && pagesLoaded.value >= page) {
      void localReaderProgressSet(gid, page, total.value);
      void readProgress.report(gid, page, total.value);
    }
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

    <div v-if="isDownloading" class="downloading-tag" :title="$t('reader.downloading_hint')">
      <span class="downloading-dot"></span>
      <span>{{ $t('reader.downloading') }}</span>
      <span v-if="activeDownload && activeDownload.total_pages" class="downloading-count">
        {{ activeDownload.done_pages }}/{{ activeDownload.total_pages }}
      </span>
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
  position: relative;
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

.downloading-tag {
  position: absolute;
  left: 14px;
  bottom: 64px;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 10px;
  background: rgba(0, 0, 0, 0.75);
  border: 1px solid rgba(255, 255, 255, 0.18);
  border-radius: 999px;
  color: #ffd27a;
  font-size: 0.75rem;
  font-weight: 500;
  z-index: 3;
  pointer-events: none;
  backdrop-filter: blur(6px);
  -webkit-backdrop-filter: blur(6px);
  user-select: none;
}

.downloading-tag .downloading-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: #ffd27a;
  box-shadow: 0 0 6px rgba(255, 210, 122, 0.9);
  animation: downloading-pulse 1.4s ease-in-out infinite;
}

.downloading-tag .downloading-count {
  color: rgba(255, 255, 255, 0.7);
  font-weight: 400;
}

@keyframes downloading-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.25; }
}
</style>
