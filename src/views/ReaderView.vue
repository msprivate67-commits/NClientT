<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch, nextTick } from "vue";
import { useRoute, useRouter } from "vue-router";

import { AlertTriangle } from "@lucide/vue";
import ReaderPager from "@/components/ReaderPager.vue";
import ReaderToolbar from "@/components/ReaderToolbar.vue";
import { useGalleryStore } from "@/stores/gallery";
import { useSettingsStore } from "@/stores/settings";
import { useOverlayStore } from "@/stores/overlay";
import { useReadProgressStore } from "@/stores/readProgress";
import { usePriorityPreloadQueue } from "@/composables/usePriorityPreloadQueue";
import {
  cachedImageObjectUrl,
  loadImageObjectUrl,
} from "@/composables/useImageObjectCache";
import {
  type ReaderDirection,
  type ReaderFitMode,
  useReaderNavigation,
} from "@/composables/useReaderNavigation";

const props = defineProps<{ id: number | string; overlay?: boolean }>();
const emit = defineEmits<{ back: [] }>();
const route = useRoute();
const router = useRouter();
const gallery = useGalleryStore();
const settings = useSettingsStore();
const overlay = useOverlayStore();
const readProgress = useReadProgressStore();

const id = computed(() => Number(props.id));
const fitMode = ref<ReaderFitMode>(
  (settings.settings.reader_fit_mode as ReaderFitMode) || "height",
);
const scrollMode = ref<ReaderDirection>(
  (settings.settings.reader_direction as ReaderDirection) || "vertical",
);

const pages = computed(() => gallery.current?.pages ?? []);
const total = computed(() => pages.value.length);
const rtl = computed(() => settings.settings.use_rtl);

const failedPages = ref(new Set<number>());
const retries = ref(new Map<number, number>());
const {
  scrollRef,
  currentPage,
  onScroll,
  scrollToPage,
  previous: prev,
  next,
  reset: resetNavigation,
} = useReaderNavigation(total, scrollMode, {
  rtl,
  onPageSettled: () => reportProgress(),
});

function pageSrc(i: number): string {
  void retries.value.get(i);
  return cachedImageObjectUrl(pages.value[i]?.path);
}

function thumbSrc(i: number): string {
  const currentIndex = currentPage.value - 1;
  if (
    i < currentIndex - THUMBNAIL_PREVIOUS
    || i > currentIndex + THUMBNAIL_NEXT
  ) return "";
  const t = pages.value[i]?.thumbnail;
  return cachedImageObjectUrl(t);
}

const pagePreloader = usePriorityPreloadQueue(async (index) => {
  const page = pages.value[index];
  if (!page) return;
  const urls = [...new Set([page.thumbnail, page.path].filter((url): url is string => !!url))];
  await Promise.all(urls.map((url) => loadImageObjectUrl(url)));
}, 2);

function prioritizeReaderPages(index: number) {
  pagePreloader.enqueue([index - 1, index, index + 1, index + 2], true);
}

function continuePreloadingFrom(index: number) {
  prioritizeReaderPages(index);
}

