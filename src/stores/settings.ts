import { defineStore } from "pinia";
import { ref, computed } from "vue";

import {
  authStatus,
  cloudflareCheck,
  cloudflareIsSolved,
  settingsGet,
  settingsSet,
  type AuthStatus,
} from "@/api";
import type { Settings } from "@/types";

const DEFAULT_SETTINGS: Settings = {
  mirror: "nhentai.net",
  user_agent: "",
  request_timeout_secs: 30,
  auth: { api_key: "", valid: false },
  sort_type: "recent_all_time",
  only_language: "all",
  title_type: "auto",
  exact_tag_match: false,
  remove_avoided_galleries: true,
  show_titles: true,
  column_count: 3,
  use_rtl: false,
  default_zoom_pct: 100,
  button_change_page: true,
  usage_wifi: "full",
  usage_mobile: "thumbnail",
  keep_history: true,
  max_history: 100,
  favorite_limit: 100,
  download_dir: "",
  parallel_downloads: 3,
  parallel_pages: 8,
  lock_screen: false,
  pin: "",
};

export const useSettingsStore = defineStore("settings", () => {
  const settings = ref<Settings>({ ...DEFAULT_SETTINGS });
  const loaded = ref(false);
  const auth = ref<AuthStatus>({
    has_credentials: false,
    api_key_valid: false,
    cloudflare_solved: false,
  });
  const cloudflareNeeded = ref(false);

  const mirror = computed(() => settings.value.mirror);
  const baseUrl = computed(() => `https://${settings.value.mirror}/`);

  async function load() {
    if (loaded.value) return settings.value;
    settings.value = await settingsGet();
    if (!settings.value.download_dir) {
      // Defensive default (backend normally fills this).
      settings.value.download_dir = "";
    }
    loaded.value = true;
    await refreshAuth();
    return settings.value;
  }

  async function save(patch: Partial<Settings>) {
    const next = { ...settings.value, ...patch };
    settings.value = await settingsSet(next);
    return settings.value;
  }

  async function refreshAuth() {
    auth.value = await authStatus();
    return auth.value;
  }

  async function checkCloudflare() {
    try {
      cloudflareNeeded.value = await cloudflareCheck();
    } catch (e) {
      console.warn("cloudflare check failed", e);
      cloudflareNeeded.value = false;
    }
    return cloudflareNeeded.value;
  }

  async function isCloudflareSolved() {
    return cloudflareIsSolved();
  }

  return {
    settings,
    loaded,
    auth,
    cloudflareNeeded,
    mirror,
    baseUrl,
    load,
    save,
    refreshAuth,
    checkCloudflare,
    isCloudflareSolved,
  };
});
