<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, onUnmounted, reactive, ref, watch, defineAsyncComponent } from "vue";
import { RouterView, useRoute, useRouter } from "vue-router";
import { onBackButtonPress } from "@tauri-apps/api/app";
import type { PluginListener } from "@tauri-apps/api/core";

import AppSidebar from "@/components/AppSidebar.vue";
import { cloudflareOpenChallenge, onDownloadProgress } from "@/api";
import { useSettingsStore } from "@/stores/settings";
import { useDownloadsStore } from "@/stores/downloads";
import { useFavoritesStore } from "@/stores/favorites";
import { useTagsStore } from "@/stores/tags";
import { useOverlayStore } from "@/stores/overlay";
import { useReadProgressStore } from "@/stores/readProgress";
import { useDownloadedStore } from "@/stores/downloaded";
import { ensureNotificationPermission, handleDownloadNotification } from "@/composables/useNotifications";

const GalleryView = defineAsyncComponent(() => import("@/views/GalleryView.vue"));
const ReaderView = defineAsyncComponent(() => import("@/views/ReaderView.vue"));
const LocalDetailView = defineAsyncComponent(() => import("@/views/LocalDetailView.vue"));
const LocalReaderView = defineAsyncComponent(() => import("@/views/LocalReaderView.vue"));

const settings = useSettingsStore();
const downloads = useDownloadsStore();
const favorites = useFavoritesStore();
const tags = useTagsStore();
const overlay = useOverlayStore();
const readProgress = useReadProgressStore();
const downloaded = useDownloadedStore();
const router = useRouter();
const route = useRoute();

const sidebarOpen = ref(true);
const cloudflareBanner = ref(false);

// Responsive layout: below this breakpoint the sidebar becomes a slide-over
// drawer and the content takes the full width. We track it live so rotating
// the device / resizing the window switches modes cleanly.
const COMPACT_QUERY = "(max-width: 760px)";
const isCompact = ref(false);
function syncCompact() {
  isCompact.value = window.matchMedia(COMPACT_QUERY).matches;
  // On compact screens the sidebar starts closed (drawer hidden). On desktop
  // it starts expanded so frequent nav items are visible.
  sidebarOpen.value = !isCompact.value;
}
let compactMql: MediaQueryList | null = null;

const overlayPanelRef = ref<HTMLElement | null>(null);

const overlayPanelDrag = reactive({
  active: false,
  startX: 0,
  startY: 0,
  x: 0,
  dismissing: false,
});

function resetOverlayDrag() {
  overlayPanelDrag.active = false;
  overlayPanelDrag.x = 0;
  overlayPanelDrag.dismissing = false;
}

const overlayPanelStyle = computed(() => {
  if (!overlayPanelDrag.active && !overlayPanelDrag.dismissing) return {};
  const x = overlayPanelDrag.dismissing ? "100%" : `${overlayPanelDrag.x}px`;
  return {
    transform: `translateX(${x})`,
    transition: overlayPanelDrag.active ? "none" : "transform 0.28s ease",
  };
});

function onOverlayTouchStart(e: TouchEvent) {
  if (e.touches.length !== 1) return;
  overlayPanelDrag.startX = e.touches[0].clientX;
  overlayPanelDrag.startY = e.touches[0].clientY;
  overlayPanelDrag.active = false;
  overlayPanelDrag.x = 0;
  overlayPanelDrag.dismissing = false;
}

function onOverlayTouchMove(e: TouchEvent) {
  if (e.touches.length !== 1) return;
  const dx = e.touches[0].clientX - overlayPanelDrag.startX;
  const dy = e.touches[0].clientY - overlayPanelDrag.startY;

  if (!overlayPanelDrag.active) {
    if (dx > 8 && Math.abs(dx) > Math.abs(dy)) {
      overlayPanelDrag.active = true;
    } else {
      return;
    }
  }

  overlayPanelDrag.x = Math.max(0, dx);
  e.preventDefault();
}

function onOverlayTouchEnd() {
  if (!overlayPanelDrag.active) return;

  if (overlayPanelDrag.x > window.innerWidth * 0.3) {
    overlayPanelDrag.dismissing = true;
    overlayPanelDrag.active = false;
    nextTick(() => {
      const el = overlayPanelRef.value;
      if (el) {
        const onEnd = (e: TransitionEvent) => {
          if (e.propertyName === "transform") {
            el.removeEventListener("transitionend", onEnd);
            overlay.pop();
            resetOverlayDrag();
          }
        };
        el.addEventListener("transitionend", onEnd);
      }
    });
  } else {
    overlayPanelDrag.dismissing = false;
    overlayPanelDrag.active = false;
    overlayPanelDrag.x = 0;
  }
}

