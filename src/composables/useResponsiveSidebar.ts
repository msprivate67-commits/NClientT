import { onMounted, onUnmounted, ref } from "vue";

const DEFAULT_COMPACT_QUERY = "(max-width: 760px)";

export function useResponsiveSidebar(query = DEFAULT_COMPACT_QUERY) {
  const isCompact = ref(false);
  const sidebarOpen = ref(true);
  let mediaQuery: MediaQueryList | null = null;

  function syncLayout() {
    isCompact.value = window.matchMedia(query).matches;
    sidebarOpen.value = !isCompact.value;
  }

  function toggleSidebar() {
    sidebarOpen.value = !sidebarOpen.value;
  }

  onMounted(() => {
    syncLayout();
    mediaQuery = window.matchMedia(query);
    mediaQuery.addEventListener("change", syncLayout);
  });

  onUnmounted(() => {
    mediaQuery?.removeEventListener("change", syncLayout);
    mediaQuery = null;
  });

  return { isCompact, sidebarOpen, toggleSidebar };
}
