<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { localGet, localGetMeta, localSetTranslatedTitle, translateTitle, imageProxyUrl } from "@/api";
import { BookOpen, Loader, RefreshCw, Languages, ArrowLeft, ChevronDown, ChevronUp } from "@lucide/vue";
import TagChip from "@/components/TagChip.vue";
import GalleryGrid from "@/components/GalleryGrid.vue";
import { useOverlayStore } from "@/stores/overlay";
import { useSettingsStore } from "@/stores/settings";
import { stripLeadingId } from "@/utils/title";
import type { Gallery, LocalGallery, Tag } from "@/types";

const props = defineProps<{ folder: string; overlay?: boolean }>();
const emit = defineEmits<{ back: [] }>();
const router = useRouter();
const overlayStore = useOverlayStore();
const settingsStore = useSettingsStore();

const local = ref<LocalGallery | null>(null);
const loading = ref(true);
const translating = ref(false);
const translated = ref("");
const translateError = ref("");
// Offline metadata (tags + related) read from the folder's `.nomedia` file.
// May be `null` for imported folders without a cached gallery JSON; the tags and
// related sections simply stay hidden in that case.
const meta = ref<Gallery | null>(null);
const tagsExpanded = ref(false);

const title = computed(() => stripLeadingId(local.value?.title || `#${props.folder}`));
const translatedTitle = computed(() => local.value?.translated_title || "");
const coverSrc = computed(() => {
  const t = local.value?.thumbnail_path;
  return t ? imageProxyUrl(t) : "";
});

// Tags grouped by type, mirroring GalleryView's tagsByType.
const tagsByType = computed(() => {
  const map = new Map<string, Tag[]>();
  if (!meta.value) return map;
  for (const t of meta.value.tags) {
    const list = map.get(t.type) ?? [];
    list.push(t);
    map.set(t.type, list);
  }
  return map;
});

async function load() {
  loading.value = true;
  try {
    local.value = await localGet(Number(props.folder));
  } catch {
    local.value = null;
  } finally {
    loading.value = false;
  }
  // Fetch the cached gallery metadata in parallel with rendering the header —
  // a missing/corrupt .nomedia returns null and the extra sections stay hidden.
  try {
    meta.value = await localGetMeta(Number(props.folder));
  } catch {
    meta.value = null;
  }
}

async function doTranslate() {
  if (!local.value) return;
  translating.value = true;
  translateError.value = "";
  const s = settingsStore.settings;
  try {
    const result = await translateTitle(
      s.tl_base_url, s.tl_model, s.tl_api_key,
      title.value, s.tl_target_lang, s.tl_thinking,
    );
    translated.value = result;
    await localSetTranslatedTitle(local.value.id, result);
    if (local.value) {
      local.value = { ...local.value, translated_title: result };
    }
  } catch (e: any) {
    translateError.value = String(e?.message ?? e);
  } finally {
    translating.value = false;
  }
}

function read() {
  overlayStore.openLocalReader(String(local.value?.id ?? props.folder));
}

function onTagClick(t: Tag) {
  // Same behavior as GalleryView: close any overlay panel, then search.
  if (props.overlay) {
    overlayStore.closeAll();
  }
  const name = encodeURIComponent(t.name);
  const type = encodeURIComponent(t.type);
  router.push({ name: "search", query: { tags: `${t.id}:accepted:${name}:${type}` } });
}

function goToSettings() {
  // From an overlay panel, close it first so the settings route renders full
  // screen (same pattern as onTagClick).
  if (props.overlay) {
    overlayStore.closeAll();
  }
  router.push({ name: "settings" });
}

onMounted(load);
watch(() => props.folder, load);
</script>

<template>
  <div class="view" :class="{ 'overlay-mode': overlay }">
    <div v-if="overlay" class="overlay-bar">
      <button class="btn" @click="emit('back')"><ArrowLeft :size="16" /></button>
      <span class="overlay-title">{{ title }}</span>
    </div>

    <div v-if="loading" class="loading">{{ $t('localDetail.loading') }}</div>
    <template v-else-if="local">
      <div class="header">
        <div class="cover" v-if="coverSrc">
          <img :src="coverSrc" :alt="title" />
        </div>
        <div class="info">
          <h1 class="title">{{ title }}</h1>
          <div v-if="translatedTitle || translated" class="translated-title">
            {{ translated || translatedTitle }}
          </div>
          <div v-if="translateError" class="tl-error">{{ translateError }}</div>
          <div v-if="translateError" class="tl-error-hint">
            {{ $t('localDetail.translate_error_hint') }}
            <button class="link-btn" @click="goToSettings">{{ $t('localDetail.go_to_ai_settings') }}</button>
          </div>
          <div class="meta">
            <span>{{ local.num_pages }} {{ $t('localDetail.pages') }}</span>
          </div>
          <div class="primary-actions">
            <button class="btn primary read-btn" @click="read"><BookOpen :size="18" /> {{ $t('localDetail.read') }}</button>
            <button
              class="btn"
              :disabled="translating"
              @click="doTranslate"
            >
              <span v-if="translating"><Loader :size="14" class="spin" /> {{ $t('localDetail.translating') }}</span>
              <span v-else-if="translatedTitle || translated"><RefreshCw :size="14" /> {{ $t('localDetail.retranslate') }}</span>
              <span v-else><Languages :size="14" /> {{ $t('localDetail.translate') }}</span>
            </button>
          </div>
        </div>
      </div>

      <!-- Tags + related: sourced from the cached gallery JSON (.nomedia).
           Mirrors the online GalleryView layout. Only rendered when metadata
           is present. -->
      <div v-if="meta" class="body">
        <div v-if="tagsByType.size" class="tag-toggle-bar">
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

        <section v-if="meta.related.length" class="related">
          <div class="section-title">{{ $t('gallery.section_related') }}</div>
          <GalleryGrid :galleries="meta.related" />
        </section>
      </div>
    </template>
    <div v-else class="error">{{ $t('localDetail.load_error') }}</div>
  </div>
</template>

<style scoped>
.view {
  width: 100%;
  max-width: 1000px;
  /* min-width:0 keeps the page width driven by the window rather than by the
     title's intrinsic (max-content) width — see GalleryView for the same rule. */
  min-width: 0;
  margin: 0 auto;
  padding: 14px;
  overflow-y: auto;
  height: 100%;
}
.overlay-mode {
  height: 100%;
  overflow-y: auto;
  /* The overlay title bar should start at the panel's actual top edge. */
  padding-top: 0;
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
  /* Constrain to the info column so a long title can't push the page wider
     than the window — page width follows the window, not the title length. */
  width: 100%;
  max-width: 100%;
  font-size: 1.35rem;
  line-height: 1.35;
  /* Long titles must wrap rather than inflate the info column and overflow. */
  overflow-wrap: anywhere;
  word-break: break-word;
}
.translated-title {
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
.loading, .error {
  color: var(--text-dim);
  padding: 20px;
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
.section-title {
  font-size: 0.82rem;
  color: var(--text-dim);
  margin-bottom: 6px;
  text-transform: capitalize;
}
.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
/* Related galleries: thumbnail grid (same component as the online detail page).
   GalleryCard handles click → online detail. */
.related {
  margin-top: 22px;
}
@media (max-width: 768px) {
  .view {
    max-width: 100%;
  }
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
  .primary-actions {
    flex-direction: column;
  }
  .read-btn {
    font-size: 1.05rem;
    padding: 14px 20px;
  }
  .primary-actions > .btn:not(.read-btn) {
    text-align: center;
  }
}
</style>