const contentEdgeSwipe = reactive({
  tracking: false,
  startX: 0,
});

function onContentTouchStart(e: TouchEvent) {
  if (e.touches.length !== 1 || !isCompact.value || sidebarOpen.value) {
    contentEdgeSwipe.tracking = false;
    return;
  }
  const x = e.touches[0].clientX;
  if (x < 30) {
    contentEdgeSwipe.startX = x;
    contentEdgeSwipe.tracking = true;
  } else {
    contentEdgeSwipe.tracking = false;
  }
}

function onContentTouchMove(e: TouchEvent) {
  if (!contentEdgeSwipe.tracking) return;
  const dx = e.touches[0].clientX - contentEdgeSwipe.startX;
  if (dx > 50) {
    sidebarOpen.value = true;
    contentEdgeSwipe.tracking = false;
    e.preventDefault();
  }
}

function onContentTouchEnd() {
  contentEdgeSwipe.tracking = false;
}

const canGoBack = computed(() => {
  if (overlay.hasAny()) return true;
  const detailRoutes = ["gallery", "reader", "reader-local"];
  return detailRoutes.includes(String(route.name));
});

watch(() => route.fullPath, () => {
  overlay.closeAll();
  // Auto-close the mobile drawer on navigation so the new view is fully
  // visible after picking a destination.
  if (isCompact.value) sidebarOpen.value = false;
});

onMounted(async () => {
  syncCompact();
  compactMql = window.matchMedia(COMPACT_QUERY);
  compactMql.addEventListener("change", syncCompact);
  try {
    await settings.load();
    await Promise.allSettled([
      downloads.init(),
      favorites.load(),
      tags.load(),
      readProgress.load(),
      downloaded.load(),
    ]);
    // Best-effort: ask for notification permission early so download progress
    // / completion can be surfaced. No-op where the plugin is unavailable.
    ensureNotificationPermission().catch(() => {});
    // Soft CF check on launch (best effort).
    const needed = await settings.checkCloudflare();
    cloudflareBanner.value = needed;
  } catch (e) {
    console.error("init failed", e);
  }

  // Watch download progress so we can (a) keep the "downloaded" cover badge in
  // sync the moment a gallery finishes on disk, and (b) post Android
  // notifications for progress + completion. Unsubscribed on teardown below.
  progressUnlisten = await onDownloadProgress(async (p) => {
    if (p.status === "finished") {
      // The backend indexes a finished gallery into the local library, so the
      // id is now on disk — optimistically flip the badge, then re-sync.
      downloaded.add(p.id);
      await downloaded.refresh();
    }
    // Notifications are guarded internally; safe to always invoke.
    handleDownloadNotification(p).catch(() => {});
  });
});

function toggleSidebar() {
  sidebarOpen.value = !sidebarOpen.value;
}

async function solveCloudflare() {
  await cloudflareOpenChallenge();
}


// Programmatic back triggered by the on-screen ← / Close buttons. On Android
// this runs inside the SPA only; the hardware back button is handled by
// `onBackButtonPress` below, which is the single authority that can prevent
// the app from being closed.
function goBack() {
  if (overlay.hasAny()) {
    overlay.pop();
    return;
  }
  router.back();
}

// --- Hardware back button (Android) -------------------------------------
// Tauri routes the Android hardware back button to this JS callback (added in
// @tauri-apps/api 2.9 / tauri 2.9). Registering it *prevents* the WebView from
// running its default goBack/exit, so the app can never be kicked to the home
// screen by the back button — it is fully hijacked here. Desktop is a no-op.
//
// Priority: overlay → SPA route back → swallow (stay in app). router.back() is
// a safe no-op when there is no route history to pop, so on the root screen the
// press is simply swallowed and the app stays open.
let backButtonUnlisten: PluginListener | null = null;

// Unsubscribe handle for the download:progress listener set up in onMounted.
// Used to refresh the "downloaded" cover badge on completion and to post
// download notifications. Typed loosely (UnlistenFn) to avoid importing the
// type just for teardown.
let progressUnlisten: (() => void) | null = null;

function handleBackButton() {
  // An open overlay always closes first (reader/gallery slide-over panel).
  if (overlay.hasAny()) {
    overlay.pop();
    return;
  }
  router.back();
}

// NOTE: the overlay (reader/gallery slide-over) is not a router route, so we do
// NOT push placeholder history entries for it. The hardware back button closes
// it via `onBackButtonPress` above, and the on-screen buttons / drag gestures
// close it directly via `overlay.pop()`. Keeping the history stack clean avoids
// leftover entries that would make `router.back()` feel stuck (needing extra
// presses to actually navigate).

