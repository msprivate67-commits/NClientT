<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { save as taSave, open as taOpen } from "@tauri-apps/plugin-dialog";
import { writeTextFile, readTextFile } from "@tauri-apps/plugin-fs";
import { useI18n } from "vue-i18n";
import { Check } from "lucide-vue-next";
import { SUPPORTED_LANGUAGES, exportLocaleJson, applyImportedMessages, setLocale, getLocale, type AppLanguage } from "@/i18n";

import {
  authClear,
  authSetApiKey,
  cloudflareCheck,
  cloudflareIsSolved,
  cloudflareOpenChallenge,
  settingsClearCookies,
  settingsGetPaths,
  settingsPickDirectory,
  settingsListDownloadCandidates,
} from "@/api";
import { useSettingsStore } from "@/stores/settings";
import { useScrollCache } from "@/composables/useScrollCache";

const i18n = useI18n();

const settings = useSettingsStore();
const draft = ref(JSON.parse(JSON.stringify(settings.settings)));
const saved = ref(false);
const appData = ref<string>("");
const viewRef = ref<HTMLElement | null>(null);
useScrollCache(viewRef);
const apiKeyInput = ref("");
const cfNeeded = ref(false);
const cfSolved = ref(false);

const currentLang = ref<string>(getLocale());
const importMissing = ref<string[] | null>(null);
const importError = ref("");
const langSaved = ref(false);

const dirty = computed(() => JSON.stringify(draft.value) !== JSON.stringify(settings.settings));

async function save() {
  await settings.save(draft.value);
  draft.value = JSON.parse(JSON.stringify(settings.settings));
  saved.value = true;
  setTimeout(() => (saved.value = false), 1500);
}

async function pickDownloadDir() {
  try {
    const selected = await taOpen({ directory: true, multiple: false });
    if (typeof selected === "string") {
      draft.value.download_dir = selected;
      return;
    }
    if (selected === null) return;
  } catch {
    // Dialog might fail on some platforms (e.g. Android) — try backend fallback
  }
  // No native dialog (typical on Android): offer the backend-provided
  // candidate directories so the user can switch between public Download,
  // the app's own external storage, and internal storage.
  try {
    const candidates = await settingsListDownloadCandidates();
    if (candidates.length === 0) {
      const picked = await settingsPickDirectory();
      if (picked) draft.value.download_dir = picked;
      return;
    }
    if (candidates.length === 1) {
      draft.value.download_dir = candidates[0][1];
      return;
    }
    const menu = candidates
      .map(([label, path], i) => `${i + 1}. ${label}\n   ${path}`)
      .join("\n\n");
    const choice = window.prompt(
      `Choose a download directory (enter the number):\n\n${menu}`,
      "2",
    );
    const idx = Number.parseInt((choice ?? "").trim(), 10) - 1;
    if (Number.isInteger(idx) && idx >= 0 && idx < candidates.length) {
      draft.value.download_dir = candidates[idx][1];
    }
  } catch {
    // Both failed; user can type manually in the editable field
  }
}

async function saveApiKey() {
  if (!apiKeyInput.value.trim()) return;
  await authSetApiKey(apiKeyInput.value.trim());
  apiKeyInput.value = "";
  await settings.refreshAuth();
}

async function clearAuth() {
  await authClear();
  await settings.refreshAuth();
  draft.value.auth = { api_key: "", valid: false };
}

async function clearCookies() {
  await settingsClearCookies();
}

async function checkCf() {
  cfNeeded.value = await cloudflareCheck();
  cfSolved.value = await cloudflareIsSolved();
}

async function solveCf() {
  await cloudflareOpenChallenge();
}

async function changeLang(code: AppLanguage) {
  currentLang.value = code;
  setLocale(code);
  i18n.locale.value = code;
  await settings.save({ app_language: code });
  draft.value = JSON.parse(JSON.stringify(settings.settings));
  langSaved.value = true;
  setTimeout(() => langSaved.value = false, 1500);
}

