import { defineStore } from "pinia";
import { ref } from "vue";

export const useOverlayStore = defineStore("overlay", () => {
  const galleryId = ref<number | null>(null);
  const readerId = ref<number | null>(null);
  const readerPage = ref<number | null>(null);

  function openGallery(id: number) {
    galleryId.value = id;
    readerId.value = null;
  }

  function openReader(id: number, page?: number) {
    readerId.value = id;
    readerPage.value = page ?? null;
  }

  function pop() {
    if (readerId.value) {
      readerId.value = null;
      readerPage.value = null;
    } else if (galleryId.value) {
      galleryId.value = null;
    }
  }

  function closeAll() {
    galleryId.value = null;
    readerId.value = null;
    readerPage.value = null;
  }

  function hasAny() {
    return galleryId.value !== null || readerId.value !== null;
  }

  return { galleryId, readerId, readerPage, openGallery, openReader, pop, closeAll, hasAny };
});
