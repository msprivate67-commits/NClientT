<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";

import TagChip from "@/components/TagChip.vue";
import GalleryGrid from "@/components/GalleryGrid.vue";
import {
  ArrowLeft,
  RefreshCw,
  Loader,
  Languages,
  Download,
  Star,
  BookOpen,
  Check,
  ChevronDown,
  ChevronUp,
  Heart,
} from "@lucide/vue";
import {
  imageProxyUrl,
  openInBrowser,
  translateTitle,
} from "@/api";
import { useGalleryStore } from "@/stores/gallery";
import { useFavoritesStore } from "@/stores/favorites";
import { useDownloadsStore } from "@/stores/downloads";
import { useSettingsStore } from "@/stores/settings";
import { useOverlayStore } from "@/stores/overlay";
import { useDownloadedStore } from "@/stores/downloaded";
import { useScrollCache } from "@/composables/useScrollCache";

const props = defineProps<{ id: number | string; overlay?: boolean }>();
const emit = defineEmits<{ back: [] }>();
const router = useRouter();
const gallery = useGalleryStore();
const favorites = useFavoritesStore();
const downloads = useDownloadsStore();
const downloaded = useDownloadedStore();

const downloadState = computed(() => {
  if (!g.value) return null;
  const entry = downloads.items.find(
    (d) => d.id === g.value!.id && (d.status === "downloading" || d.status === "pending"),
  );
  return entry?.status ?? null;
});
const settings = useSettingsStore();
const { t: i18n } = useI18n();
const overlay = useOverlayStore();

const id = computed(() => Number(props.id));
const error = ref<string | null>(null);
const commentsOpen = ref(false);
const tagsExpanded = ref(false);
const loading = ref(false);
const viewRef = ref<HTMLElement | null>(null);

// Is this gallery already on disk in the local library? When true the download
// button is disabled — re-downloading would just duplicate the folder.
const isDownloaded = computed(() => (g.value ? downloaded.has(g.value.id) : false));
useScrollCache(viewRef);

const translating = ref(false);
const translatedTitle = ref("");
const translateError = ref("");

async function doTranslate() {
  if (!g.value) return;
  translating.value = true;
  translateError.value = "";
  const s = settings.settings;
  try {
    translatedTitle.value = await translateTitle(
      s.tl_base_url, s.tl_model, s.tl_api_key,
      title.value, s.tl_target_lang, s.tl_thinking,
    );
  } catch (e: any) {
    translateError.value = String(e?.message ?? e);
  } finally {
    translating.value = false;
  }
}

const g = computed(() => gallery.current);

const title = computed(() => {
  if (!g.value) return "";
  const t = g.value.titles;
  const pref = settings.settings.title_type;
  if (pref === "pretty" && t.pretty) return t.pretty;
  if (pref === "english" && t.english) return t.english;
  if (pref === "japanese" && t.japanese) return t.japanese;
  return t.pretty || t.english || t.japanese || i18n("common.unnamed");
});

const coverSrc = computed(() => {
  const p = g.value?.cover?.path ?? g.value?.thumbnail?.path;
  return p ? imageProxyUrl(p) : "";
});

const tagsByType = computed(() => {
  const map = new Map<string, typeof g.value extends infer _ ? any : any>();
  if (!g.value) return map;
  for (const t of g.value.tags) {
    const list = map.get(t.type) ?? [];
    list.push(t);
    map.set(t.type, list);
  }
  return map;
});

const thumbColumns = computed(() => {
  const cols = settings.settings.page_thumbnail_columns;
  if (cols <= 0) return "auto";
  return cols;
});

async function load() {
  error.value = null;
  loading.value = true;
  try {
    await gallery.load(id.value);
    setupThumbObserver();
  } catch (e: any) {
    error.value = String(e?.message ?? e);
  } finally {
    loading.value = false;
  }
}

const loadedThumbs = ref(new Set<number>());
let thumbObserver: IntersectionObserver | null = null;

