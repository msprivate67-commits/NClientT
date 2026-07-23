import { invoke } from "@tauri-apps/api/core";
import type { AuthCredentials, AuthStatus, Settings } from "@/types";

export interface SettingsPaths {
  app_data: string;
  log_dir: string | null;
}

export const settingsGet = (): Promise<Settings> => invoke("settings_get");
export const settingsSet = (settings: Settings): Promise<Settings> =>
  invoke("settings_set", { newSettings: settings });
export const settingsGetPaths = (): Promise<SettingsPaths> => invoke("settings_get_paths");
export const settingsPickDirectory = (): Promise<string | null> => invoke("settings_pick_directory");
export const settingsListDownloadCandidates = (): Promise<[string, string][]> =>
  invoke("settings_list_download_candidates");
export const settingsClearCookies = (): Promise<void> => invoke("settings_clear_cookies");

export const authGet = (): Promise<AuthCredentials> => invoke("auth_get");
export const authSetApiKey = (apiKey: string): Promise<Settings> =>
  invoke("auth_set_api_key", { apiKey });
export const authClear = (): Promise<Settings> => invoke("auth_clear");
export const authStatus = (): Promise<AuthStatus> => invoke("auth_status");

export const cloudflareCheck = (): Promise<boolean> => invoke("cloudflare_check");
export const cloudflareOpenChallenge = (): Promise<void> => invoke("cloudflare_open_challenge");
export const cloudflareIsSolved = (): Promise<boolean> => invoke("cloudflare_is_solved");
