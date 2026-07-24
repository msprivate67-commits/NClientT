import { convertFileSrc, invoke } from "@tauri-apps/api/core";

export const exportPdf = (folder: string, out?: string): Promise<string> =>
  invoke("export_pdf", { folder, out });
export const exportZip = (folder: string, out?: string): Promise<string> =>
  invoke("export_zip", { folder, out });

export const openInBrowser = (path: string): Promise<void> => invoke("open_in_browser", { path });
export const openPath = (path: string): Promise<void> => invoke("open_path", { path });
export const resolveAsset = (path: string): Promise<string> => invoke("resolve_asset", { path });
export const readLocalImage = (path: string): Promise<string | null> =>
  invoke("read_local_image", { path });
export const registerApp = (): Promise<void> => invoke("register_app");
export const androidPrivacySet = (enabled: boolean): Promise<void> =>
  invoke("android_privacy_set", { enabled });

export function imageProxyUrl(url: string): string {
  if (!url) return "";
  return convertFileSrc(url, "nclient-image");
}

export interface LatestRelease {
  tag: string;
  name: string;
  html_url: string;
  is_newer: boolean;
  prerelease: boolean;
}

export const getLatestRelease = (): Promise<LatestRelease | null> =>
  invoke("get_latest_release");
