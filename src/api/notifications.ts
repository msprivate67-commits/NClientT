import { invoke } from "@tauri-apps/api/core";

export const windowsDownloadProgress = (
  title: string,
  status: string,
  valueString: string,
  value: number,
  initial: boolean,
): Promise<void> => invoke("windows_download_progress", {
  title,
  status,
  valueString,
  value,
  initial,
});

export const windowsDownloadComplete = (
  title: string,
  status: string,
  valueString: string,
): Promise<void> => invoke("windows_download_complete", { title, status, valueString });
