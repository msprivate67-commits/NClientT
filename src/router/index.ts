import { createRouter, createWebHistory } from "vue-router";

const routes = [
  {
    name: "home",
    path: "/",
    component: () => import("@/views/HomeView.vue"),
  },
  {
    name: "search",
    path: "/search",
    component: () => import("@/views/SearchView.vue"),
  },
  {
    name: "gallery",
    path: "/gallery/:id",
    component: () => import("@/views/GalleryView.vue"),
    props: true,
  },
  {
    name: "reader",
    path: "/gallery/:id/read",
    component: () => import("@/views/ReaderView.vue"),
    props: true,
  },
  {
    name: "reader-local",
    path: "/local/:folder",
    component: () => import("@/views/LocalReaderView.vue"),
    props: true,
  },
  {
    name: "favorites",
    path: "/favorites",
    component: () => import("@/views/FavoritesView.vue"),
  },
  {
    name: "history",
    path: "/history",
    component: () => import("@/views/HistoryView.vue"),
  },
  {
    name: "downloads",
    path: "/downloads",
    component: () => import("@/views/DownloadsView.vue"),
  },
  {
    name: "local",
    path: "/local",
    component: () => import("@/views/LocalLibraryView.vue"),
  },
  {
    name: "tags",
    path: "/tags",
    component: () => import("@/views/TagsView.vue"),
  },
  {
    name: "settings",
    path: "/settings",
    component: () => import("@/views/SettingsView.vue"),
  },
];

export const router = createRouter({
  history: createWebHistory(),
  routes,
  scrollBehavior(_to, _from, saved) {
    if (saved) return saved;
    return false;
  },
});
