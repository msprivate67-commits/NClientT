<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch, defineAsyncComponent } from "vue";
import { RouterView, useRoute, useRouter } from "vue-router";

import AppSidebar from "@/components/AppSidebar.vue";
import { cloudflareOpenChallenge } from "@/api";
import { useSettingsStore } from "@/stores/settings";
import { useDownloadsStore } from "@/stores/downloads";
import { useFavoritesStore } from "@/stores/favorites";
import { useTagsStore } from "@/stores/tags";
import { useOverlayStore } from "@/stores/overlay";
import { useReadProgressStore } from "@/stores/readProgress";

const GalleryView = defineAsyncComponent(() => import("@/views/GalleryView.vue"));
const ReaderView = defineAsyncComponent(() => import("@/views/ReaderView.vue"));

const settings = useSettingsStore();
const downloads = useDownloadsStore();
const favorites = useFavoritesStore();
const tags = useTagsStore();
const overlay = useOverlayStore();
const readProgress = useReadProgressStore();
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
    ]);
    // Soft CF check on launch (best effort).
    const needed = await settings.checkCloudflare();
    cloudflareBanner.value = needed;
  } catch (e) {
    console.error("init failed", e);
  }
});

function toggleSidebar() {
  sidebarOpen.value = !sidebarOpen.value;
}

async function solveCloudflare() {
  await cloudflareOpenChallenge();
}


function goBack() {
  if (overlay.hasAny()) {
    overlay.pop();
    return;
  }
  // Initiate a programmatic back — let the popstate pass through unblocked.
  backBlocked = true;
  router.back();
}

// Track the route before the last popstate so we can decide whether to allow
// the system back button (only on detail pages with a visible back/close button).
const previousRouteName = ref<string | null>(null);
watch(() => route.name, (_, old) => {
  previousRouteName.value = old ? String(old) : null;
}, { flush: "sync" });

let backBlocked = false;

function onPopstate() {
  // Overlay back handling — the overlay system manages its own pushState/popState cycle.
  if (overlay.hasAny()) {
    overlay.pop();
    if (overlay.hasAny()) {
      history.pushState(null, "", window.location.href);
    }
    return;
  }

  // Programmatic back (from goBack) — let it through without blocking.
  if (backBlocked) {
    backBlocked = false;
    return;
  }

  // System back button — block by default.
  history.pushState(null, "", window.location.href);

  // Only allow the back to proceed when the user was on a detail page
  // that has a visible back/close button.
  const detailRoutes = ["gallery", "reader", "reader-local"];
  if (previousRouteName.value && detailRoutes.includes(previousRouteName.value)) {
    backBlocked = true;
    window.history.back();
  }
}

watch(
  () => overlay.hasAny(),
  (has, prev) => {
    if (has && !prev) {
      history.pushState(null, "", window.location.href);
    }
  },
);

onMounted(() => {
  window.addEventListener("popstate", onPopstate);
});

onUnmounted(() => {
  window.removeEventListener("popstate", onPopstate);
  compactMql?.removeEventListener("change", syncCompact);
});

const globalSpeedLabel = computed(() => {
  const bps = downloads.totalSpeed;
  if (bps <= 0) return "";
  if (bps >= 1024 * 1024) return `↓ ${(bps / (1024 * 1024)).toFixed(1)} MB/s`;
  if (bps >= 1024) return `↓ ${(bps / 1024).toFixed(0)} KB/s`;
  return `↓ ${bps.toFixed(0)} B/s`;
});

const searchQuery = computed({
  get: () => String(route.query.q ?? ""),
  set: (val: string) => {
    if (route.name === "search") {
      router.replace({ query: { ...route.query, q: val.trim() || undefined } });
    }
  },
});

function doSearch() {
  const q = searchQuery.value.trim();
  if (!q) return;
  if (route.name === "search") {
    router.replace({ query: { ...route.query, q } });
  } else {
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

    <main class="content">
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
      <div v-if="overlay.galleryId !== null" :key="'gallery-' + overlay.galleryId" class="overlay-panel">
        <GalleryView :id="overlay.galleryId" overlay @back="overlay.pop()" />
      </div>
    </Transition>
    <Transition name="slide-in">
      <div v-if="overlay.readerId !== null" :key="'reader-' + overlay.readerId" class="overlay-panel overlay-panel--full">
        <ReaderView :id="overlay.readerId" overlay @back="overlay.pop()" />
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

.overlay-panel {
  position: fixed;
  inset: 0;
  z-index: 1000;
  background: var(--bg);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.overlay-panel--full {
  z-index: 1001;
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
