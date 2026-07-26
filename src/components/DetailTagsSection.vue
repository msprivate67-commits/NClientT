<script setup lang="ts">
import { computed, nextTick, onUnmounted, ref, watch } from "vue";
import { Ban, ChevronDown, ChevronUp, Languages, Loader, ShieldCheck } from "@lucide/vue";

import { translateTags } from "@/api";
import TagChip from "@/components/TagChip.vue";
import { useSettingsStore } from "@/stores/settings";
import type { Tag } from "@/types";

const props = defineProps<{
  groups: Map<string, Tag[]>;
  expanded: boolean;
  blacklistedIds?: Set<number>;
}>();

defineEmits<{
  "update:expanded": [expanded: boolean];
  select: [tag: Tag];
  toggleBlacklist: [tag: Tag];
}>();

const settingsStore = useSettingsStore();
const translations = ref(new Map<number, string>());
const translating = ref(false);
const translateError = ref("");
const reasoningText = ref("");
const reasoningRef = ref<HTMLElement | null>(null);
let translationController: AbortController | null = null;

const allTags = computed(() => {
  const unique = new Map<number, Tag>();
  props.groups.forEach((tags) => tags.forEach((tag) => unique.set(tag.id, tag)));
  return [...unique.values()];
});
const tagSignature = computed(() => allTags.value.map((tag) => `${tag.id}:${tag.name}`).join("|"));

async function scrollReasoningToBottom() {
  await nextTick();
  if (reasoningRef.value) reasoningRef.value.scrollTop = reasoningRef.value.scrollHeight;
}

async function translateAll() {
  if (translating.value || !allTags.value.length) return;
  translationController?.abort();
  const controller = new AbortController();
  translationController = controller;
  translating.value = true;
  translateError.value = "";
  reasoningText.value = "";
  const settings = settingsStore.settings;
  try {
    translations.value = await translateTags(
      settings.tl_base_url,
      settings.tl_model,
      settings.tl_api_key,
      allTags.value.map(({ id, name }) => ({ id, name })),
      settings.tl_target_lang,
      settings.tl_thinking,
      settings.tl_use_proxy,
      {
        signal: controller.signal,
        onReasoning: (chunk) => {
          if (translationController !== controller || controller.signal.aborted) return;
          reasoningText.value += chunk;
          void scrollReasoningToBottom();
        },
      },
    );
  } catch (error: unknown) {
    if (!controller.signal.aborted) {
      translateError.value = error instanceof Error ? error.message : String(error);
    }
  } finally {
    if (translationController === controller) {
      translationController = null;
      translating.value = false;
    }
  }
}

watch(tagSignature, () => {
  translationController?.abort();
  translationController = null;
  translating.value = false;
  translateError.value = "";
  reasoningText.value = "";
  translations.value = new Map();
});

onUnmounted(() => translationController?.abort());
</script>

<template>
  <section class="detail-card tags-card">
    <div class="section-toggle-bar tag-toggle-bar">
      <div class="section-title">{{ $t('tags.title') }}</div>
      <button
        class="btn small"
        type="button"
        :aria-expanded="expanded"
        @click="$emit('update:expanded', !expanded)"
      >
        <ChevronUp v-if="expanded" :size="14" />
        <ChevronDown v-else :size="14" />
        {{ expanded ? $t('gallery.collapse') : $t('gallery.expand') }}
      </button>
    </div>
    <div v-show="expanded" class="tags-content">
      <div class="translation-controls">
        <button
          class="btn small"
          type="button"
          :disabled="translating || !allTags.length"
          @click="translateAll"
        >
          <Loader v-if="translating" :size="14" class="spin" />
          <Languages v-else :size="14" />
          {{ translating
            ? $t('tags.translating')
            : translations.size
              ? $t('tags.retranslate_all')
              : $t('tags.translate_all') }}
        </button>
      </div>
      <div v-if="reasoningText" class="tag-reasoning">
        <div class="reasoning-label">{{ $t('gallery.reasoning') }}</div>
        <div ref="reasoningRef" class="reasoning-text">{{ reasoningText }}</div>
      </div>
      <div v-if="translateError" class="translate-error">
        {{ $t('tags.translate_error', { message: translateError }) }}
      </div>
      <section v-for="[type, tags] in groups" :key="type" class="tag-group">
        <div class="section-title">{{ type }}</div>
        <div class="chips">
          <span v-for="tag in tags" :key="tag.id" class="tag-with-action">
            <TagChip
              :tag="{ ...tag, blacklisted: blacklistedIds?.has(tag.id) ?? tag.blacklisted }"
              :translated-name="translations.get(tag.id)"
              show-type
              @click="$emit('select', tag)"
            />
            <button
              class="blacklist-action"
              type="button"
              :class="{ active: blacklistedIds?.has(tag.id) ?? tag.blacklisted }"
              :title="(blacklistedIds?.has(tag.id) ?? tag.blacklisted)
                ? $t('tags.remove_blacklist')
                : $t('tags.add_blacklist')"
              @click="$emit('toggleBlacklist', tag)"
            >
              <ShieldCheck v-if="blacklistedIds?.has(tag.id) ?? tag.blacklisted" :size="12" />
              <Ban v-else :size="12" />
            </button>
          </span>
        </div>
      </section>
    </div>
  </section>
</template>

<style scoped>
.section-toggle-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.section-toggle-bar .section-title {
  margin: 0;
  margin-right: auto;
}
.tags-content {
  margin-top: 12px;
}
.translation-controls {
  display: flex;
  justify-content: flex-end;
  margin-bottom: 10px;
}
.tag-reasoning {
  margin-bottom: 12px;
}
.reasoning-label {
  margin-bottom: 4px;
  color: var(--text-dim);
  font-size: 0.75rem;
}
.reasoning-text {
  max-height: calc(5 * 1.45em + 12px);
  padding: 6px 8px;
  overflow-y: auto;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.04);
  color: var(--text-dim);
  font-size: 0.75rem;
  font-style: italic;
  line-height: 1.45;
  overflow-wrap: anywhere;
  text-align: left;
  white-space: pre-wrap;
}
.translate-error {
  margin-bottom: 10px;
  color: #f08080;
  font-size: 0.78rem;
}
.spin {
  animation: spin 1s linear infinite;
}
@keyframes spin {
  to { transform: rotate(360deg); }
}
.tag-group {
  margin-bottom: 14px;
}
.tag-group:last-child {
  margin-bottom: 0;
}
.tag-with-action {
  display: inline-flex;
  align-items: center;
}
.blacklist-action {
  display: inline-grid;
  place-items: center;
  width: 22px;
  height: 22px;
  margin-left: -4px;
  border: 1px solid var(--border);
  border-radius: 50%;
  background: var(--surface);
  color: var(--text-dim);
  cursor: pointer;
}
.blacklist-action:hover,
.blacklist-action.active {
  border-color: #e05252;
  color: #ff8e8e;
}
</style>
