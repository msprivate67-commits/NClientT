import { defineStore } from "pinia";
import { ref } from "vue";

import { readProgressIds, readProgressSet } from "@/api";

/**
 * Tracks how far the user has read into each gallery. The backend stores the
 * furthest page reached; a gallery is considered "read" once >= 50% of its
 * pages have been viewed. We mirror only the finished set here (what the UI
 * badges on covers) — full per-gallery progress is fetched on demand.
 */
export const useReadProgressStore = defineStore("readProgress", () => {
  // IDs of galleries the user has finished (>= 50% read).
  const readIds = ref<Set<number>>(new Set());
  const loaded = ref(false);

  async function load() {
    if (loaded.value) return;
    try {
      const ids = await readProgressIds();
      readIds.value = new Set(ids);
    } catch (e) {
      console.warn("read progress load failed", e);
    }
    loaded.value = true;
  }

  /** Record the furthest page reached and refresh the finished set if needed. */
  async function report(galleryId: number, lastPage: number, totalPages: number) {
    // Optimistic: flip the badge immediately when crossing the 50% threshold
    // so the cover updates without waiting for a round-trip.
    if (totalPages > 0 && lastPage * 2 >= totalPages) {
      readIds.value = new Set(readIds.value).add(galleryId);
    }
    try {
      await readProgressSet(galleryId, lastPage, totalPages);
      // Only pay for a full re-sync if our optimistic guess didn't already
      // mark it — keeps the common case to a single write.
      const ids = await readProgressIds();
      readIds.value = new Set(ids);
    } catch (e) {
      console.warn("read progress report failed", e);
    }
  }

  function has(id: number): boolean {
    return readIds.value.has(id);
  }

  return { readIds, loaded, load, report, has };
});
