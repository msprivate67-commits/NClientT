<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { localGet, localSetTranslatedTitle, translateTitle, imageProxyUrl } from "@/api";
import { BookOpen, Loader, RefreshCw, Languages, ArrowLeft } from "lucide-vue-next";
import { useOverlayStore } from "@/stores/overlay";
import { useSettingsStore } from "@/stores/settings";
import type { LocalGallery } from "@/types";

const props = defineProps<{ folder: string; overlay?: boolean }>();
const emit = defineEmits<{ back: [] }>();
const overlayStore = useOverlayStore();
const settingsStore = useSettingsStore();

const local = ref<LocalGallery | null>(null);
const loading = ref(true);
const translating = ref(false);
const translated = ref("");
const translateError = ref("");

const title = computed(() => local.value?.title || `#${props.folder}`);
const translatedTitle = computed(() => local.value?.translated_title || "");
const coverSrc = computed(() => {
  const t = local.value?.thumbnail_path;
  return t ? imageProxyUrl(t) : "";
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
          <div class="meta">
            <span>{{ local.num_pages }} {{ $t('localDetail.pages') }}</span>
          </div>
          <div class="actions">
            <button class="btn primary read-btn" @click="read"><BookOpen :size="16" /> {{ $t('localDetail.read') }}</button>
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
    </template>
    <div v-else class="error">{{ $t('localDetail.load_error') }}</div>
  </div>
</template>

<style scoped>
.view {
  max-width: 1000px;
  margin: 0 auto;
  padding: 14px;
  overflow-y: auto;
  height: 100%;
}
.overlay-mode {
  height: 100%;
  overflow-y: auto;
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
.translated-title {
  color: var(--accent);
  font-size: 1.1rem;
  font-weight: 500;
  margin-bottom: 8px;
  font-style: italic;
}
.tl-error {
  color: #f08080;
  font-size: 0.82rem;
  margin-bottom: 8px;
  padding: 6px 10px;
  background: rgba(220, 60, 60, 0.1);
  border-radius: 6px;
}
.meta {
  color: var(--text-dim);
  font-size: 0.85rem;
  margin-bottom: 14px;
}
.actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
.read-btn {
  font-size: 1rem;
  font-weight: 700;
  padding: 10px 28px;
}
.loading, .error {
  color: var(--text-dim);
  padding: 20px;
}
@media (max-width: 640px) {
  .view {
    max-width: 100%;
  }
  .header {
    flex-direction: column;
    align-items: stretch;
    gap: 12px;
  }
  .cover {
    width: 100%;
    max-width: 220px;
    align-self: center;
    aspect-ratio: 3 / 4;
  }
  .title {
    font-size: 1.15rem;
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
