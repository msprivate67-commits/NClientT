<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useRouter } from "vue-router";

import { imageProxyUrl, localList, localDelete } from "@/api";
import type { LocalGallery } from "@/types";

const props = defineProps<{ folder: number | string }>();
const router = useRouter();

const local = ref<LocalGallery | null>(null);
const index = ref(0);
const fitMode = ref<"width" | "height" | "original">("height");

// Cache-busting key forces the <img> to reload the current page from the proxy.
const imgReloadKey = ref(0);

const pages = computed(() => local.value?.page_files ?? []);
const total = computed(() => pages.value.length);
const src = computed(() => {
  const p = pages.value[index.value];
  if (!p) return "";
  const base = imageProxyUrl(p);
  return imgReloadKey.value ? `${base}${base.includes("?") ? "&" : "?"}_r=${imgReloadKey.value}` : base;
});

function refreshImage() {
  imgReloadKey.value++;
}

function prev() {
  if (index.value > 0) index.value--;
}
function next() {
  if (index.value < total.value - 1) index.value++;
}

function onKey(e: KeyboardEvent) {
  if (e.key === "ArrowRight") next();
  else if (e.key === "ArrowLeft") prev();
  else if (e.key === "Escape") router.back();
}

async function load() {
  const all = await localList();
  // `folder` route param is the gallery id for local galleries.
  local.value =
    all.find((l) => String(l.id) === String(props.folder)) ?? null;
  index.value = 0;
}

onMounted(() => {
  load();
  window.addEventListener("keydown", onKey);
});
onUnmounted(() => window.removeEventListener("keydown", onKey));

async function remove() {
  if (!local.value) return;
  if (!confirm(`Delete "${local.value.title}" from disk?`)) return;
  await localDelete(local.value.folder);
  router.push({ name: "local" });
}
</script>

<template>
  <div class="reader" :class="['fit-' + fitMode]">
    <header class="bar">
      <button class="btn" @click="router.back()">✕ Close</button>
      <span class="counter">{{ index + 1 }} / {{ total || "?" }}</span>
      <div class="fit">
        <button class="btn small" :class="{ primary: fitMode === 'height' }" @click="fitMode = 'height'">Fit H</button>
        <button class="btn small" :class="{ primary: fitMode === 'width' }" @click="fitMode = 'width'">Fit W</button>
        <button class="btn small" :class="{ primary: fitMode === 'original' }" @click="fitMode = 'original'">1:1</button>
        <button class="btn small" title="Reload current page" @click="refreshImage">🔄</button>
      </div>
      <button class="btn danger" @click="remove">Delete</button>
    </header>

    <div class="page-area" @click="next">
      <img v-if="src" :src="src" :alt="`page ${index + 1}`" />
      <div v-else class="loading">No pages found.</div>
    </div>

    <footer class="bar">
      <button class="btn" @click="prev">‹ Prev</button>
      <input type="range" min="0" :max="Math.max(0, total - 1)" v-model.number="index" />
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
.loading {
  color: #888;
}
input[type="range"] {
  flex: 1;
}
</style>
