import { defineStore } from "pinia";
import { ref } from "vue";

import { localIds } from "@/api";

/**
 * Tracks which galleries the user has downloaded (present on disk in the
 * local library). Mirrors only the id set — the UI badges online covers with
 * a "downloaded" mark and uses this to disable re-downloading.
 *
 * The source of truth is the local-library scan (folders on disk that carry a
 * gallery id), kept in sync via `localIds()` after scans, downloads finish,
 * and deletes.
 */
export const useDownloadedStore = defineStore("downloaded", () => {
  const downloadedIds = ref<Set<number>>(new Set());
  const loaded = ref(false);

  async function load() {
    if (loaded.value) return;
    try {
      const ids = await localIds();
      downloadedIds.value = new Set(ids);
    } catch (e) {
      console.warn("downloaded load failed", e);
    }
    loaded.value = true;
  }

  /** Re-fetch the on-disk id set after a scan / download / delete. */
  async function refresh() {
    try {
      const ids = await localIds();
      downloadedIds.value = new Set(ids);
    } catch (e) {
      console.warn("downloaded refresh failed", e);
    }
    loaded.value = true;
  }

  /** Mark a gallery as downloaded optimistically (e.g. a download just finished). */
  function add(id: number) {
    downloadedIds.value = new Set(downloadedIds.value).add(id);
  }

  function has(id: number): boolean {
    return downloadedIds.value.has(id);
  }

  return { downloadedIds, loaded, load, refresh, add, has };
});
