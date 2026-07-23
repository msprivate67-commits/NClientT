import { defineStore } from "pinia";
import { ref, computed } from "vue";

import {
  downloadClear,
  downloadDelete,
  downloadGallery,
  downloadList,
  downloadPause,
  downloadRange,
  downloadResume,
  downloadCancel,
  downloadPauseIds,
  downloadResumeIds,
  downloadCancelIds,
  downloadDeleteIds,
  onDownloadProgress,
  type DownloadEntry,
  type DownloadProgress,
  type DownloadRequest,
} from "@/api";

export const useDownloadsStore = defineStore("downloads", () => {
  const items = ref<DownloadEntry[]>([]);
  const initialized = ref(false);
  const selected = ref<Set<number>>(new Set());
  let unlisten: (() => void) | null = null;

  const selectable = computed(() =>
    items.value.filter((i) => i.status !== "finished" && i.status !== "canceled")
  );
  const allSelected = computed(() => {
    const sel = selectable.value;
    return sel.length > 0 && sel.every((i) => selected.value.has(i.id));
  });
  const selectedCount = computed(() => selected.value.size);

  function toggleSelect(id: number) {
    const s = new Set(selected.value);
    if (s.has(id)) s.delete(id);
    else s.add(id);
    selected.value = s;
  }
  function selectAll() {
    if (allSelected.value) {
      selected.value = new Set();
    } else {
      selected.value = new Set(selectable.value.map((i) => i.id));
    }
  }
  function clearSelection() {
    selected.value = new Set();
  }

  function applyProgress(p: DownloadProgress) {
    if (p.status === "canceled") {
      items.value = items.value.filter((i) => i.id !== p.id);
      return;
    }
    const idx = items.value.findIndex((i) => i.id === p.id);
    const entry: DownloadEntry = {
      id: p.id,
      title: p.title,
      folder: p.folder,
      thumbnail: null,
      status: p.status,
      done_pages: p.done_pages,
      total_pages: p.total_pages,
      bytes_per_second: p.bytes_per_second,
      total_bytes: p.total_bytes,
    };
    if (idx >= 0) {
      items.value[idx] = { ...items.value[idx], ...entry };
    } else {
      items.value.unshift(entry);
    }
  }

  async function init() {
    if (initialized.value) return;
    items.value = await downloadList();
    unlisten = await onDownloadProgress((p) => applyProgress(p));
    initialized.value = true;
  }

  function dispose() {
    unlisten?.();
    unlisten = null;
    initialized.value = false;
  }

  async function refresh() {
    items.value = await downloadList();
  }

  async function enqueue(req: DownloadRequest) {
    const entry = await downloadGallery(req);
    const idx = items.value.findIndex((i) => i.id === entry.id);
    if (idx >= 0) items.value[idx] = entry;
    else items.value.unshift(entry);
    return entry;
  }

  async function enqueueRange(galleryId: number, fromPage?: number, toPage?: number) {
    const entry = await downloadRange(galleryId, fromPage, toPage);
    const idx = items.value.findIndex((i) => i.id === entry.id);
    if (idx >= 0) items.value[idx] = entry;
    else items.value.unshift(entry);
    return entry;
  }

  async function pause(id: number) {
    await downloadPause(id);
  }
  async function resume(id: number) {
    await downloadResume(id);
  }
  async function cancel(id: number) {
    await downloadCancel(id);
    items.value = items.value.filter((i) => i.id !== id);
    selected.value = new Set();
  }
  async function deleteDownload(id: number) {
    await downloadDelete(id);
    items.value = items.value.filter((i) => i.id !== id);
    selected.value = new Set();
  }
  async function clear() {
    await downloadClear();
    items.value = items.value.filter((i) => i.status !== "finished");
    selected.value = new Set();
  }

  /**
   * Return the live queue entry for a gallery. Completed, paused, canceled,
   * and failed rows can remain in the download history, but none of them
   * should make the UI claim that the gallery is currently downloading.
   */
  function activeForGallery(id: number): DownloadEntry | null {
    return items.value.find(
      (i) => i.id === id && (i.status === "pending" || i.status === "downloading"),
    ) ?? null;
  }

  // ── batch operations ────────────────────────────────────────────────
  async function batchPause() {
    const ids = [...selected.value];
    await downloadPauseIds(ids);
    clearSelection();
  }
  async function batchResume() {
    const ids = [...selected.value];
    await downloadResumeIds(ids);
    clearSelection();
  }
  async function batchCancel() {
    const ids = [...selected.value];
    await downloadCancelIds(ids);
    items.value = items.value.filter((i) => !ids.includes(i.id));
    clearSelection();
  }
  async function batchDelete() {
    const ids = [...selected.value];
    await downloadDeleteIds(ids);
    items.value = items.value.filter((i) => !ids.includes(i.id));
    clearSelection();
  }

  const hasActive = computed(() => items.value.some((i) => i.status === "downloading"));
  const totalSpeed = computed(() => {
    let sum = 0;
    for (const i of items.value) {
      if (i.status === "downloading" && i.bytes_per_second != null) {
        sum += i.bytes_per_second;
      }
    }
    return sum;
  });

  return {
    items,
    selected,
    selectable,
    allSelected,
    selectedCount,
    hasActive,
    totalSpeed,
    init,
    dispose,
    refresh,
    enqueue,
    enqueueRange,
    toggleSelect,
    selectAll,
    clearSelection,
    pause,
    resume,
    cancel,
    deleteDownload,
    clear,
    activeForGallery,
    batchPause,
    batchResume,
    batchCancel,
    batchDelete,
  };
});
