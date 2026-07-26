import { shallowRef } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface CachedImage {
  objectUrl: string | null;
  promise: Promise<string>;
  bytes: number;
  lastUsed: number;
  pinCount: number;
}

const MAX_CACHE_ENTRIES = 256;
const MAX_CACHE_BYTES = 192 * 1024 * 1024;

const entries = new Map<string, CachedImage>();
const pendingPins = new Map<string, number>();
let cachedBytes = 0;
let accessCounter = 0;

/**
 * Changes whenever an object URL becomes available or is evicted. Consumers
 * read this ref in template source helpers so Vue redraws images without each
 * view maintaining a second, disconnected cache.
 */
export const imageObjectCacheVersion = shallowRef(0);

function notifyChanged() {
  imageObjectCacheVersion.value++;
}

function evictIfNeeded(protectedSource: string) {
  while (entries.size > MAX_CACHE_ENTRIES || cachedBytes > MAX_CACHE_BYTES) {
    let oldestSource: string | null = null;
    let oldestAccess = Number.POSITIVE_INFINITY;
    for (const [source, entry] of entries) {
      if (source === protectedSource || !entry.objectUrl || entry.pinCount > 0) continue;
      if (entry.lastUsed < oldestAccess) {
        oldestAccess = entry.lastUsed;
        oldestSource = source;
      }
    }
    if (!oldestSource) break;
    const oldest = entries.get(oldestSource);
    if (!oldest?.objectUrl) break;
    entries.delete(oldestSource);
    cachedBytes = Math.max(0, cachedBytes - oldest.bytes);
    URL.revokeObjectURL(oldest.objectUrl);
  }
}

export function cachedImageObjectUrl(source: string | null | undefined): string {
  // Establish the reactive dependency even when this source has not finished
  // loading yet.
  void imageObjectCacheVersion.value;
  if (!source) return "";
  const entry = entries.get(source);
  if (!entry?.objectUrl) return "";
  entry.lastUsed = ++accessCounter;
  return entry.objectUrl;
}

/**
 * Keep one source resident even if the normal LRU byte limit is exceeded.
 * Gallery detail pins its pages while it is open so a completed background
 * download cannot be evicted just before the user enters the reader.
 */
export function pinImageObjectSource(source: string | null | undefined): () => void {
  if (!source) return () => {};
  const entry = entries.get(source);
  if (entry) {
    entry.pinCount++;
  } else {
    pendingPins.set(source, (pendingPins.get(source) ?? 0) + 1);
  }

  let released = false;
  return () => {
    if (released) return;
    released = true;
    const current = entries.get(source);
    if (current) {
      current.pinCount = Math.max(0, current.pinCount - 1);
      return;
    }
    const count = pendingPins.get(source) ?? 0;
    if (count <= 1) pendingPins.delete(source);
    else pendingPins.set(source, count - 1);
  };
}

/**
 * Fetch an image through the Rust image protocol exactly once and expose the
 * returned bytes as a stable blob URL. Android WebView can discard a decoded
 * custom-protocol image between two <img> elements; the blob URL avoids a
 * second protocol/network request when moving from detail to reader.
 */
export function loadImageObjectUrl(source: string | null | undefined): Promise<string> {
  if (!source) return Promise.resolve("");
  const existing = entries.get(source);
  if (existing) {
    existing.lastUsed = ++accessCounter;
    return existing.promise;
  }

  const entry: CachedImage = {
    objectUrl: null,
    promise: Promise.resolve(""),
    bytes: 0,
    lastUsed: ++accessCounter,
    pinCount: pendingPins.get(source) ?? 0,
  };
  pendingPins.delete(source);
  const fetchBlob = () => invoke<ArrayBuffer | Uint8Array>("image_fetch", { source })
    .then((response) => {
      const bytes = response instanceof ArrayBuffer
        ? response
        : Uint8Array.from(response).buffer;
      return new Blob([bytes], { type: imageMimeType(source) });
    });
  entry.promise = fetchBlob()
    .catch(async () => {
      // A just-started Android network stack occasionally fails its first
      // custom-protocol request. Keep the retry local to this shared promise
      // so all waiters still result in at most one second request.
      await new Promise((resolve) => setTimeout(resolve, 250));
      return fetchBlob();
    })
    .then((blob) => {
      entry.objectUrl = URL.createObjectURL(blob);
      entry.bytes = blob.size;
      entry.lastUsed = ++accessCounter;
      cachedBytes += blob.size;
      evictIfNeeded(source);
      notifyChanged();
      return entry.objectUrl;
    })
    .catch((error: unknown) => {
      if (entry.pinCount > 0) pendingPins.set(source, entry.pinCount);
      entries.delete(source);
      notifyChanged();
      throw error;
    });
  entries.set(source, entry);
  return entry.promise;
}

function imageMimeType(source: string): string {
  const pathname = source.split(/[?#]/, 1)[0].toLowerCase();
  if (pathname.endsWith(".png")) return "image/png";
  if (pathname.endsWith(".webp")) return "image/webp";
  if (pathname.endsWith(".gif")) return "image/gif";
  if (pathname.endsWith(".avif")) return "image/avif";
  return "image/jpeg";
}
