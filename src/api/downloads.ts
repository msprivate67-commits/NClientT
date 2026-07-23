import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { DownloadEntry, DownloadProgress, DownloadRequest, DownloadRow } from "@/types";

export const downloadGallery = (req: DownloadRequest): Promise<DownloadEntry> =>
  invoke("download_gallery", { req });
export const downloadRange = (
  galleryId: number,
  fromPage?: number,
  toPage?: number,
): Promise<DownloadEntry> => invoke("download_range", { galleryId, fromPage, toPage });
export const downloadList = (): Promise<DownloadEntry[]> => invoke("download_list");
export const downloadRows = (): Promise<DownloadRow[]> => invoke("download_rows");
export const downloadCancel = (id: number): Promise<void> => invoke("download_cancel", { id });
export const downloadDelete = (id: number): Promise<void> => invoke("download_delete", { id });
export const downloadPause = (id: number): Promise<void> => invoke("download_pause", { id });
export const downloadResume = (id: number): Promise<void> => invoke("download_resume", { id });
export const downloadClear = (): Promise<void> => invoke("download_clear");
export const downloadPauseIds = (ids: number[]): Promise<void> => invoke("download_pause_ids", { ids });
export const downloadResumeIds = (ids: number[]): Promise<void> => invoke("download_resume_ids", { ids });
export const downloadCancelIds = (ids: number[]): Promise<void> => invoke("download_cancel_ids", { ids });
export const downloadDeleteIds = (ids: number[]): Promise<void> => invoke("download_delete_ids", { ids });

export function onDownloadProgress(callback: (progress: DownloadProgress) => void): Promise<UnlistenFn> {
  return listen<DownloadProgress>("download:progress", (event) => callback(event.payload));
}