onMounted(() => {
  // Hijack the Android hardware back button globally. Registering this callback
  // prevents the WebView from running its default goBack/exit, so the app can
  // never be kicked to the home screen by the back button — it is fully
  // hijacked here. The promise rejects on desktop (no such event), which we
  // ignore — desktop uses the on-screen ← button and Esc keys instead.
  onBackButtonPress(() => handleBackButton())
    .then((listener) => {
      backButtonUnlisten = listener;
    })
    .catch(() => {
      // Not Android (or plugin unavailable) — nothing to do.
    });
});

onBeforeUnmount(() => {
  backButtonUnlisten?.unregister();
  backButtonUnlisten = null;
});

onUnmounted(() => {
  compactMql?.removeEventListener("change", syncCompact);
  progressUnlisten?.();
  progressUnlisten = null;
});

const globalSpeedLabel = computed(() => {
  const bps = downloads.totalSpeed;
  if (bps <= 0) return "";
  if (bps >= 1024 * 1024) return `↓ ${(bps / (1024 * 1024)).toFixed(1)} MB/s`;
  if (bps >= 1024) return `↓ ${(bps / 1024).toFixed(0)} KB/s`;
  return `↓ ${bps.toFixed(0)} B/s`;
});

// Local search-box model. Kept deliberately separate from the route so typing
// does NOT trigger a search on every keystroke — the user must press Enter or
// click the submit button (doSearch). We only seed it from the URL when the
// route changes externally (e.g. opening a tag link from another page).
const searchQuery = ref(String(route.query.q ?? ""));

watch(
  () => route.query.q,
  (q) => {
    searchQuery.value = String(q ?? "");
  },
);

function doSearch() {
  const q = searchQuery.value.trim();
  if (route.name === "search") {
    // Empty query is allowed on the search page (clears the text filter) so the
    // user can drop a text search but keep tag / language filters.
    router.replace({ query: { ...route.query, q: q || undefined } });
  } else {
    if (!q) return;
    router.push({ name: "search", query: { q } });
  }
}
</script>

<template>
  <div class="app" :class="{ 'sidebar-collapsed': !sidebarOpen, compact: isCompact }">
    <!-- Desktop: inline collapsible sidebar (collapses to an icon rail).
         Mobile: slide-over drawer + tap-to-dismiss backdrop, rendered via v-if
         so it occupies no layout space at all when closed. -->
    <AppSidebar v-if="!isCompact" :open="sidebarOpen" @toggle="toggleSidebar" />
    <template v-else>
      <Transition name="fade">
        <div v-if="sidebarOpen" class="drawer-backdrop" @click="toggleSidebar" />
      </Transition>
      <AppSidebar
        mobile
        :open="sidebarOpen"
        @toggle="toggleSidebar"
        @navigate="sidebarOpen = false"
      />
    </template>

    <main
      class="content"
      @touchstart="onContentTouchStart"
      @touchmove="onContentTouchMove"
      @touchend="onContentTouchEnd"
    >
      <header class="topbar">
        <button v-if="canGoBack" class="icon-btn back-btn" @click="goBack" title="Back">
          ←
        </button>
        <button
          class="icon-btn"
          :title="isCompact ? 'Menu' : 'Toggle sidebar'"
          @click="toggleSidebar"
        >
          ☰
        </button>
        <div class="search">
          <span>🔍</span>
          <input
            v-model="searchQuery"
            type="text"
            class="search-input"
            placeholder="Search galleries, tags..."
            @keydown.enter="doSearch"
          />
          <button
            type="button"
            class="search-btn"
            :disabled="!searchQuery.trim() && route.name !== 'search'"
            title="Search"
            @click="doSearch"
          >Search</button>
        </div>
      </header>

      <div v-if="cloudflareBanner" class="banner cf">
        <span>Cloudflare verification required to reach {{ settings.mirror }}.</span>
        <button @click="solveCloudflare">Solve now</button>
      </div>

      <RouterView v-slot="{ Component }">
        <KeepAlive>
          <component :is="Component" />
        </KeepAlive>
      </RouterView>
    </main>

    <Transition name="slide-in">
      <div v-if="overlay.galleryId !== null" :key="'gallery-' + overlay.galleryId" class="overlay-wrapper">
        <div
          ref="overlayPanelRef"
          class="overlay-panel"
          :style="overlayPanelStyle"
          @touchstart="onOverlayTouchStart"
          @touchmove="onOverlayTouchMove"
          @touchend="onOverlayTouchEnd"
        >
          <GalleryView :id="overlay.galleryId" overlay @back="overlay.pop()" />
        </div>
      </div>
    </Transition>
    <Transition name="slide-in">
      <div v-if="overlay.localDetailFolder !== null" :key="'local-detail-' + overlay.localDetailFolder" class="overlay-wrapper">
        <div
          ref="overlayPanelRef"
          class="overlay-panel"
          :style="overlayPanelStyle"
          @touchstart="onOverlayTouchStart"
          @touchmove="onOverlayTouchMove"
          @touchend="onOverlayTouchEnd"
        >
          <LocalDetailView :folder="overlay.localDetailFolder" overlay @back="overlay.pop()" />
        </div>
      </div>
    </Transition>
    <Transition name="slide-in">
      <div v-if="overlay.readerId !== null" :key="'reader-' + overlay.readerId" class="overlay-wrapper overlay-wrapper--full">
        <div
          ref="overlayPanelRef"
          class="overlay-panel overlay-panel--full"
          :style="overlayPanelStyle"
        >
          <ReaderView :id="overlay.readerId" overlay @back="overlay.pop()" />
        </div>
      </div>
    </Transition>
    <Transition name="slide-in">
      <div v-if="overlay.localReaderFolder !== null" :key="'local-' + overlay.localReaderFolder" class="overlay-wrapper overlay-wrapper--full">
        <div
          ref="overlayPanelRef"
          class="overlay-panel overlay-panel--full"
          :style="overlayPanelStyle"
        >
          <LocalReaderView :folder="overlay.localReaderFolder" overlay @back="overlay.pop()" />
        </div>
      </div>
    </Transition>

    <div v-if="globalSpeedLabel" class="global-speed">{{ globalSpeedLabel }}</div>
  </div>
