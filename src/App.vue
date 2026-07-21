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

const GalleryView = defineAsyncComponent(() => import("@/views/GalleryView.vue"));
const ReaderView = defineAsyncComponent(() => import("@/views/ReaderView.vue"));

const settings = useSettingsStore();
const downloads = useDownloadsStore();
const favorites = useFavoritesStore();
const tags = useTagsStore();
const overlay = useOverlayStore();
const router = useRouter();
const route = useRoute();

const sidebarOpen = ref(true);
const cloudflareBanner = ref(false);

const canGoBack = computed(() => {
  if (overlay.hasAny()) return true;
  const detailRoutes = ["gallery", "reader", "reader-local"];
  return detailRoutes.includes(String(route.name));
});

watch(() => route.fullPath, () => {
  overlay.closeAll();
});

onMounted(async () => {
  try {
    await settings.load();
    await Promise.allSettled([
      downloads.init(),
      favorites.load(),
      tags.load(),
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

function go(route: string) {
  router.push({ name: route });
}

function goBack() {
  if (overlay.hasAny()) {
    overlay.pop();
  } else {
    router.back();
  }
}

function onPopstate() {
  if (overlay.hasAny()) {
    overlay.pop();
    history.pushState(null, "", window.location.href);
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
});

const globalSpeedLabel = computed(() => {
  const bps = downloads.totalSpeed;
  if (bps <= 0) return "";
  if (bps >= 1024 * 1024) return `↓ ${(bps / (1024 * 1024)).toFixed(1)} MB/s`;
  if (bps >= 1024) return `↓ ${(bps / 1024).toFixed(0)} KB/s`;
  return `↓ ${bps.toFixed(0)} B/s`;
});
</script>

<template>
  <div class="app" :class="{ 'sidebar-collapsed': !sidebarOpen }">
    <AppSidebar :open="sidebarOpen" @toggle="toggleSidebar" />
    <main class="content">
      <header class="topbar">
        <button v-if="canGoBack" class="icon-btn back-btn" @click="goBack" title="Back">
          ←
        </button>
        <button class="icon-btn" @click="toggleSidebar">☰</button>
        <div class="search" @click="go('search')">
          <span>🔍</span>
          <span class="placeholder">Search galleries, tags...</span>
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
  cursor: pointer;
  max-width: 540px;
}
.search .placeholder {
  font-size: 0.85rem;
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
</style>
