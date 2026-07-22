import { createI18n } from "vue-i18n";
import { locale as osLocale } from "@tauri-apps/plugin-os";

import en from "./locales/en.json";
import zh from "./locales/zh.json";
import ja from "./locales/ja.json";
import ru from "./locales/ru.json";

export type AppLanguage = "en" | "zh" | "ja" | "ru";

export const SUPPORTED_LANGUAGES: { code: AppLanguage; name: string; nativeName: string }[] = [
  { code: "en", name: "English", nativeName: "English" },
  { code: "zh", name: "Chinese", nativeName: "中文" },
  { code: "ja", name: "Japanese", nativeName: "日本語" },
  { code: "ru", name: "Russian", nativeName: "Русский" },
];

export const LOCALE_MESSAGES: Record<AppLanguage, Record<string, unknown>> = {
  en: en as Record<string, unknown>,
  zh: zh as Record<string, unknown>,
  ja: ja as Record<string, unknown>,
  ru: ru as Record<string, unknown>,
};

const EN_MESSAGES = en as Record<string, unknown>;

let _currentLocale = "en";

function collectKeys(obj: Record<string, unknown>, prefix = ""): string[] {
  const keys: string[] = [];
  for (const [k, v] of Object.entries(obj)) {
    const path = prefix ? `${prefix}.${k}` : k;
    if (v != null && typeof v === "object" && !Array.isArray(v)) {
      keys.push(...collectKeys(v as Record<string, unknown>, path));
    } else {
      keys.push(path);
    }
  }
  return keys;
}

function deepMerge(target: Record<string, unknown>, source: Record<string, unknown>): Record<string, unknown> {
  const result = { ...target };
  for (const [k, v] of Object.entries(source)) {
    if (v != null && typeof v === "object" && !Array.isArray(v) && k in result && result[k] != null && typeof result[k] === "object") {
      result[k] = deepMerge(result[k] as Record<string, unknown>, v as Record<string, unknown>);
    } else {
      result[k] = v;
    }
  }
  return result;
}

export function createI18nInstance() {
  return createI18n({
    legacy: false,
    locale: _currentLocale,
    fallbackLocale: "en",
    messages: LOCALE_MESSAGES as any,
    missing: (_locale: string, key: string) => {
      console.warn(`[i18n] Missing translation: ${key}`);
    },
  });
}

export function setLocale(locale: string) {
  _currentLocale = locale;
}

export function getLocale(): string {
  return _currentLocale;
}

export function getLocaleMessages(locale: string): Record<string, unknown> {
  const msgs = LOCALE_MESSAGES[locale as AppLanguage];
  return msgs ? JSON.parse(JSON.stringify(msgs)) : {};
}

export function exportLocaleJson(locale: string): string {
  const msgs = getLocaleMessages(locale);
  return JSON.stringify(msgs, null, 2);
}

export function importLocaleJson(
  jsonStr: string,
): { messages: Record<string, unknown>; missingKeys: string[] } {
  const enKeys = collectKeys(EN_MESSAGES);
  let imported: Record<string, unknown>;
  try {
    imported = JSON.parse(jsonStr);
  } catch (e) {
    throw new Error(`Invalid JSON: ${String(e)}`);
  }
  const merged = deepMerge(EN_MESSAGES, imported);
  const importedKeys = collectKeys(imported);
  const missingKeys = enKeys.filter((k) => !importedKeys.includes(k));
  return { messages: merged, missingKeys };
}

export function applyImportedMessages(locale: string, jsonStr: string): { missingKeys: string[] } {
  const { messages, missingKeys } = importLocaleJson(jsonStr);
  LOCALE_MESSAGES[locale as AppLanguage] = messages;
  return { missingKeys };
}

export async function detectPlatformLanguage(): Promise<AppLanguage> {
  try {
    const raw = await osLocale();
    if (!raw) return "en";
    const lang = raw.split("-")[0].toLowerCase();
    switch (lang) {
      case "zh":
      case "zh-cn":
      case "zh-tw":
      case "zh-hk":
        return "zh";
      case "ja":
      case "jp":
        return "ja";
      case "ru":
        return "ru";
      default:
        return "en";
    }
  } catch {
    try {
      const browserLang = navigator.language?.split("-")[0].toLowerCase();
      switch (browserLang) {
        case "zh":
          return "zh";
        case "ja":
          return "ja";
        case "ru":
          return "ru";
        default:
          return "en";
      }
    } catch {
      return "en";
    }
  }
}

export function isValidLanguage(lang: string): lang is AppLanguage {
  return ["en", "zh", "ja", "ru"].includes(lang);
}