</template>

<style scoped>
.app {
  display: flex;
  height: 100vh;
  background: var(--bg);
  color: var(--text);
}
.content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  overflow: hidden;
}
.topbar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 14px;
  border-bottom: 1px solid var(--border);
  background: var(--surface);
  flex-shrink: 0;
}
.icon-btn {
  background: transparent;
  border: none;
  color: var(--text);
  font-size: 1.1rem;
  cursor: pointer;
  padding: 4px 8px;
}
.back-btn {
  font-size: 1.3rem;
  font-weight: 700;
  margin-right: -4px;
}
.search {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 6px 12px;
  color: var(--text-dim);
  max-width: 540px;
}
.search-input {
  flex: 1;
  background: transparent;
  border: none;
  color: var(--text);
  font-size: 0.85rem;
  outline: none;
  min-width: 0;
}
.search-input::placeholder {
  color: var(--text-dim);
  opacity: 0.7;
}
.search-btn {
  background: var(--accent);
  border: none;
  color: #fff;
  border-radius: 6px;
  padding: 4px 14px;
  font-size: 0.8rem;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  flex-shrink: 0;
}
.search-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
.banner {
  margin: 10px 14px 0;
  padding: 10px 14px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  font-size: 0.85rem;
}
.banner.cf {
  background: rgba(255, 170, 50, 0.12);
  border: 1px solid rgba(255, 170, 50, 0.5);
  color: #ffce80;
}
.banner button {
  background: var(--accent);
  border: none;
  color: #fff;
  padding: 6px 14px;
  border-radius: 6px;
  cursor: pointer;
}

.overlay-wrapper {
  position: fixed;
  inset: 0;
  z-index: 1000;
}
.overlay-wrapper--full {
  z-index: 1001;
}
.overlay-panel {
  width: 100%;
  height: 100%;
  background: var(--bg);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  touch-action: pan-y;
}
.overlay-panel--full {
  background: #000;
}

.slide-in-enter-active,
.slide-in-leave-active {
  transition: transform 0.28s ease;
}
.slide-in-enter-from {
  transform: translateX(100%);
}
.slide-in-leave-to {
  transform: translateX(100%);
}

.global-speed {
  position: fixed;
  bottom: 8px;
  left: 8px;
  z-index: 1500;
  background: rgba(0, 0, 0, 0.75);
  color: var(--accent);
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 0.78rem;
  font-variant-numeric: tabular-nums;
  pointer-events: none;
}

/* Dimmed backdrop behind the mobile drawer; tap to close. Sits above content
   but below the drawer itself (z-index 1200 in AppSidebar). */
.drawer-backdrop {
  position: fixed;
  inset: 0;
  z-index: 1100;
  background: rgba(0, 0, 0, 0.5);
}
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.22s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
