<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useRouter } from "vue-router";

import { imageProxyUrl } from "@/api";
import { useGalleryStore } from "@/stores/gallery";
import { useSettingsStore } from "@/stores/settings";

const props = defineProps<{ id: number | string }>();
const router = useRouter();
const gallery = useGalleryStore();
const settings = useSettingsStore();

const id = computed(() => Number(props.id));
const index = ref(0);
const fitMode = ref<"width" | "height" | "original">(fitFromZoom(settings.settings.default_zoom_pct));

function fitFromZoom(z: number): "width" | "height" | "original" {
  if (z <= 50) return "height";
  if (z >= 150) return "original";
  return "width";
}

const pages = computed(() => gallery.current?.pages ?? []);
const total = computed(() => pages.value.length);
const current = computed(() => pages.value[index.value]);
const src = computed(() => {
  const p = current.value?.path;
  if (!p) return "";
  const base = imageProxyUrl(p);
  // Append a cache-buster so the <img> refetches when refreshImage() bumps the key.
  return imgReloadKey.value ? `${base}${base.includes("?") ? "&" : "?"}_r=${imgReloadKey.value}` : base;
});

const rtl = computed(() => settings.settings.use_rtl);

function go(delta: number) {
  const next = index.value + delta;
  if (next < 0 || next >= total.value) return;
  index.value = next;
}
function prev() {
  go(rtl.value ? 1 : -1);
}
function next() {
  go(rtl.value ? -1 : 1);
}

function onKey(e: KeyboardEvent) {
  if (e.key === "ArrowRight") next();
  else if (e.key === "ArrowLeft") prev();
  else if (e.key === "Escape") router.back();
}

async function load() {
  if (!gallery.current || gallery.current.id !== id.value) {
    await gallery.load(id.value);
  }
  index.value = 0;
}

// Cache-busting key forces the <img> to reload the current page from the proxy.
const imgReloadKey = ref(0);
function refreshImage() {
  imgReloadKey.value++;
}

onMounted(() => {
  load();
  window.addEventListener("keydown", onKey);
});
onUnmounted(() => window.removeEventListener("keydown", onKey));
watch(id, load);
</script>

<template>
  <div class="reader" :class="['fit-' + fitMode, { rtl }]">
    <header class="bar">
      <button class="btn" @click="router.back()">✕ Close</button>
      <span class="counter">{{ index + 1 }} / {{ total || "?" }}</span>
      <div class="fit">
        <button class="btn small" :class="{ primary: fitMode === 'height' }" @click="fitMode = 'height'">Fit H</button>
        <button class="btn small" :class="{ primary: fitMode === 'width' }" @click="fitMode = 'width'">Fit W</button>
        <button class="btn small" :class="{ primary: fitMode === 'original' }" @click="fitMode = 'original'">1:1</button>
        <button class="btn small" title="Reload current page" @click="refreshImage">🔄</button>
      </div>
    </header>

    <div class="page-area" @click="next">
      <img v-if="src" :src="src" :alt="`page ${index + 1}`" />
      <div v-else class="loading">Loading…</div>
    </div>

    <footer class="bar">
      <button class="btn" @click="prev">‹ Prev</button>
      <input
        type="range"
        min="0"
        :max="Math.max(0, total - 1)"
        v-model.number="index"
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
}
.counter {
  font-size: 0.85rem;
}
.fit {
  margin-left: auto;
  display: flex;
  gap: 4px;
}
.btn.small {
  padding: 2px 8px;
  font-size: 0.72rem;
}
.page-area {
  flex: 1;
  min-height: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: auto;
  cursor: pointer;
}
.page-area img {
  display: block;
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}
.fit-height .page-area img {
  height: 100%;
  width: auto;
  max-width: 100%;
  object-fit: contain;
}
.fit-width .page-area img {
  width: 100%;
  height: auto;
  max-height: 100%;
  object-fit: contain;
}
.fit-original .page-area img {
  max-width: none;
  max-height: none;
}
.loading {
  color: #888;
}
input[type="range"] {
  flex: 1;
}
</style>