function setupThumbObserver() {
  thumbObserver?.disconnect();
  thumbObserver = null;
  loadedThumbs.value = new Set<number>();
  void nextTick(() => {
    const root = viewRef.value;
    if (!root) return;
    thumbObserver = new IntersectionObserver(
      (entries) => {
        const next = new Set(loadedThumbs.value);
        let changed = false;
        for (const entry of entries) {
          if (!entry.isIntersecting) continue;
          const index = Number((entry.target as HTMLElement).dataset.pageIndex);
          if (!Number.isInteger(index) || next.has(index)) continue;
          next.add(index);
          changed = true;
          thumbObserver?.unobserve(entry.target);
        }
        if (changed) loadedThumbs.value = next;
      },
      { root, rootMargin: "600px 0px", threshold: 0.01 },
    );
    root.querySelectorAll<HTMLElement>(".thumb-item").forEach((element) => {
      thumbObserver?.observe(element);
    });
  });
}

async function toggleFavorite() {
  if (!g.value) return;
  await favorites.toggle({
    id: g.value.id,
    title: title.value,
    media_id: g.value.media_id,
    thumbnail: g.value.thumbnail.thumbnail || g.value.thumbnail.path || "",
  });
}

async function download() {
  if (!g.value) return;
  if (downloadState.value !== null) return;
  if (isDownloaded.value) return; // already on disk — nothing to download
  await downloads.enqueue({ gallery_id: g.value.id });
}

function read() {
  if (props.overlay) {
    overlay.openReader(id.value);
  } else {
    router.push({ name: "reader", params: { id: id.value } });
  }
}

function readPage(pageNum: number) {
  if (props.overlay) {
    overlay.openReader(id.value, pageNum);
  } else {
    router.push({ name: "reader", params: { id: id.value }, query: { page: pageNum } });
  }
}

function goBack() {
  // Non-overlay galleries are reached via router navigation; the app's global
  // top bar also exposes a back button, but we keep one here for when the
  // detail view is the active route.
  if (window.history.length > 1) {
    router.back();
  } else {
    router.push({ name: "home" });
  }
}

function goToSettings() {
  // From an overlay panel, close it first so the settings route renders full
  // screen (same pattern as onTagClick).
  if (props.overlay) {
    overlay.closeAll();
  }
  router.push({ name: "settings" });
}

onMounted(load);
watch(id, load);
onUnmounted(() => thumbObserver?.disconnect());

async function toggleComments() {
  commentsOpen.value = !commentsOpen.value;
  if (commentsOpen.value && g.value && gallery.comments.length === 0) {
    await gallery.loadComments(g.value.id);
  }
}

async function onTagClick(t: any) {
  if (props.overlay) {
    overlay.closeAll();
  }
  const name = encodeURIComponent(t.name);
  const type = encodeURIComponent(t.type);
  router.push({ name: "search", query: { tags: `${t.id}:accepted:${name}:${type}` } });
}
</script>

