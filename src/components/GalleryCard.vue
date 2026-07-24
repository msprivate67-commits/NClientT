<script setup lang="ts">
import { Check, Download, Loader, Star } from "@lucide/vue";
import { computed, ref } from "vue";

import { imageProxyUrl } from "@/api";
import { useLazyVisible } from "@/composables/useLazyVisible";
import { useFavoritesStore } from "@/stores/favorites";
import { useOverlayStore } from "@/stores/overlay";
import { useReadProgressStore } from "@/stores/readProgress";
import { useDownloadedStore } from "@/stores/downloaded";
import { useDownloadsStore } from "@/stores/downloads";
import type { SimpleGallery } from "@/types";

const props = defineProps<{
  gallery: SimpleGallery;
  /** When true, this is a local (on-disk) gallery; clicking opens the local reader. */
  local?: boolean;
  /** Optional override thumbnail (used for local galleries). */
  thumbnailOverride?: string | null;
  /** Optional override for the displayed title (e.g. a local translated title).
   *  When omitted, falls back to gallery.title. */
  displayTitle?: string;
  /** When true, the card shows a selection checkbox. */
  selectable?: boolean;
  /** Whether this card is currently selected. */
  selected?: boolean;
}>();

const emit = defineEmits<{
  (e: "select", id: number): void;
  (e: "deselect", id: number): void;
}>();

const favorites = useFavoritesStore();
const overlay = useOverlayStore();
const readProgress = useReadProgressStore();
const downloaded = useDownloadedStore();
const downloads = useDownloadsStore();
const cardRef = ref<HTMLElement | null>(null);
const coverVisible = useLazyVisible(cardRef);

const thumb = computed(
  () => props.thumbnailOverride ?? props.gallery.thumbnail ?? "",
);
// Title shown on the card: prefers the override (e.g. a translated title) and
// falls back to the gallery's own title when none is supplied.
const displayTitle = computed(
  () => props.displayTitle ?? props.gallery.title,
);
const src = computed(() => {
  if (!coverVisible.value || !thumb.value) return "";
  return imageProxyUrl(thumb.value);
});

// Language -> flag shown in the top-left corner of the cover. English maps
// to the US flag per the product spec (the dominant English-language audience
// for the source site is US-based).
const languageFlag = computed(() => {
  switch (props.gallery.language) {
    case "english":
      return "🇺🇸";
    case "japanese":
      return "🇯🇵";
    case "chinese":
      return "🇨🇳";
    default:
      return "";
  }
});

// Has the user read >= 50% of this gallery? Badged in the bottom-left corner.
const isRead = computed(() => readProgress.has(props.gallery.id));

// Has the user already downloaded this gallery to the local library? Also
// badged bottom-left, stacked *below* the read mark when both are present.
// Local cards are obviously already on disk, so the badge is redundant there.
const isDownloaded = computed(
  () => !props.local && downloaded.has(props.gallery.id),
);

// A partially-created local gallery may already be visible while its pages
// are still arriving. Only queued/running entries receive this badge.
const isDownloading = computed(
  () => props.local && downloads.activeForGallery(props.gallery.id) !== null,
);

function open() {
  if (props.selectable) {
    handleSelect();
    return;
  }
  if (props.local) {
    overlay.openLocalDetail(String(props.gallery.id));
  } else {
    overlay.openGallery(props.gallery.id);
  }
}

