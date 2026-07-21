<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";

import TagChip from "@/components/TagChip.vue";
import GalleryGrid from "@/components/GalleryGrid.vue";
import {
  exportPdf,
  exportZip,
  imageProxyUrl,
  localList,
  openInBrowser,
} from "@/api";
import { useGalleryStore } from "@/stores/gallery";
import { useFavoritesStore } from "@/stores/favorites";
import { useDownloadsStore } from "@/stores/downloads";
import { useSettingsStore } from "@/stores/settings";
import { useScrollCache } from "@/composables/useScrollCache";

const props = defineProps<{ id: number | string }>();
const router = useRouter();
const gallery = useGalleryStore();
const favorites = useFavoritesStore();
const downloads = useDownloadsStore();
const settings = useSettingsStore();

const id = computed(() => Number(props.id));
const error = ref<string | null>(null);
const commentsOpen = ref(false);
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

async function load() {
  error.value = null;
  loading.value = true;
  try {
    await gallery.load(id.value);
  } catch (e: any) {
    error.value = String(e?.message ?? e);
  } finally {
    loading.value = false;
  }
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

async function exportAs(kind: "pdf" | "zip") {
  if (!g.value) return;
  const folder = await findLocalFolder(g.value.id);
  if (!folder) {
    error.value = "Gallery must be downloaded first.";
    return;
  }
  const out = kind === "pdf" ? await exportPdf(folder) : await exportZip(folder);
  error.value = `Exported: ${out}`;
}

async function findLocalFolder(_gid: number): Promise<string | null> {
  const all = await localList();
  const match = all.find((l) => l.id === _gid);
  return match?.folder ?? null;
}

function read() {
  router.push({ name: "reader", params: { id: id.value } });
}

onMounted(load);
watch(id, load);

async function toggleComments() {
  commentsOpen.value = !commentsOpen.value;
  if (commentsOpen.value && g.value && gallery.comments.length === 0) {
    await gallery.loadComments(g.value.id);
  }
}

async function onTagClick(t: any) {
  router.push({ name: "search", query: { tags: `${t.id}:accepted` } });
}
</script>

<template>
  <div ref="viewRef" class="view gallery-view">
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
          <button class="btn" @click="download">⬇ Download</button>
          <button
            class="btn"
            :class="{ primary: g.is_favorited || favorites.ids.has(g.id) }"
            @click="toggleFavorite"
          >
            ★ Favorite
          </button>
          <button class="btn" @click="openInBrowser(String(g.id))">🌐 Open</button>
          <button class="btn" @click="exportAs('zip')">📦 ZIP</button>
          <button class="btn" @click="exportAs('pdf')">📄 PDF</button>
        </div>
      </div>
    </div>

    <div v-if="g" class="body">
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
.body {
  margin-top: 8px;
}
.tag-group {
  margin-bottom: 14px;
}
.related {
  margin-top: 22px;
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
</style>
