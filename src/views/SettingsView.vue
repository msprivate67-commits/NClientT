<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";

import {
  authClear,
  authSetApiKey,
  cloudflareCheck,
  cloudflareIsSolved,
  cloudflareOpenChallenge,
  settingsClearCookies,
  settingsGetPaths,
} from "@/api";
import { useSettingsStore } from "@/stores/settings";
import type { Language, SortType, TitleType } from "@/types";

const settings = useSettingsStore();
const draft = ref(JSON.parse(JSON.stringify(settings.settings)));
const saved = ref(false);
const appData = ref<string>("");
const apiKeyInput = ref("");
const cfNeeded = ref(false);
const cfSolved = ref(false);

const sorts: { value: SortType; label: string }[] = [
  { value: "recent_all_time", label: "Recent" },
  { value: "popular_all_time", label: "Popular (all)" },
  { value: "popular_week", label: "Popular (week)" },
  { value: "popular_day", label: "Popular (day)" },
  { value: "popular_month", label: "Popular (month)" },
];
const langs: { value: Language; label: string }[] = [
  { value: "all", label: "All" },
  { value: "english", label: "English" },
  { value: "japanese", label: "Japanese" },
  { value: "chinese", label: "Chinese" },
];
const titleTypes: { value: TitleType; label: string }[] = [
  { value: "auto", label: "Auto" },
  { value: "pretty", label: "Pretty" },
  { value: "english", label: "English" },
  { value: "japanese", label: "Japanese" },
];

const dirty = computed(() => JSON.stringify(draft.value) !== JSON.stringify(settings.settings));

async function save() {
  await settings.save(draft.value);
  draft.value = JSON.parse(JSON.stringify(settings.settings));
  saved.value = true;
  setTimeout(() => (saved.value = false), 1500);
}

async function pickDownloadDir() {
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected === "string") {
    draft.value.download_dir = selected;
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
  <div class="view settings">
    <div class="view-header">
      <div class="view-title">Settings</div>
      <div class="toolbar">
        <button class="btn primary" :disabled="!dirty" @click="save">
          {{ saved ? "Saved ✓" : "Save" }}
        </button>
      </div>
    </div>

    <section>
      <div class="section-title">Site & Network</div>
      <div class="fields">
        <div class="field">
          <label>Mirror host</label>
          <input v-model="draft.mirror" type="text" placeholder="nhentai.net" />
        </div>
        <div class="field">
          <label>User-Agent (blank = default)</label>
          <input v-model="draft.user_agent" type="text" placeholder="NClientT/0.1.0 ..." />
        </div>
        <div class="field">
          <label>Request timeout (seconds)</label>
          <input v-model.number="draft.request_timeout_secs" type="number" min="5" max="300" />
        </div>
      </div>
    </section>

    <section>
      <div class="section-title">Cloudflare</div>
      <div class="row">
        <span>Status:</span>
        <strong v-if="cfSolved" class="ok">Solved</strong>
        <strong v-else-if="cfNeeded" class="warn">Challenge needed</strong>
        <strong v-else>Unknown</strong>
        <button class="btn" @click="checkCf">Check</button>
        <button class="btn" :disabled="cfSolved" @click="solveCf">Solve</button>
        <button class="btn" @click="clearCookies">Clear cookies</button>
      </div>
    </section>

    <section>
      <div class="section-title">API Key Authentication</div>
      <div class="row">
        <span>Has key:</span>
        <strong>{{ settings.auth.has_credentials ? "yes" : "no" }}</strong>
        <strong v-if="settings.auth.has_credentials" :class="{ ok: settings.auth.api_key_valid, warn: !settings.auth.api_key_valid }">
          {{ settings.auth.api_key_valid ? "(valid)" : "(invalid)" }}
        </strong>
      </div>
      <div class="row">
        <input v-model="apiKeyInput" type="password" placeholder="Paste API key" />
        <button class="btn primary" @click="saveApiKey">Save key</button>
        <button class="btn danger" :disabled="!settings.auth.has_credentials" @click="clearAuth">Clear</button>
      </div>
      <p class="hint">
        The key is sent as <code>Authorization: Key &lt;key&gt;</code> on every API request, mirroring NClientV3's
        <code>ApiAuthInterceptor</code>.
      </p>
    </section>

    <section>
      <div class="section-title">Browsing</div>
      <div class="fields">
        <div class="field">
          <label>Default sort</label>
          <select v-model="draft.sort_type">
            <option v-for="s in sorts" :key="s.value" :value="s.value">{{ s.label }}</option>
          </select>
        </div>
        <div class="field">
          <label>Default language filter</label>
          <select v-model="draft.only_language">
            <option v-for="l in langs" :key="l.value" :value="l.value">{{ l.label }}</option>
          </select>
        </div>
        <div class="field">
          <label>Title preference</label>
          <select v-model="draft.title_type">
            <option v-for="t in titleTypes" :key="t.value" :value="t.value">{{ t.label }}</option>
          </select>
        </div>
      </div>
      <div class="checkboxes">
        <label><input type="checkbox" v-model="draft.show_titles" /> Show titles</label>
        <label><input type="checkbox" v-model="draft.exact_tag_match" /> Exact tag match</label>
        <label><input type="checkbox" v-model="draft.remove_avoided_galleries" /> Hide avoided</label>
        <label><input type="checkbox" v-model="draft.use_rtl" /> RTL reader</label>
        <label><input type="checkbox" v-model="draft.keep_history" /> Keep history</label>
      </div>
    </section>

    <section>
      <div class="section-title">Display</div>
      <div class="fields">
        <div class="field">
          <label>Grid columns (approx)</label>
          <input v-model.number="draft.column_count" type="number" min="2" max="10" />
        </div>
        <div class="field">
          <label>Default zoom (%)</label>
          <input v-model.number="draft.default_zoom_pct" type="number" min="20" max="300" />
        </div>
      </div>
      <div class="checkboxes">
        <label><input type="checkbox" v-model="draft.button_change_page" /> Page-change buttons</label>
      </div>
    </section>

    <section>
      <div class="section-title">Downloads</div>
      <div class="field path-field">
        <label>Download directory</label>
        <div class="row">
          <input v-model="draft.download_dir" type="text" readonly />
          <button class="btn" @click="pickDownloadDir">Browse…</button>
        </div>
      </div>
      <div class="fields">
        <div class="field">
          <label>Parallel galleries</label>
          <input v-model.number="draft.parallel_downloads" type="number" min="1" max="10" />
        </div>
        <div class="field">
          <label>Parallel pages per gallery</label>
          <input v-model.number="draft.parallel_pages" type="number" min="1" max="32" />
        </div>
      </div>
    </section>

    <section v-if="appData">
      <div class="section-title">Data</div>
      <p class="hint">App data directory: <code>{{ appData }}</code></p>
    </section>
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
</style>