<template>
  <div ref="viewRef" class="view gallery-view" :class="{ 'overlay-mode': overlay }">
    <div v-if="overlay" class="overlay-bar">
      <button class="btn" @click="emit('back')"><ArrowLeft :size="16" /></button>
      <span class="overlay-title">{{ title }}</span>
    </div>

    <!-- Sticky title bar: stays pinned to the top while the page scrolls,
         so the back button + title are always reachable on long pages. -->
    <div v-else class="sticky-title-bar">
      <button class="btn ghost back" @click="goBack" :title="$t('gallery.back')"><ArrowLeft :size="16" /></button>
      <span class="sticky-title">{{ title }}</span>
    </div>

    <div v-if="error" class="error">{{ error }}</div>

    <div v-if="g" class="header">
      <div class="cover">
        <img v-if="coverSrc" :src="coverSrc" :alt="title" />
      </div>
      <div class="info">
        <h1 class="title">{{ title }}</h1>
        <div v-if="translatedTitle" class="translated-title">{{ translatedTitle }}</div>
        <div v-if="translateError" class="tl-error">{{ translateError }}</div>
        <div v-if="translateError" class="tl-error-hint">
          {{ $t('gallery.translate_error_hint') }}
          <button class="link-btn" @click="goToSettings">{{ $t('gallery.go_to_ai_settings') }}</button>
        </div>
        <div class="meta">
          <span>#{{ g.id }}</span>
          <span>·</span>
          <span>{{ g.num_pages }} {{ $t('gallery.pages') }}</span>
          <span>·</span>
          <span><Heart :size="12" /> {{ g.num_favorites }}</span>
          <span v-if="g.upload_date">·</span>
          <span v-if="g.upload_date">{{ new Date(g.upload_date).toLocaleDateString() }}</span>
        </div>
        <div class="primary-actions">
          <button
            class="btn primary read-btn"
            @click="read"
            :title="$t('gallery.read')"
          >
            <BookOpen :size="18" /> {{ $t('gallery.read') }}
          </button>
          <div class="tool-btns">
            <button class="btn" :disabled="loading" @click="load" :title="$t('gallery.reload_gallery')">
              <RefreshCw v-if="!loading" :size="14" />{{ loading ? $t('common.refreshing') : ' ' + $t('common.refresh') }}
            </button>
            <button
              class="btn"
              :disabled="translating"
              @click="doTranslate"
              :title="translatedTitle ? $t('gallery.retranslate') : $t('gallery.translate_title')"
            >
              <span v-if="translating"><Loader :size="14" class="spin" /> {{ $t('gallery.translating') }}</span>
              <span v-else-if="translatedTitle"><RefreshCw :size="14" /> {{ $t('gallery.retranslate') }}</span>
              <span v-else><Languages :size="14" /> {{ $t('gallery.translate') }}</span>
            </button>
          </div>
        </div>
        <div class="actions">
          <button class="btn" @click="openInBrowser(String(g.id))">{{ $t('gallery.open') }}</button>
          <button
            class="btn"
            :disabled="downloadState !== null || isDownloaded"
            :class="{ downloading: downloadState === 'downloading', queued: downloadState === 'pending', done: isDownloaded && downloadState === null }"
            :title="isDownloaded && downloadState === null ? $t('gallery.already_downloaded') : undefined"
            @click="download"
          >
            <span v-if="downloadState === 'downloading'"><Loader :size="14" class="spin" /> {{ $t('gallery.downloading') }}</span>
            <span v-else-if="downloadState === 'pending'"><Loader :size="14" class="spin" /> {{ $t('gallery.queued') }}</span>
            <span v-else-if="isDownloaded"><Check :size="14" /> {{ $t('gallery.downloaded') }}</span>
            <span v-else><Download :size="14" /> {{ $t('gallery.download_btn') }}</span>
          </button>
          <button
            class="btn"
            :class="{ primary: g.is_favorited || favorites.ids.has(g.id) }"
            @click="toggleFavorite"
          >
            {{ '' }}<Star :size="14" :fill="g.is_favorited || favorites.ids.has(g.id) ? 'currentColor' : 'none'" /> {{ $t('gallery.favorite') }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="g" class="body">
      <div class="tag-toggle-bar">
        <button class="btn small" @click="tagsExpanded = !tagsExpanded">
          <ChevronUp v-if="tagsExpanded" :size="14" /> {{ tagsExpanded ? $t('gallery.collapse_tags') : '' }}<ChevronDown v-if="!tagsExpanded" :size="14" /> {{ !tagsExpanded ? $t('gallery.expand_tags') : '' }}
        </button>
      </div>
      <div v-show="tagsExpanded">
        <section v-for="[type, tags] in tagsByType" :key="type" class="tag-group">
          <div class="section-title">{{ type }}</div>
          <div class="chips">
            <TagChip
              v-for="t in tags"
              :key="t.id"
              :tag="t"
              show-type
              @click="onTagClick(t)"
            />
          </div>
        </section>
      </div>

      <section v-if="g.pages.length" class="page-thumbs">
        <div class="section-title">{{ $t('gallery.section_pages') }}</div>
        <div
          class="thumb-grid"
          :style="thumbColumns === 'auto' ? { gridTemplateColumns: 'repeat(auto-fill, minmax(140px, 1fr))' } : { gridTemplateColumns: `repeat(${thumbColumns}, 1fr)` }"
        >
          <div
            v-for="(page, i) in g.pages"
            :key="i"
            class="thumb-item"
            :data-page-index="i"
            @click="readPage(i + 1)"
          >
            <img
              v-if="loadedThumbs.has(i) && (page.thumbnail || page.path)"
              :src="imageProxyUrl(page.thumbnail || page.path || '')"
              :alt="$t('common.page_n', { n: i + 1 })"
              loading="lazy"
              decoding="async"
              class="thumb-img"
              @load="($event.target as HTMLImageElement).classList.add('loaded')"
            />
            <span class="thumb-label">{{ i + 1 }}</span>
          </div>
        </div>
      </section>

      <section v-if="g.related.length" class="related">
        <div class="section-title">{{ $t('gallery.section_related') }}</div>
        <GalleryGrid :galleries="g.related" />
      </section>

      <section class="comments">
        <button class="btn" @click="toggleComments">
          {{ commentsOpen ? $t('gallery.hide_comments') : $t('gallery.show_comments') }}
        </button>
        <div v-if="commentsOpen" class="comment-list">
          <div v-for="c in gallery.comments" :key="c.id" class="comment">
            <div class="who">
              <strong>{{ c.poster.username }}</strong>
              <span v-if="c.post_date">{{ new Date(c.post_date).toLocaleString() }}</span>
            </div>
            <div class="body">{{ c.body }}</div>
          </div>
          <div v-if="!gallery.comments.length" class="empty">{{ $t('gallery.no_comments') }}</div>
        </div>
      </section>
    </div>

    <div v-else-if="loading" class="loading-skeleton">
      <div class="header">
        <div class="cover skeleton-pulse"></div>
        <div class="info">
          <div class="skeleton-line w-70"></div>
          <div class="skeleton-line w-30"></div>
          <div class="skeleton-line w-100"></div>
        </div>
      </div>
      <div class="skeleton-section">
        <div class="skeleton-line w-15"></div>
        <div class="chips">
          <span class="skeleton-chip"></span>
          <span class="skeleton-chip w-60"></span>
          <span class="skeleton-chip w-50"></span>
          <span class="skeleton-chip w-40"></span>
          <span class="skeleton-chip w-30"></span>
        </div>
      </div>
      <div class="skeleton-section">
        <div class="skeleton-line w-20"></div>
        <div class="chips">
          <span class="skeleton-chip"></span>
          <span class="skeleton-chip w-45"></span>
          <span class="skeleton-chip w-55"></span>
        </div>
      </div>
      <div class="skeleton-section">
        <div class="skeleton-line w-25"></div>
        <div class="chips">
          <span class="skeleton-chip w-70"></span>
          <span class="skeleton-chip w-35"></span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.gallery-view {
  /* The centered, width-capped content is itself the scroll container, so the
     scrollbar hugs the content's right edge instead of sitting at the window
     edge on wide screens. height:100% + flex column keeps sticky positioning
     (sticky-title-bar / overlay-bar) anchored to this element while it
     scrolls. */
  width: 100%;
  max-width: 1000px;
  /* min-width:0 is essential: without it a long unbreakable title (or any
     intrinsic-content child) would make this flex/block item grow to its
     content's min-content width, so the whole page would visibly track the
     title length. With it the page width follows the window, not the title. */
  min-width: 0;
  height: 100%;
  margin: 0 auto;
  /* Keep content inset on the sides/bottom, but let the title bar occupy the
     real top edge. This avoids relying on a negative margin to cancel top
     padding, which can leave a gap with sticky positioning in some WebViews. */
  padding: 0 14px 14px;
  overflow-y: auto;
}
.error {
  padding: 12px 14px;
  background: rgba(255, 80, 80, 0.1);
  border: 1px solid rgba(255, 80, 80, 0.4);
  border-radius: 8px;
  color: #ff9e9e;
  margin-bottom: 14px;
  font-size: 0.85rem;
}
/* Pinned back + title row. Stays visible while the detail page scrolls so
   the user can always navigate away, even deep in a long comments/thread. */
.sticky-title-bar {
  position: sticky;
  top: 0;
  z-index: 5;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 4px;
  margin: 0 -14px 12px;
  background: var(--bg);
  border-bottom: 1px solid var(--border);
}
.sticky-title-bar .back {
  font-size: 1.2rem;
  font-weight: 700;
  padding: 2px 10px;
  flex-shrink: 0;
}
.sticky-title {
  /* flex:1 + min-width:0 lets the title ellipsize instead of pushing the back
     button off-screen when the title is very long. */
  flex: 1;
  min-width: 0;
  font-size: 0.95rem;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.btn.ghost {
  background: transparent;
  border-color: transparent;
}
.header {
  display: flex;
  gap: 18px;
  margin-bottom: 18px;
}
.cover {
  width: 220px;
  flex-shrink: 0;
  aspect-ratio: 3 / 4;
  background: var(--surface);
  border-radius: 8px;
  overflow: hidden;
}
.cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.title {
  margin: 0;
  /* Explicitly constrain to the info column so the heading's own intrinsic
     (max-content) width can never push the column/page wider than the window.
     This is what keeps the detail page width driven by the window, not by the
     title length. */
  width: 100%;
  max-width: 100%;
  font-size: 1.35rem;
  line-height: 1.35;
  /* Long titles (no spaces, e.g. long romanized/japanese strings) must wrap
     rather than inflate the info column and overflow the window. */
  overflow-wrap: anywhere;
  word-break: break-word;
}
.primary-actions {
  display: flex;
  align-items: stretch;
  gap: 10px;
}
.read-btn {
  flex: 1;
  font-size: 1rem;
  font-weight: 700;
  padding: 12px 24px;
  white-space: nowrap;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
}
.tool-btns {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}
.tool-btns .btn {
  padding: 10px 14px;
  font-size: 0.82rem;
  white-space: nowrap;
}
.translated-title {
  /* Constrain + wrap so a long translated line can't inflate the column. */
  width: 100%;
  max-width: 100%;
  color: var(--accent);
  font-size: 1.05rem;
  font-weight: 500;
  font-style: italic;
  overflow-wrap: anywhere;
  word-break: break-word;
}
.tl-error {
  color: #f08080;
  font-size: 0.82rem;
  padding: 6px 10px;
  background: rgba(220, 60, 60, 0.1);
  border-radius: 6px;
  overflow-wrap: anywhere;
}
.tl-error-hint {
  font-size: 0.78rem;
  color: var(--text-dim);
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}
.link-btn {
  background: none;
  border: none;
  color: var(--accent);
  cursor: pointer;
  text-decoration: underline;
  padding: 0;
  font-size: inherit;
}
.meta {
  color: var(--text-dim);
  font-size: 0.85rem;
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}
.actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.btn.downloading {
  opacity: 0.7;
  cursor: default;
  color: var(--accent);
}
.btn.queued {
  opacity: 0.7;
  cursor: default;
  color: #ffce80;
}
.btn.done {
  opacity: 0.7;
  cursor: default;
  color: #6ec16e;
}
.body {
  margin-top: 8px;
}
.tag-toggle-bar {
  margin-bottom: 10px;
}
.tag-group {
  margin-bottom: 14px;
}
.related {
  margin-top: 22px;
}
.page-thumbs {
  margin-top: 22px;
}
.thumb-grid {
  display: grid;
  gap: 8px;
}
.thumb-item {
  aspect-ratio: 3 / 4;
  border-radius: 6px;
  overflow: hidden;
  background: var(--surface);
  cursor: pointer;
  position: relative;
  transition: transform 0.15s, box-shadow 0.15s;
}
.thumb-item:hover {
  transform: scale(1.05);
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.4);
  z-index: 1;
}
.thumb-item:active {
  transform: scale(0.97);
}
.thumb-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  opacity: 0;
  transition: opacity 0.3s ease;
}
.thumb-img.loaded {
  opacity: 1;
}
.thumb-label {
  position: absolute;
  bottom: 4px;
  right: 4px;
  background: rgba(0, 0, 0, 0.7);
  color: #fff;
  font-size: 0.7rem;
  font-weight: 600;
  padding: 1px 6px;
  border-radius: 4px;
  pointer-events: none;
}
.comments {
  margin-top: 22px;
}
.comment {
  padding: 10px 0;
  border-top: 1px solid var(--border);
}
.comment .who {
  display: flex;
  gap: 8px;
  align-items: baseline;
  font-size: 0.78rem;
  color: var(--text-dim);
}
.comment .body {
  margin-top: 4px;
  font-size: 0.88rem;
  white-space: pre-wrap;
}
.empty {
  color: var(--text-dim);
  padding: 14px 0;
}

