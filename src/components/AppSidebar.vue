<script setup lang="ts">
import { RouterLink } from "vue-router";

defineProps<{
  open: boolean;
  /** When true the sidebar renders as a slide-over drawer (small screens)
   *  that fully hides when closed, instead of leaving an icon rail. */
  mobile?: boolean;
}>();
defineEmits<{ (e: "toggle"): void; (e: "navigate"): void }>();

const items = [
  { to: { name: "home" }, icon: "🏠", label: "Home" },
  { to: { name: "search" }, icon: "🔍", label: "Search" },
  { to: { name: "favorites" }, icon: "★", label: "Favorites" },
  { to: { name: "history" }, icon: "🕑", label: "History" },
  { to: { name: "downloads" }, icon: "⬇", label: "Downloads" },
  { to: { name: "local" }, icon: "📁", label: "Local Library" },
  { to: { name: "tags" }, icon: "🏷", label: "Tags" },
  { to: { name: "settings" }, icon: "⚙", label: "Settings" },
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
      <span class="logo">N</span>
      <span v-if="open || mobile" class="name">NClientT</span>
    </div>
    <nav>
      <RouterLink
        v-for="item in items"
        :key="item.label"
        :to="item.to"
        class="nav-item"
        :title="item.label"
        @click="$emit('navigate')"
      >
        <span class="icon">{{ item.icon }}</span>
        <span v-if="open || mobile" class="label">{{ item.label }}</span>
      </RouterLink>
    </nav>
    <button v-if="!mobile" class="collapse" @click="$emit('toggle')">
      {{ open ? "‹" : "›" }}
    </button>
    <button v-else class="collapse" @click="$emit('toggle')">✕</button>
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
}
.nav-item:hover {
  background: var(--surface-2);
}
.nav-item.router-link-active {
  background: var(--accent-soft);
  color: var(--accent);
}
.icon {
  font-size: 1rem;
  width: 20px;
  text-align: center;
  flex-shrink: 0;
}
.collapse {
  margin: 8px;
  padding: 6px;
  background: var(--surface-2);
  border: 1px solid var(--border);
  color: var(--text);
  border-radius: 6px;
  cursor: pointer;
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
