import { computed, nextTick, reactive, ref } from "vue";

export function useSwipeDismiss(onDismiss: () => void, thresholdRatio = 0.3) {
  const panelRef = ref<HTMLElement | null>(null);
  const drag = reactive({
    active: false,
    startX: 0,
    startY: 0,
    x: 0,
    dismissing: false,
  });

  const panelStyle = computed(() => {
    if (!drag.active && !drag.dismissing) return {};
    return {
      transform: `translateX(${drag.dismissing ? "100%" : `${drag.x}px`})`,
      transition: drag.active ? "none" : "transform 0.28s ease",
    };
  });

  function reset() {
    drag.active = false;
    drag.x = 0;
    drag.dismissing = false;
  }

  function onTouchStart(event: TouchEvent) {
    if (event.touches.length !== 1) return;
    drag.startX = event.touches[0].clientX;
    drag.startY = event.touches[0].clientY;
    reset();
  }

  function onTouchMove(event: TouchEvent) {
    if (event.touches.length !== 1) return;
    const dx = event.touches[0].clientX - drag.startX;
    const dy = event.touches[0].clientY - drag.startY;
    if (!drag.active) {
      if (dx <= 8 || Math.abs(dx) <= Math.abs(dy)) return;
      drag.active = true;
    }
    drag.x = Math.max(0, dx);
    event.preventDefault();
  }

  function onTouchEnd() {
    if (!drag.active) return;
    if (drag.x <= window.innerWidth * thresholdRatio) {
      reset();
      return;
    }

    drag.dismissing = true;
    drag.active = false;
    nextTick(() => {
      const element = panelRef.value;
      if (!element) return;
      const finish = (event: TransitionEvent) => {
        if (event.propertyName !== "transform") return;
        element.removeEventListener("transitionend", finish);
        onDismiss();
        reset();
      };
      element.addEventListener("transitionend", finish);
    });
  }

  return { panelRef, panelStyle, onTouchStart, onTouchMove, onTouchEnd };
}