.loading-skeleton {
  animation: fadein 0.2s ease;
}
.loading-skeleton .header {
  display: flex;
  gap: 18px;
  margin-bottom: 18px;
}
.loading-skeleton .cover {
  width: 220px;
  flex-shrink: 0;
  aspect-ratio: 3 / 4;
}
.loading-skeleton .info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding-top: 6px;
}
.skeleton-pulse {
  background: var(--surface-2);
  border-radius: 8px;
  animation: pulse 1.5s ease-in-out infinite;
}
.skeleton-line {
  height: 14px;
  background: var(--surface-2);
  border-radius: 6px;
  animation: pulse 1.5s ease-in-out infinite;
}
.skeleton-line.w-10 { width: 10%; }
.skeleton-line.w-15 { width: 15%; }
.skeleton-line.w-20 { width: 20%; }
.skeleton-line.w-25 { width: 25%; }
.skeleton-line.w-30 { width: 30%; }
.skeleton-line.w-40 { width: 40%; }
.skeleton-line.w-45 { width: 45%; }
.skeleton-line.w-50 { width: 50%; }
.skeleton-line.w-55 { width: 55%; }
.skeleton-line.w-60 { width: 60%; }
.skeleton-line.w-70 { width: 70%; }
.skeleton-line.w-100 { width: 100%; }
.skeleton-chip {
  display: inline-block;
  height: 26px;
  width: 80px;
  background: var(--surface-2);
  border-radius: 6px;
  animation: pulse 1.5s ease-in-out infinite;
}
.skeleton-chip.w-30 { width: 60px; }
.skeleton-chip.w-35 { width: 70px; }
.skeleton-chip.w-40 { width: 80px; }
.skeleton-chip.w-45 { width: 90px; }
.skeleton-chip.w-50 { width: 100px; }
.skeleton-chip.w-55 { width: 110px; }
.skeleton-chip.w-60 { width: 130px; }
.skeleton-chip.w-70 { width: 150px; }
.skeleton-section {
  margin-bottom: 18px;
  padding-bottom: 14px;
  border-bottom: 1px solid var(--border);
}
.skeleton-section .chips {
  margin-top: 10px;
}
@keyframes pulse {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 0.8; }
}
@keyframes fadein {
  from { opacity: 0; }
  to { opacity: 1; }
}

