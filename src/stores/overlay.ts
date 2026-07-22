import { defineStore } from "pinia";
import { ref } from "vue";

export const useOverlayStore = defineStore("overlay", () => {
  const galleryId = ref<number | null>(null);
  const readerId = ref<number | null>(null);
  const readerPage = ref<number | null>(null);
  const localDetailFolder = ref<string | null>(null);
  const localReaderFolder = ref<string | null>(null);

  function openGallery(id: number) {
    galleryId.value = id;
    localDetailFolder.value = null;
    readerId.value = null;
    localReaderFolder.value = null;
  }

  function openReader(id: number, page?: number) {
    readerId.value = id;
    readerPage.value = page ?? null;
    localReaderFolder.value = null;
  }

  function openLocalDetail(folder: string) {
    localDetailFolder.value = folder;
    galleryId.value = null;
    readerId.value = null;
    localReaderFolder.value = null;
  }

  function openLocalReader(folder: string) {
    localReaderFolder.value = folder;
    readerId.value = null;
  }

  function pop() {
    if (localReaderFolder.value) {
      localReaderFolder.value = null;
    } else if (readerId.value) {
      readerId.value = null;
      readerPage.value = null;
    } else if (localDetailFolder.value) {
      localDetailFolder.value = null;
    } else if (galleryId.value) {
      galleryId.value = null;
    }
  }

  function closeAll() {
    galleryId.value = null;
    readerId.value = null;
    readerPage.value = null;
    localDetailFolder.value = null;
    localReaderFolder.value = null;
  }

  function hasAny() {
    return galleryId.value !== null || readerId.value !== null
      || localDetailFolder.value !== null || localReaderFolder.value !== null;
  }

  return {
    galleryId, readerId, readerPage,
    localDetailFolder, localReaderFolder,
    openGallery, openReader, openLocalDetail, openLocalReader,
    pop, closeAll, hasAny,
  };
});
