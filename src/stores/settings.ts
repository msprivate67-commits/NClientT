import { defineStore } from "pinia";
import { ref, computed } from "vue";

import {
  authStatus,
  cloudflareCheck,
  cloudflareIsSolved,
  settingsGet,
  settingsSet,
  testTranslationConnection,
  type AuthStatus,
} from "@/api";
import type { Settings } from "@/types";

const DEFAULT_SETTINGS: Settings = {
  mirror: "nhentai.net",
  user_agent: "",
  request_timeout_secs: 30,
  auth: { api_key: "", valid: false },
  proxy_type: "none",
  proxy_host: "",
  proxy_port: 1080,
  proxy_username: "",
  proxy_password: "",
  sort_type: "recent_all_time",
  only_language: "all",
  title_type: "auto",
  exact_tag_match: false,
  remove_avoided_galleries: true,
  show_titles: true,
  column_count: 3,
  page_thumbnail_columns: 0,
  use_rtl: false,
  default_zoom_pct: 100,
  reader_fit_mode: "height",
  reader_direction: "vertical",
  button_change_page: true,
  usage_wifi: "full",
  usage_mobile: "thumbnail",
  keep_history: true,
  max_history: 100,
  favorite_limit: 100,
  download_dir: "",
  parallel_downloads: 1,
  parallel_pages: 8,
  notifications_enabled: true,
  privacy_screen: false,
  lock_screen: false,
  pin: "",
  tl_base_url: "https://api.deepseek.com",
  tl_model: "deepseek-v4-flash",
  tl_api_key: "",
  tl_target_lang: "简体中文，尽量用古典章回体小说标题风格",
  tl_thinking: false,
  tl_auto_translate: true,
  app_language: "",
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
  const translationAvailable = ref<boolean | null>(null);
  const translationStatusMessage = ref("");
  const translationChecking = ref(false);
  let translationCheckId = 0;

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
    // Start the once-per-launch AI probe without delaying the rest of app
    // initialization. GalleryView watches the status and can react when ready.
    void refreshTranslationAvailability();
    await refreshAuth();
    return settings.value;
  }

  async function save(patch: Partial<Settings>) {
    const translationSettingsChanged = [
      "tl_base_url",
      "tl_model",
      "tl_api_key",
      "tl_target_lang",
      "tl_thinking",
      "tl_auto_translate",
    ]
      .some((key) => key in patch && patch[key as keyof Settings] !== settings.value[key as keyof Settings]);
    const next = { ...settings.value, ...patch };
    settings.value = await settingsSet(next);
    if (translationSettingsChanged) await refreshTranslationAvailability();
    return settings.value;
  }

  async function refreshTranslationAvailability() {
    const checkId = ++translationCheckId;
    translationChecking.value = true;
    translationAvailable.value = null;
    translationStatusMessage.value = "";
    try {
      const result = await testTranslationConnection(
        settings.value.tl_base_url,
        settings.value.tl_model,
        settings.value.tl_api_key,
      );
      if (checkId === translationCheckId) {
        translationAvailable.value = result.ok;
        translationStatusMessage.value = result.message;
      }
      return result;
    } finally {
      if (checkId === translationCheckId) translationChecking.value = false;
    }
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
    translationAvailable,
    translationStatusMessage,
    translationChecking,
    mirror,
    baseUrl,
    load,
    save,
    refreshAuth,
    refreshTranslationAvailability,
    checkCloudflare,
    isCloudflareSolved,
  };
});
