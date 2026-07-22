<script setup lang="ts">
import { RouterLink } from "vue-router";
import { useI18n } from "vue-i18n";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  House,
  Search,
  Star,
  Clock,
  Download,
  Folder,
  Tag,
  Settings,
  PackageOpen,
  MessageCircle,
  ChevronLeft,
  ChevronRight,
  X,
} from "lucide-vue-next";

const { t } = useI18n();

defineProps<{
  open: boolean;
  /** When true the sidebar renders as a slide-over drawer (small screens)
   *  that fully hides when closed, instead of leaving an icon rail. */
  mobile?: boolean;
}>();
defineEmits<{ (e: "toggle"): void; (e: "navigate"): void }>();

const GITHUB_RELEASES = "https://github.com/msprivate67-commits/NClientT/releases";
const GITHUB_ISSUES = "https://github.com/msprivate67-commits/NClientT/issues";

function openRelease() {
  openUrl(GITHUB_RELEASES);
}

function openIssue() {
  openUrl(GITHUB_ISSUES);
}

const items = [
  { to: { name: "home" },      icon: House,          key: "sidebar.home" },
  { to: { name: "search" },    icon: Search,         key: "sidebar.search" },
  { to: { name: "favorites" }, icon: Star,           key: "sidebar.favorites" },
  { to: { name: "history" },   icon: Clock,          key: "sidebar.history" },
  { to: { name: "downloads" }, icon: Download,       key: "sidebar.downloads" },
  { to: { name: "local" },     icon: Folder,         key: "sidebar.local_library" },
  { to: { name: "tags" },      icon: Tag,            key: "sidebar.tags" },
  { to: { name: "settings" },  icon: Settings,       key: "sidebar.settings" },
];
</script>

<template>
  <!--
    Two modes share the same markup:
      • desktop (`!mobile`): an in-flow column that collapses to a 56px icon
        rail — the icons stay visible as a quick jump bar.
      • mobile (`mobile`): a fixed slide-over drawer that is entirely off
        screen when closed, so it steals no horizontal space on a phone. A
        backdrop (rendered by App.vue) handles dismiss-on-outside-tap.
  -->
  <aside
    class="sidebar"
    :class="{ collapsed: !open && !mobile, 'is-mobile': mobile, 'is-open': mobile && open }"
  >
    <div class="brand">
      <span class="logo">{{ t("sidebar.short_brand") }}</span>
      <span v-if="open || mobile" class="name">{{ t("sidebar.brand") }}</span>
    </div>
    <nav>
      <RouterLink
        v-for="item in items"
        :key="item.key"
        :to="item.to"
        class="nav-item"
        :title="t(item.key)"
        @click="$emit('navigate')"
      >
        <component :is="item.icon" :size="18" class="icon" />
        <span v-if="open || mobile" class="label">{{ t(item.key) }}</span>
      </RouterLink>

      <div class="nav-sep" />

      <button class="nav-item" :title="t('sidebar.get_latest')" @click="openRelease">
        <PackageOpen :size="18" class="icon" />
        <span v-if="open || mobile" class="label">{{ t('sidebar.get_latest') }}</span>
      </button>
      <button class="nav-item" :title="t('sidebar.feedback')" @click="openIssue">
        <MessageCircle :size="18" class="icon" />
        <span v-if="open || mobile" class="label">{{ t('sidebar.feedback') }}</span>
      </button>
    </nav>
    <button v-if="!mobile" class="collapse" @click="$emit('toggle')">
      <ChevronLeft v-if="open" :size="16" />
      <ChevronRight v-else :size="16" />
    </button>
    <button v-else class="collapse" @click="$emit('toggle')"><X :size="16" /></button>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 200px;
  background: var(--surface);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  transition: width 0.18s ease;
  flex-shrink: 0;
  overflow: hidden;
}
.sidebar.collapsed {
  width: 56px;
}
.brand {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 16px;
  font-weight: 600;
  border-bottom: 1px solid var(--border);
}
.logo {
  display: grid;
  place-items: center;
  width: 28px;
  height: 28px;
  background: var(--accent);
  color: #fff;
  border-radius: 6px;
  font-weight: 700;
  flex-shrink: 0;
}
.name {
  color: var(--text);
}
nav {
  flex: 1;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  border-radius: 6px;
  color: var(--text);
  text-decoration: none;
  font-size: 0.88rem;
  cursor: pointer;
  background: transparent;
  border: none;
  width: 100%;
  text-align: left;
  font-family: inherit;
}
.nav-item:hover {
  background: var(--surface-2);
}
.nav-item.router-link-active {
  background: var(--accent-soft);
  color: var(--accent);
}
.nav-sep {
  height: 1px;
  background: var(--border);
  margin: 4px 12px;
}
.icon {
  width: 20px;
  text-align: center;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}
.collapse {
  margin: 8px;
  padding: 6px;
  background: var(--surface-2);
  border: 1px solid var(--border);
  color: var(--text);
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* ---- Mobile slide-over drawer ----
   Taken out of flow so it overlays content and hides completely when closed
   (no leftover icon rail eating horizontal space on a phone). */
.sidebar.is-mobile {
  position: fixed;
  top: 0;
  bottom: 0;
  left: 0;
  width: 240px;
  max-width: 80vw;
  z-index: 1200;
  transform: translateX(-100%);
  transition: transform 0.22s ease;
  box-shadow: 4px 0 24px rgba(0, 0, 0, 0.4);
}
.sidebar.is-mobile.is-open {
  transform: translateX(0);
}
</style>