function pageWrapStyle(i: number): Record<string, string> {
  const page = pages.value[i];
  if (!page?.width || !page.height) return {};
  if (fitMode.value === "width") {
    return { aspectRatio: `${page.width} / ${page.height}` };
  }
  if (fitMode.value === "original") {
    return { width: `${page.width}px`, height: `${page.height}px` };
  }
  return {};
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

const THUMBNAIL_PREVIOUS = 3;
const THUMBNAIL_NEXT = 4;

/**
 * Persist how far the user has read. We report the furthest page *viewed*
 * (high-water mark), not the page currently on screen, so scrolling back to
 * re-read earlier pages never marks the gallery as less-read. The backend
 * derives the "read" flag (>= 50%) from this.
 */
let reportedMax = 0;
function reportProgress() {
  const totalVal = total.value;
  if (!totalVal || !gallery.current) return;
  const page = currentPage.value;
  if (page > reportedMax) reportedMax = page;
  // Only write when the high-water mark actually advances, to avoid a DB
  // round-trip on every scroll tick.
  if (reportedMax > 0) {
    void readProgress.report(gallery.current.id, reportedMax, totalVal);
  }
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
  pagePreloader.reset();
  if (!gallery.current || gallery.current.id !== id.value) {
    await gallery.load(id.value);
  }
  const start = props.overlay
    ? overlay.readerPage
    : Number(route.query.page) || null;
  failedPages.value.clear();
  retries.value.clear();
  reportedMax = 0;
  await nextTick();
  if (start && start > 0 && start <= total.value) {
    currentPage.value = start;
    scrollToPage(start - 1, false);
  } else {
    resetNavigation();
  }
  continuePreloadingFrom(currentPage.value - 1);
  if (props.overlay) overlay.readerPage = null;
}

onMounted(() => {
  load();
  window.addEventListener("keydown", onKey);
});
onUnmounted(() => {
  window.removeEventListener("keydown", onKey);
  // Make sure the final position is recorded when leaving the reader.
  reportProgress();
});
watch(id, load);
watch(currentPage, (page) => prioritizeReaderPages(page - 1));
watch(fitMode, () => {
  nextTick(() => {
    if (scrollRef.value && currentPage.value > 1) {
      scrollToPage(currentPage.value - 1, false);
    }
  });
  settings.save({ reader_fit_mode: fitMode.value });
});
watch(scrollMode, () => {
  const page = currentPage.value;
  nextTick(() => scrollToPage(page - 1, false));
  settings.save({ reader_direction: scrollMode.value });
});
</script>

<template>
  <div class="reader" :class="[`fit-${fitMode}`, `direction-${scrollMode}`, { rtl }]">
    <ReaderToolbar
      v-model:fit-mode="fitMode"
      v-model:direction="scrollMode"
      :current-page="currentPage"
      :total="total"
      @close="props.overlay ? emit('back') : router.back()"
    />

    <div ref="scrollRef" class="scroll-strip" @scroll="onScroll">
      <div v-if="!total" class="loading">{{ $t('reader.loading') }}</div>
      <div
        v-for="(_p, i) in pages"
        :key="i"
        class="page-wrap"
        :style="pageWrapStyle(i)"
      >
        <img
          v-if="thumbSrc(i)"
          :src="thumbSrc(i)"
          :alt="$t('common.page_n', { n: i + 1 })"
          loading="lazy"
          decoding="async"
          class="page-thumb"
          :style="pages[i]?.width && pages[i]?.height ? { aspectRatio: `${pages[i].width} / ${pages[i].height}` } : {}"
        />
        <img
          v-if="pageSrc(i)"
          :src="pageSrc(i)"
          :alt="$t('common.page_n', { n: i + 1 })"
          :loading="Math.abs(i - (currentPage - 1)) <= 1 ? 'eager' : 'lazy'"
          decoding="async"
          class="page-img"
          :style="pages[i]?.width && pages[i]?.height ? { aspectRatio: `${pages[i].width} / ${pages[i].height}` } : {}"
          @error="onImageError(i)"
          @load="(e) => { (e.target as HTMLImageElement).classList.add('loaded'); }"
        />
        <div v-if="failedPages.has(i) && !thumbSrc(i)" class="page-error">
          <AlertTriangle :size="20" />
          <button class="btn" @click="reloadPage(i)">{{ $t('reader.reload') }}</button>
        </div>
      </div>
    </div>

    <ReaderPager
      :current-page="currentPage"
      :total="total"
      @previous="prev"
      @next="next"
      @select="(page) => { currentPage = page; scrollToPage(page - 1, false); }"
    />
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
  /* On narrow windows (e.g. a slim desktop pane or phone landscape) the
     counter + scroll-mode + fit + slider don't all fit on one line. Wrap them
     down instead of overflowing horizontally. */
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
  overscroll-behavior-x: contain;
  scroll-snap-type: x mandatory;
  touch-action: pan-x pinch-zoom;
}
.direction-horizontal .page-wrap {
  flex-shrink: 0;
  width: 100%;
  height: 100%;
  margin-bottom: 0;
  overflow: hidden;
  scroll-snap-align: start;
  scroll-snap-stop: always;
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
