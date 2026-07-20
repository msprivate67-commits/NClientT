<script setup lang="ts">
import { RouterLink } from "vue-router";

defineProps<{ open: boolean }>();
defineEmits<{ (e: "toggle"): void }>();

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
  <aside class="sidebar" :class="{ collapsed: !open }">
    <div class="brand">
      <span class="logo">N</span>
      <span v-if="open" class="name">NClientT</span>
    </div>
    <nav>
      <RouterLink
        v-for="item in items"
        :key="item.label"
        :to="item.to"
        class="nav-item"
        :title="item.label"
      >
        <span class="icon">{{ item.icon }}</span>
        <span v-if="open" class="label">{{ item.label }}</span>
      </RouterLink>
    </nav>
    <button class="collapse" @click="$emit('toggle')">
      {{ open ? "‹" : "›" }}
    </button>
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
</style>