function handleSelect() {
  if (props.selected) {
    emit("deselect", props.gallery.id);
  } else {
    emit("select", props.gallery.id);
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
  <div ref="cardRef" class="card" :class="{ selected, selectable }" @click="open">
    <div class="thumb">
      <img v-if="src" :src="src" loading="lazy" :alt="gallery.title" />
      <div v-else-if="coverVisible && !thumb" class="placeholder">{{ $t('common.no_cover') }}</div>
      <div v-if="selectable" class="select-check" :class="{ checked: selected }" @click.stop="handleSelect">
        <span v-if="selected"><Check :size="14" /></span>
      </div>
      <span v-if="languageFlag && !selectable" class="flag">{{ languageFlag }}</span>
      <span v-if="gallery.num_pages" class="pages">{{ gallery.num_pages }}p</span>
      <!-- "Read" badge: shown once the user has viewed >= 50% of the pages.
           Lives bottom-left (the favorite star is bottom-right). When a
           lower badge is also present it stacks beneath, so we lift the
           read badge up to leave room. -->
      <span
        v-if="isRead && !selectable"
        class="read-mark"
        :class="{ stacked: isDownloaded || isDownloading }"
        :title="$t('galleryCard.read_tooltip')"
      ><Check :size="12" /> {{ $t('galleryCard.read_badge') }}</span>
      <!-- "Downloaded" badge: this gallery already exists on disk in the local
           library. Sits bottom-left, below the read mark when both show. -->
      <span
        v-if="isDownloaded && !selectable"
        class="downloaded-mark"
        :title="$t('galleryCard.downloaded_tooltip')"
      ><Download :size="12" /> {{ $t('galleryCard.downloaded_badge') }}</span>
      <!-- Active local download badge: bottom-left, below the read badge when
           both are present. It disappears as soon as the status finishes. -->
      <span
        v-if="isDownloading && !selectable"
        class="downloading-mark"
        :title="$t('reader.downloading_hint')"
      ><Loader :size="12" class="spin" /> {{ $t('reader.downloading') }}</span>
      <button
        class="fav"
        :class="{ active: favorites.ids.has(gallery.id) }"
        :title="$t('galleryCard.favorite_tooltip')"
        @click="toggleFav"
      >
        <Star :size="14" :fill="favorites.ids.has(gallery.id) ? 'currentColor' : 'none'" />
      </button>
    </div>
    <div class="meta">
      <div class="title" :title="displayTitle">{{ displayTitle || $t('common.unnamed') }}</div>
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
.read-mark {
  position: absolute;
  bottom: 6px;
  left: 6px;
  background: rgba(40, 170, 90, 0.92);
  color: #fff;
  font-size: 0.66rem;
  font-weight: 600;
  padding: 1px 7px;
  border-radius: 10px;
  letter-spacing: 0.02em;
  pointer-events: none;
  transition: bottom 0.15s ease;
}
/* When both read + downloaded badges show, lift the read badge above the
   downloaded one so the two stack instead of overlapping. */
.read-mark.stacked {
  bottom: 25px;
}
.downloaded-mark,
.downloading-mark {
  position: absolute;
  bottom: 6px;
  left: 6px;
  background: rgba(60, 130, 220, 0.92);
  color: #fff;
  font-size: 0.66rem;
  font-weight: 600;
  padding: 1px 7px;
  border-radius: 10px;
  letter-spacing: 0.02em;
  pointer-events: none;
}
.downloading-mark {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  background: rgba(220, 135, 35, 0.94);
}
.spin {
  animation: spin 1s linear infinite;
}
@keyframes spin {
  to { transform: rotate(360deg); }
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

.select-check {
  position: absolute;
  top: 6px;
  left: 6px;
  width: 22px;
  height: 22px;
  border-radius: 4px;
  border: 2px solid rgba(255, 255, 255, 0.6);
  background: rgba(0, 0, 0, 0.4);
  display: grid;
  place-items: center;
  color: #fff;
  font-size: 0.75rem;
  font-weight: 700;
  transition: background 0.15s, border-color 0.15s;
}
.select-check.checked {
  background: var(--accent);
  border-color: var(--accent);
}
.card.selectable {
  cursor: default;
}
.card.selectable:hover {
  transform: none;
  box-shadow: none;
}
.card.selected {
  outline: 2px solid var(--accent);
  border-radius: 8px;
}
</style>
