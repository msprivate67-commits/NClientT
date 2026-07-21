<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useRouter } from "vue-router";

import TagChip from "@/components/TagChip.vue";
import GalleryGrid from "@/components/GalleryGrid.vue";
import {
  imageProxyUrl,
  openInBrowser,
} from "@/api";
import { useGalleryStore } from "@/stores/gallery";
import { useFavoritesStore } from "@/stores/favorites";
import { useDownloadsStore } from "@/stores/downloads";
import { useSettingsStore } from "@/stores/settings";
import { useOverlayStore } from "@/stores/overlay";
import { useScrollCache } from "@/composables/useScrollCache";

const props = defineProps<{ id: number | string; overlay?: boolean }>();
const emit = defineEmits<{ back: [] }>();
const router = useRouter();
const gallery = useGalleryStore();
const favorites = useFavoritesStore();
const downloads = useDownloadsStore();

const downloadState = computed(() => {
  if (!g.value) return null;
  const entry = downloads.items.find(
    (d) => d.id === g.value!.id && (d.status === "downloading" || d.status === "pending"),
  );
  return entry?.status ?? null;
});
const settings = useSettingsStore();
const overlay = useOverlayStore();

const id = computed(() => Number(props.id));
const error = ref<string | null>(null);
const commentsOpen = ref(false);
const tagsExpanded = ref(false);
const loading = ref(false);
const viewRef = ref<HTMLElement | null>(null);
useScrollCache(viewRef);

const g = computed(() => gallery.current);

const title = computed(() => {
  if (!g.value) return "";
  const t = g.value.titles;
  const pref = settings.settings.title_type;
  if (pref === "pretty" && t.pretty) return t.pretty;
  if (pref === "english" && t.english) return t.english;
  if (pref === "japanese" && t.japanese) return t.japanese;
  return t.pretty || t.english || t.japanese || "Unnamed";
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
    startPreload();
  } catch (e: any) {
    error.value = String(e?.message ?? e);
  } finally {
    loading.value = false;
  }
}

let preloadCancel: (() => void) | null = null;

