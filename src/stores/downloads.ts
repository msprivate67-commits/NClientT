import { defineStore } from "pinia";
import { ref, computed } from "vue";

import {
  downloadClear,
  downloadGallery,
  downloadList,
  downloadPause,
  downloadRange,
  downloadResume,
  downloadCancel,
  onDownloadProgress,
  type DownloadEntry,
  type DownloadProgress,
  type DownloadRequest,
} from "@/api";

export const useDownloadsStore = defineStore("downloads", () => {
  const items = ref<DownloadEntry[]>([]);
  const initialized = ref(false);
  let unlisten: (() => void) | null = null;

  function applyProgress(p: DownloadProgress) {
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
  }
  async function clear() {
    await downloadClear();
    items.value = items.value.filter((i) => i.status !== "finished");
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
    hasActive,
    totalSpeed,
    init,
    dispose,
    refresh,
    enqueue,
    enqueueRange,
    pause,
    resume,
    cancel,
    clear,
  };
});
