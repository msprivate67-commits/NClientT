import { onUnmounted, ref, watch, type Ref } from "vue";

const callbacks = new WeakMap<Element, () => void>();
let sharedObserver: IntersectionObserver | null = null;

function observer(): IntersectionObserver | null {
  if (typeof IntersectionObserver === "undefined") return null;
  if (!sharedObserver) {
    sharedObserver = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (!entry.isIntersecting) continue;
          callbacks.get(entry.target)?.();
          callbacks.delete(entry.target);
          sharedObserver?.unobserve(entry.target);
        }
      },
      { rootMargin: "500px 0px", threshold: 0.01 },
    );
  }
  return sharedObserver;
}

/**
 * Becomes true once an element enters the viewport preload margin.
 *
 * All callers share one IntersectionObserver, so a large gallery grid does
 * not create one observer per card. Visibility is intentionally sticky: once
 * an image has been requested it remains mounted and can use the WebView cache.
 */
export function useLazyVisible(target: Ref<HTMLElement | null>): Ref<boolean> {
  const visible = ref(false);
  let observed: HTMLElement | null = null;

  const stop = watch(
    target,
    (element) => {
      if (observed) {
        callbacks.delete(observed);
        sharedObserver?.unobserve(observed);
      }
      observed = element;
      if (!element || visible.value) return;

      const instance = observer();
      if (!instance) {
        visible.value = true;
        return;
      }
      callbacks.set(element, () => {
        visible.value = true;
      });
      instance.observe(element);
    },
    { flush: "post" },
  );

  onUnmounted(() => {
    stop();
    if (observed) {
      callbacks.delete(observed);
      sharedObserver?.unobserve(observed);
    }
  });

  return visible;
}
