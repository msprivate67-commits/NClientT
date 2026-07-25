import type { ThemePreference } from "@/types";

export function applyTheme(theme: ThemePreference): void {
  const root = document.documentElement;
  if (theme === "system") {
    root.removeAttribute("data-theme");
    return;
  }
  root.dataset.theme = theme;
}