.overlay-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 14px;
  background: var(--surface);
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
  margin: 0 -14px 14px;
  /* Keep the back + title visible while the overlay detail scrolls. */
  position: sticky;
  top: 0;
  z-index: 5;
}
.overlay-bar .btn {
  background: transparent;
  border: none;
  color: var(--accent);
  padding: 4px 10px;
  cursor: pointer;
  font-size: 0.95rem;
  font-weight: 600;
}
.overlay-bar .btn:hover {
  background: var(--accent-soft);
  border-radius: 6px;
}
.overlay-title {
  /* min-width:0 + flex:1 so the title ellipsizes within the overlay bar rather
     than stretching the bar beyond the panel width. */
  flex: 1;
  min-width: 0;
  font-size: 0.95rem;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* ---------------------------------------------------------------------------
   Responsive
   --------------------------------------------------------------------------- */
@media (max-width: 768px) {
  .header {
    flex-direction: column;
    align-items: center;
    gap: 16px;
  }
  .cover {
    width: 200px;
    max-width: 100%;
    aspect-ratio: 3 / 4;
  }
  .info {
    width: 100%;
    align-items: stretch;
    text-align: center;
  }
  .title {
    font-size: 1.15rem;
  }
  .meta {
    justify-content: center;
  }
  .primary-actions {
    flex-direction: column;
  }
  .read-btn {
    font-size: 1.05rem;
    padding: 14px 20px;
  }
  .tool-btns {
    justify-content: center;
  }
  .actions {
    gap: 6px;
  }
  .actions .btn {
    flex: 1 1 auto;
    text-align: center;
  }
}
</style>
