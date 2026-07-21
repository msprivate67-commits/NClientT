import { defineStore } from "pinia";
import { ref } from "vue";

export const useOverlayStore = defineStore("overlay", () => {
  const galleryId = ref<number | null>(null);
  const readerId = ref<number | null>(null);

  function openGallery(id: number) {
    galleryId.value = id;
    readerId.value = null;
  }

  function openReader(id: number) {
    readerId.value = id;
  }

  function pop() {
    if (readerId.value) {
      readerId.value = null;
    } else if (galleryId.value) {
      galleryId.value = null;
    }
  }

  function closeAll() {
    galleryId.value = null;
    readerId.value = null;
  }

  function hasAny() {
    return galleryId.value !== null || readerId.value !== null;
  }

  return { galleryId, readerId, openGallery, openReader, pop, closeAll, hasAny };
});
