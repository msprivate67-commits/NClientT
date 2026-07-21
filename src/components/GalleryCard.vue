<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";

import { imageProxyUrl } from "@/api";
import { useFavoritesStore } from "@/stores/favorites";
import { useOverlayStore } from "@/stores/overlay";
import type { SimpleGallery } from "@/types";

const props = defineProps<{
  gallery: SimpleGallery;
  /** When true, this is a local (on-disk) gallery; clicking opens the local reader. */
  local?: boolean;
  /** Optional override thumbnail (used for local galleries). */
  thumbnailOverride?: string | null;
}>();

const router = useRouter();
const favorites = useFavoritesStore();
const overlay = useOverlayStore();

const thumb = computed(
  () => props.thumbnailOverride ?? props.gallery.thumbnail ?? "",
);
const src = computed(() => {
  if (!thumb.value) return "";
  if (thumb.value.startsWith("http")) return thumb.value;
  return imageProxyUrl(thumb.value);
});

const languageFlag = computed(() => {
  switch (props.gallery.language) {
    case "english":
      return "🇬🇧";
    case "japanese":
      return "🇯🇵";
    case "chinese":
      return "🇨🇳";
    default:
      return "";
  }
});

function open() {
  if (props.local) {
    router.push({ name: "reader-local", params: { folder: encodeURIComponent(props.gallery.id.toString()) } });
  } else {
    overlay.openGallery(props.gallery.id);
  }
}

async function toggleFav(e: MouseEvent) {
  e.stopPropagation();
  await favorites.toggle({
    id: props.gallery.id,
    title: props.gallery.title,
    media_id: props.gallery.media_id,
    thumbnail: props.gallery.thumbnail ?? "",
  });
}
</script>

<template>
  <div class="card" @click="open">
    <div class="thumb">
      <img v-if="src" :src="src" loading="lazy" :alt="gallery.title" />
      <div v-else class="placeholder">No cover</div>
      <span v-if="languageFlag" class="flag">{{ languageFlag }}</span>
      <span v-if="gallery.num_pages" class="pages">{{ gallery.num_pages }}p</span>
      <button
        class="fav"
        :class="{ active: favorites.ids.has(gallery.id) }"
        title="Favorite"
        @click="toggleFav"
      >
        ★
      </button>
    </div>
    <div class="meta">
      <div class="title" :title="gallery.title">{{ gallery.title || "Unnamed" }}</div>
    </div>
  </div>
</template>

<style scoped>
.card {
  background: var(--surface);
  border-radius: 8px;
  overflow: hidden;
  cursor: pointer;
  transition: transform 0.12s ease, box-shadow 0.12s ease;
  display: flex;
  flex-direction: column;
}
.card:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.35);
}
.thumb {
  position: relative;
  aspect-ratio: 3 / 4;
  background: var(--surface-2);
  overflow: hidden;
}
.thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.placeholder {
  position: absolute;
  inset: 0;
  display: grid;
  place-items: center;
  color: var(--text-dim);
  font-size: 0.8rem;
}
.flag,
.pages {
  position: absolute;
  top: 6px;
  font-size: 0.72rem;
  background: rgba(0, 0, 0, 0.6);
  color: #fff;
  padding: 1px 6px;
  border-radius: 10px;
}
.flag {
  left: 6px;
}
.pages {
  right: 6px;
}
.fav {
  position: absolute;
  bottom: 6px;
  right: 6px;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 50%;
  background: rgba(0, 0, 0, 0.55);
  color: #fff;
  cursor: pointer;
  font-size: 0.95rem;
  display: grid;
  place-items: center;
}
.fav.active {
  background: var(--accent);
  color: #fff;
}
.meta {
  padding: 6px 8px 8px;
}
.title {
  font-size: 0.82rem;
  line-height: 1.25;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  color: var(--text);
}
</style>