async function exportLang() {
  try {
    const json = exportLocaleJson(currentLang.value);
    const path = await taSave({
      defaultPath: `nclientt-${currentLang.value}.json`,
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (path) {
      await writeTextFile(path, json);
    }
  } catch (e) {
    console.error("export failed", e);
  }
}

async function importLang() {
  importError.value = "";
  importMissing.value = null;
  try {
    const selected = await taOpen({
      multiple: false,
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!selected) return;
    const path = selected as string;
    const content = await readTextFile(path);
    const { missingKeys } = applyImportedMessages(currentLang.value, content);
    importMissing.value = missingKeys;
    i18n.locale.value = currentLang.value;
    langSaved.value = true;
    setTimeout(() => langSaved.value = false, 1500);
  } catch (e: any) {
    importError.value = String(e?.message ?? e);
  }
}

onMounted(async () => {
  draft.value = JSON.parse(JSON.stringify(settings.settings));
  try {
    const paths = await settingsGetPaths();
    appData.value = paths.app_data;
  } catch (e) {
    console.warn(e);
  }
  checkCf();
});
</script>

<template>
  <div ref="viewRef" class="view settings">
    <div class="view-header">
      <div class="view-title">{{ $t('settings.title') }}</div>
    </div>

    <section>
      <div class="section-title">{{ $t('settings.section_language') }}</div>
      <div class="row">
        <label style="min-width: 120px;">{{ $t('settings.app_language') }}</label>
        <select v-model="currentLang" @change="changeLang(currentLang as AppLanguage)">
          <option v-for="lang in SUPPORTED_LANGUAGES" :key="lang.code" :value="lang.code">
            {{ lang.nativeName }}
          </option>
        </select>
        <span v-if="langSaved" class="ok" style="font-size:0.82rem;">{{ $t('common.saved') }}</span>
      </div>
      <p class="hint">{{ $t('settings.app_language_hint') }}</p>
      <div class="row" style="margin-top: 8px;">
        <button class="btn" @click="exportLang">{{ $t('settings.export_lang') }}</button>
        <button class="btn" @click="importLang">{{ $t('settings.import_lang') }}</button>
      </div>
      <p class="hint" style="margin-top: 4px;">{{ $t('settings.export_lang_hint') }}</p>
      <p class="hint">{{ $t('settings.import_lang_hint') }}</p>
      <div v-if="importError" class="tl-error" style="margin-top: 6px;">{{ importError }}</div>
      <div v-if="importMissing !== null && importMissing.length > 0" class="error" style="margin-top: 6px; font-size:0.8rem;">
        Missing keys ({{ importMissing.length }}) — falling back to English:
        <div style="max-height:80px; overflow-y:auto; margin-top:4px;">
          <span v-for="k in importMissing" :key="k" style="display:inline-block; margin:2px 4px; background:var(--surface-2); padding:1px 6px; border-radius:4px; font-size:0.7rem;">{{ k }}</span>
        </div>
      </div>
      <div v-else-if="importMissing !== null && importMissing.length === 0" class="ok" style="margin-top: 6px; font-size:0.82rem;">
        <Check :size="14" /> All keys present
      </div>
    </section>

    <section>
      <div class="section-title">{{ $t('settings.section_site') }}</div>
      <div class="fields">
        <div class="field">
          <label>{{ $t('settings.mirror_host') }}</label>
          <input v-model="draft.mirror" type="text" placeholder="nhentai.net" />
        </div>
        <div class="field">
          <label>{{ $t('settings.user_agent') }}</label>
          <input v-model="draft.user_agent" type="text" placeholder="NClientT/0.1.0 ..." />
        </div>
        <div class="field">
          <label>{{ $t('settings.request_timeout') }}</label>
          <input v-model.number="draft.request_timeout_secs" type="number" min="5" max="300" />
        </div>
      </div>
    </section>

    <section>
      <div class="section-title">{{ $t('settings.section_proxy') }}</div>
      <div class="fields">
        <div class="field">
          <label>{{ $t('settings.proxy_type') }}</label>
          <select v-model="draft.proxy_type">
            <option value="none">{{ $t('settings.proxy_none') }}</option>
            <option value="http">{{ $t('settings.proxy_http') }}</option>
            <option value="socks5">{{ $t('settings.proxy_socks5') }}</option>
          </select>
        </div>
        <div class="field">
          <label>{{ $t('settings.host') }}</label>
          <input v-model="draft.proxy_host" type="text" placeholder="127.0.0.1" :disabled="draft.proxy_type === 'none'" />
        </div>
        <div class="field">
          <label>{{ $t('settings.port') }}</label>
          <input v-model.number="draft.proxy_port" type="number" min="1" max="65535" :disabled="draft.proxy_type === 'none'" />
        </div>
      </div>
      <div class="fields" style="margin-top: 8px;">
        <div class="field">
          <label>{{ $t('settings.username_optional') }}</label>
          <input v-model="draft.proxy_username" type="text" placeholder="optional" :disabled="draft.proxy_type === 'none'" />
        </div>
        <div class="field">
          <label>{{ $t('settings.password_optional') }}</label>
          <input v-model="draft.proxy_password" type="password" placeholder="optional" :disabled="draft.proxy_type === 'none'" />
        </div>
      </div>
    </section>

    <section>
      <div class="section-title">{{ $t('settings.section_cloudflare') }}</div>
      <div class="row">
        <span>{{ $t('settings.cf_status') }}</span>
        <strong v-if="cfSolved" class="ok">{{ $t('settings.cf_solved') }}</strong>
        <strong v-else-if="cfNeeded" class="warn">{{ $t('settings.cf_challenge_needed') }}</strong>
        <strong v-else>{{ $t('settings.cf_unknown') }}</strong>
        <button class="btn" @click="checkCf">{{ $t('settings.check') }}</button>
        <button class="btn" :disabled="cfSolved" @click="solveCf">{{ $t('settings.solve') }}</button>
        <button class="btn" @click="clearCookies">{{ $t('settings.clear_cookies') }}</button>
      </div>
    </section>

    <section>
      <div class="section-title">{{ $t('settings.section_api') }}</div>
      <div class="row">
        <span>{{ $t('settings.has_key') }}</span>
        <strong>{{ settings.auth.has_credentials ? $t('common.yes') : $t('common.no') }}</strong>
        <strong v-if="settings.auth.has_credentials" :class="{ ok: settings.auth.api_key_valid, warn: !settings.auth.api_key_valid }">
          {{ settings.auth.api_key_valid ? $t('settings.key_valid') : $t('settings.key_invalid') }}
        </strong>
      </div>
      <div class="row">
        <input v-model="apiKeyInput" type="password" :placeholder="$t('settings.paste_api_key')" />
        <button class="btn primary" @click="saveApiKey">{{ $t('settings.save_key') }}</button>
        <button class="btn danger" :disabled="!settings.auth.has_credentials" @click="clearAuth">{{ $t('settings.clear') }}</button>
      </div>
      <p class="hint" v-html="$t('settings.api_hint', { code: '<code>Authorization: Key &lt;key&gt;</code>' })"></p>
    </section>

    <section>
      <div class="section-title">{{ $t('settings.section_display') }}</div>
      <div class="fields">
        <div class="field">
          <label>{{ $t('settings.grid_columns') }}</label>
          <input v-model.number="draft.column_count" type="number" min="2" max="10" />
        </div>
        <div class="field">
          <label>{{ $t('settings.page_thumb_columns') }}</label>
          <input v-model.number="draft.page_thumbnail_columns" type="number" min="0" max="10" />
        </div>
        <div class="field">
          <label>{{ $t('settings.default_zoom') }}</label>
          <input v-model.number="draft.default_zoom_pct" type="number" min="20" max="300" />
        </div>
      </div>
      <div class="checkboxes">
        <label><input type="checkbox" v-model="draft.button_change_page" /> {{ $t('settings.page_change_buttons') }}</label>
      </div>
    </section>

    <section>
      <div class="section-title">{{ $t('settings.section_downloads') }}</div>
      <div class="field path-field">
        <label>{{ $t('settings.download_dir') }}</label>
        <div class="row">
          <input v-model="draft.download_dir" type="text" :placeholder="$t('settings.download_dir_placeholder')" />
          <button class="btn" @click="pickDownloadDir">{{ $t('settings.browse') }}</button>
        </div>
      </div>
    </section>

    <section>
      <div class="section-title">{{ $t('settings.section_ai') }}</div>
      <div class="fields">
        <div class="field">
          <label>{{ $t('settings.ai_base_url') }}</label>
          <input v-model="draft.tl_base_url" type="text" placeholder="https://api.deepseek.com" />
        </div>
        <div class="field">
          <label>{{ $t('settings.ai_model') }}</label>
          <input v-model="draft.tl_model" type="text" placeholder="deepseek-v4-flash" />
        </div>
        <div class="field">
          <label>{{ $t('settings.ai_api_key') }}</label>
          <input v-model="draft.tl_api_key" type="password" placeholder="sk-…" />
        </div>
        <div class="field">
          <label>{{ $t('settings.ai_target_lang') }}</label>
          <input v-model="draft.tl_target_lang" type="text" placeholder="中文" />
        </div>
      </div>
      <div class="checkboxes">
        <label><input type="checkbox" v-model="draft.tl_thinking" /> {{ $t('settings.ai_thinking') }}</label>
      </div>
    </section>

    <section v-if="appData">
      <div class="section-title">{{ $t('settings.section_data') }}</div>
      <p class="hint">{{ $t('settings.app_data_dir') }} <code>{{ appData }}</code></p>
    </section>

    <div class="save-bar">
      <button class="btn primary" :disabled="!dirty" @click="save">
        {{ saved ? $t('common.saved') : $t('common.save') }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.settings {
  max-width: 720px;
}
section {
  margin-bottom: 22px;
  padding-bottom: 18px;
  border-bottom: 1px solid var(--border);
}
section:last-child {
  border-bottom: none;
}
.fields {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 10px;
}
.row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
  flex-wrap: wrap;
}
.row input[type="text"],
.row input[type="password"],
.row input:read-only {
  flex: 1;
  min-width: 200px;
  background: var(--surface);
  border: 1px solid var(--border);
  color: var(--text);
  border-radius: 6px;
  padding: 6px 10px;
}
.checkboxes {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-top: 10px;
}
.checkboxes label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.85rem;
  color: var(--text);
}
.hint {
  font-size: 0.78rem;
  color: var(--text-dim);
  margin: 6px 0 0;
}
.hint code {
  background: var(--surface-2);
  padding: 1px 5px;
  border-radius: 4px;
}
.ok {
  color: #6ec16e;
}
.warn {
  color: #ffce80;
}
.save-bar {
  display: flex;
  justify-content: flex-end;
  padding: 16px 0 8px;
}
.save-bar .btn {
  font-size: 1rem;
  font-weight: 600;
  padding: 10px 32px;
}
</style>
