<script setup lang="ts">
import { onMounted, ref } from "vue";
import { RouterView, useRouter } from "vue-router";

import AppSidebar from "@/components/AppSidebar.vue";
import { cloudflareOpenChallenge } from "@/api";
import { useSettingsStore } from "@/stores/settings";
import { useDownloadsStore } from "@/stores/downloads";
import { useFavoritesStore } from "@/stores/favorites";
import { useTagsStore } from "@/stores/tags";

const settings = useSettingsStore();
const downloads = useDownloadsStore();
const favorites = useFavoritesStore();
const tags = useTagsStore();
const router = useRouter();

const sidebarOpen = ref(true);
const cloudflareBanner = ref(false);

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
</script>

<template>
  <div class="app" :class="{ 'sidebar-collapsed': !sidebarOpen }">
    <AppSidebar :open="sidebarOpen" @toggle="toggleSidebar" />
    <main class="content">
      <header class="topbar">
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

      <RouterView />
    </main>
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
</style>