function startPreload() {
  preloadCancel?.();
  if (!g.value?.pages?.length) return;
  const pagePaths = g.value.pages.map((p) => p.path).filter(Boolean) as string[];
  if (!pagePaths.length) return;
  let cancelled = false;
  preloadCancel = () => { cancelled = true; };
  let i = 0;
  const CONCURRENCY = 3;
  const next = () => {
    if (cancelled || i >= pagePaths.length) return;
    const batch = pagePaths.slice(i, i + CONCURRENCY);
    i += CONCURRENCY;
    for (const path of batch) {
      const img = new Image();
      img.src = imageProxyUrl(path);
      img.onload = img.onerror = () => {};
    }
    if (!cancelled && i < pagePaths.length) {
      setTimeout(next, 200);
    }
  };
  setTimeout(next, 500);
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

onMounted(load);
watch(id, load);
onUnmounted(() => preloadCancel?.());

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
      <button class="btn" @click="emit('back')">← Back</button>
      <span class="overlay-title">{{ title }}</span>
    </div>

    <!-- Sticky title bar: stays pinned to the top while the page scrolls,
         so the back button + title are always reachable on long pages. -->
    <div v-else class="sticky-title-bar">
      <button class="btn ghost back" @click="goBack" title="Back">←</button>
      <span class="sticky-title">{{ title }}</span>
    </div>

    <div v-if="error" class="error">{{ error }}</div>

    <div v-if="g" class="header">
      <div class="cover">
        <img v-if="coverSrc" :src="coverSrc" :alt="title" />
      </div>
      <div class="info">
        <div class="title-row">
          <h1 class="title">{{ title }}</h1>
          <button class="btn" :disabled="loading" @click="load" title="Reload gallery">
            {{ loading ? "Refreshing…" : "🔄 Refresh" }}
          </button>
        </div>
        <div class="meta">
          <span>#{{ g.id }}</span>
          <span>·</span>
          <span>{{ g.num_pages }} pages</span>
          <span>·</span>
          <span>❤ {{ g.num_favorites }}</span>
          <span v-if="g.upload_date">·</span>
          <span v-if="g.upload_date">{{ new Date(g.upload_date).toLocaleDateString() }}</span>
        </div>
        <div class="actions">
          <button class="btn primary" @click="read">📖 Read</button>
          <button
            class="btn"
            :disabled="downloadState !== null"
            :class="{ downloading: downloadState === 'downloading', queued: downloadState === 'pending' }"
            @click="download"
          >
            <template v-if="downloadState === 'downloading'">⏳ Downloading…</template>
            <template v-else-if="downloadState === 'pending'">⏳ Queued…</template>
            <template v-else>⬇ Download</template>
          </button>
          <button
            class="btn"
            :class="{ primary: g.is_favorited || favorites.ids.has(g.id) }"
            @click="toggleFavorite"
          >
            ★ Favorite
          </button>
          <button class="btn" @click="openInBrowser(String(g.id))">🌐 Open</button>
        </div>
      </div>
    </div>

    <div v-if="g" class="body">
      <div class="tag-toggle-bar">
        <button class="btn small" @click="tagsExpanded = !tagsExpanded">
          {{ tagsExpanded ? '▲ Collapse tags' : '▼ Expand tags' }}
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
        <div class="section-title">Pages</div>
        <div
          class="thumb-grid"
          :style="thumbColumns === 'auto' ? { gridTemplateColumns: 'repeat(auto-fill, minmax(140px, 1fr))' } : { gridTemplateColumns: `repeat(${thumbColumns}, 1fr)` }"
        >
          <div
            v-for="(page, i) in g.pages"
            :key="i"
            class="thumb-item"
            @click="readPage(i + 1)"
          >
            <img
              v-if="page.thumbnail || page.path"
              :src="imageProxyUrl(page.thumbnail || page.path || '')"
              :alt="`Page ${i + 1}`"
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
        <div class="section-title">Related</div>
        <GalleryGrid :galleries="g.related" />
      </section>

      <section class="comments">
        <button class="btn" @click="toggleComments">
          {{ commentsOpen ? "Hide comments" : "Show comments" }}
        </button>
        <div v-if="commentsOpen" class="comment-list">
          <div v-for="c in gallery.comments" :key="c.id" class="comment">
            <div class="who">
              <strong>{{ c.poster.username }}</strong>
              <span v-if="c.post_date">{{ new Date(c.post_date).toLocaleString() }}</span>
            </div>
            <div class="body">{{ c.body }}</div>
          </div>
          <div v-if="!gallery.comments.length" class="empty">No comments.</div>
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
  max-width: 1000px;
  margin: 0 auto;
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
  margin: -14px -14px 12px;
  background: var(--bg);
  border-bottom: 1px solid var(--border);
}
.sticky-title-bar .back {
  font-size: 1.2rem;
  font-weight: 700;
  padding: 2px 10px;
}
.sticky-title {
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
}
.title {
  margin: 0 0 8px;
  font-size: 1.3rem;
  line-height: 1.3;
}
.title-row {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  justify-content: space-between;
}
.title-row .title {
  flex: 1;
  min-width: 0;
}
.meta {
  color: var(--text-dim);
  font-size: 0.85rem;
  display: flex;
  gap: 6px;
  margin-bottom: 14px;
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
  margin: -14px -14px 14px;
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
  font-size: 0.95rem;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* ---------------------------------------------------------------------------
   Responsive: on small screens (phones / narrow windows) stack the cover
   above the info block instead of side-by-side, so nothing overflows. The
   meta + action rows are already flex-wrap, so they reflow naturally.
   --------------------------------------------------------------------------- */
@media (max-width: 640px) {
  .gallery-view {
    /* Let the detail page use the full width rather than being centred in a
       1000px column with huge side margins on a phone. */
    max-width: 100%;
  }
  .header {
    flex-direction: column;
    align-items: stretch;
    gap: 12px;
  }
  .cover {
    /* Cover becomes a banner-ish strip on top instead of a tall left rail. */
    width: 100%;
    max-width: 220px;
    align-self: center;
    aspect-ratio: 3 / 4;
  }
  .title-row {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
  }
  .title-row .title {
    font-size: 1.15rem;
  }
  .actions {
    /* Each action gets enough room to be tappable; wraps to new rows. */
    gap: 6px;
  }
  .actions .btn {
    flex: 1 1 auto;
    text-align: center;
  }
}
</style>
