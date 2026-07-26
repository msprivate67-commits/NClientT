import { type ComputedRef, type Ref, onUnmounted, ref } from "vue";

export type ReaderDirection = "vertical" | "horizontal";
export type ReaderFitMode = "width" | "height" | "original";

interface ReaderNavigationOptions {
  rtl?: ComputedRef<boolean>;
  onPageSettled?: () => void;
}

export function useReaderNavigation(
  total: ComputedRef<number>,
  direction: Ref<ReaderDirection>,
  options: ReaderNavigationOptions = {},
) {
  const scrollRef = ref<HTMLElement | null>(null);
  const currentPage = ref(1);
  let scrollTimer: ReturnType<typeof setTimeout> | null = null;

  function computeCurrentPage() {
    if (!scrollRef.value || !total.value) return;
    const container = scrollRef.value;
    const horizontal = direction.value === "horizontal";
    const viewCenter = horizontal
      ? container.scrollLeft + container.clientWidth / 2
      : container.scrollTop + container.clientHeight / 2;

    let closestIndex = 0;
    let closestDistance = Infinity;
    const wraps = container.querySelectorAll<HTMLElement>(".page-wrap");
    for (let index = 0; index < wraps.length; index++) {
      const element = wraps[index];
      const position = horizontal ? element.offsetLeft : element.offsetTop;
      const size = horizontal ? element.offsetWidth : element.offsetHeight;
      const distance = Math.abs(viewCenter - (position + size / 2));
      if (distance < closestDistance) {
        closestDistance = distance;
        closestIndex = index;
      }
    }
    currentPage.value = closestIndex + 1;
  }

  function onScroll() {
    if (scrollTimer) clearTimeout(scrollTimer);
    scrollTimer = setTimeout(() => {
      computeCurrentPage();
      options.onPageSettled?.();
    }, 150);
  }

  function scrollToPage(index: number, smooth = true) {
    if (!scrollRef.value || index < 0 || index >= total.value) return;
    const element = scrollRef.value.querySelectorAll<HTMLElement>(".page-wrap")[index];
    if (!element) return;
    const behavior = smooth ? "smooth" : "auto";
    if (direction.value === "horizontal") {
      scrollRef.value.scrollTo({ left: element.offsetLeft, behavior });
    } else {
      scrollRef.value.scrollTo({ top: element.offsetTop, behavior });
    }
  }

  function goPage(delta: number) {
    const nextIndex = currentPage.value - 1 + delta;
    if (nextIndex < 0 || nextIndex >= total.value) return;
    scrollToPage(nextIndex);
  }

  function previous() {
    goPage(options.rtl?.value ? 1 : -1);
  }

  function next() {
    goPage(options.rtl?.value ? -1 : 1);
  }

  function reset() {
    currentPage.value = 1;
    if (!scrollRef.value) return;
    scrollRef.value.scrollTop = 0;
    scrollRef.value.scrollLeft = 0;
  }

  onUnmounted(() => {
    if (scrollTimer) clearTimeout(scrollTimer);
  });

  return {
    scrollRef,
    currentPage,
    computeCurrentPage,
    onScroll,
    scrollToPage,
    previous,
    next,
    reset,
  };
}
