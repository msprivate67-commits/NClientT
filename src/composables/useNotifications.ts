/**
 * Thin wrapper around `@tauri-apps/plugin-notification` for surfacing download
 * progress and completion (primarily on Android; best-effort on desktop).
 *
 * On Android we ask for `POST_NOTIFICATIONS` once (the OS only prompts the
 * first time). The plugin is absent from some targets, so every call is
 * guarded — if the API isn't reachable we silently no-op.
 *
 * Progress notifications use the gallery id as the notification id, so each
 * active download owns a single updatable notification rather than spawning a
 * new one per progress tick.
 */
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
  removeActive,
} from "@tauri-apps/plugin-notification";
import type { DownloadProgress } from "@/types";
import { getLocale, LOCALE_MESSAGES, type AppLanguage } from "@/i18n";

function tfn(key: string, params?: Record<string, unknown>): string {
  const locale = getLocale() as AppLanguage;
  const msgs = LOCALE_MESSAGES[locale] || LOCALE_MESSAGES.en;
  const parts = key.split(".");
  let val: unknown = msgs;
  for (const p of parts) {
    val = (val as Record<string, unknown>)?.[p];
  }
  if (typeof val !== "string") return key;
  if (params) {
    return val.replace(/\{(\w+)\}/g, (_, k) => String(params[k] ?? `{${k}}`));
  }
  return val;
}

let permissionAsked = false;

/** Ensure the notification permission has been requested at least once. */
export async function ensureNotificationPermission(): Promise<boolean> {
  try {
    let granted = await isPermissionGranted();
    if (!granted && !permissionAsked) {
      const perm = await requestPermission();
      granted = perm === "granted";
      permissionAsked = true;
    }
    return granted;
  } catch {
    return false;
  }
}

/** Post (or update) a notification; no-op if the plugin is unavailable. */
function notify(
  id: number,
  title: string,
  body: string,
  opts: { ongoing?: boolean; autoCancel?: boolean } = {},
) {
  try {
    sendNotification({
      id,
      title,
      body,
      ongoing: opts.ongoing,
      autoCancel: opts.autoCancel,
    });
  } catch {
    // Plugin unavailable / blocked — ignore.
  }
}

/** Dismiss an active notification by id; no-op if unavailable. */
function dismiss(id: number) {
  try {
    removeActive([{ id }]);
  } catch {
    // ignore
  }
}

function progressBody(p: DownloadProgress): string {
  const pct =
    p.total_pages > 0 ? Math.round((p.done_pages / p.total_pages) * 100) : 0;
  return `${p.done_pages}/${p.total_pages} pages (${pct}%)`;
}

/// Tracks which downloads we've already announced as finished so a resumed /
/// re-emitted finished event doesn't re-notify.
const finishedAnnounced = new Set<number>();
/// Tracks the last percent we posted for each active download, so we throttle
/// progress notifications to at most one ~5% change (avoids notification spam
/// on fast downloads).
const lastPostedPct = new Map<number, number>();

/**
 * React to a `download:progress` payload: post/refresh a progress notification
 * while a download is active, and post a completion notification when it
 * finishes. Paused / canceled / failed downloads dismiss the notification.
 *
 * @param p the progress event payload
 */
export async function handleDownloadNotification(p: DownloadProgress) {
  if (p.status === "finished") {
    if (finishedAnnounced.has(p.id)) return;
    finishedAnnounced.add(p.id);
    lastPostedPct.delete(p.id);
    // Dismiss the ongoing progress notification, then post a tappable
    // completion notification.
    dismiss(p.id);
    const ok = await ensureNotificationPermission();
    if (ok) {
      notify(
        p.id,
        tfn("notification.download_complete"),
        `${p.title}\n${tfn("notification.pages_saved", { n: p.total_pages })}`,
        { autoCancel: true },
      );
    }
    return;
  }

  if (p.status === "downloading" || p.status === "pending") {
    finishedAnnounced.delete(p.id);
  }

  // Terminal-but-not-finished states: cancel the notification and stop.
  if (p.status === "paused" || p.status === "canceled" || p.status === "failed") {
    lastPostedPct.delete(p.id);
    dismiss(p.id);
    return;
  }

  // Progress notifications. Throttle to ~5% steps so a fast download doesn't
  // flood the shade with updates.
  const pct =
    p.total_pages > 0 ? Math.round((p.done_pages / p.total_pages) * 100) : 0;
  const last = lastPostedPct.get(p.id) ?? -100;
  if (pct - last < 5) return;
  lastPostedPct.set(p.id, pct);

  const ok = await ensureNotificationPermission();
  if (!ok) return;
  // `ongoing` keeps the notification pinned while the download runs and lets
  // Android dismiss it automatically once we cancel it on completion.
  notify(p.id, tfn("notification.downloading", { title: p.title }), progressBody(p), { ongoing: true });
}
