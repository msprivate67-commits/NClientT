import { reactive, type Ref } from "vue";

interface EdgeSwipeOptions {
  startWidth?: number;
  openDistance?: number;
}

export function useEdgeSwipe(
  isOpen: Ref<boolean>,
  open: () => void,
  options: EdgeSwipeOptions = {},
) {
  const startWidth = options.startWidth ?? 40;
  const openDistance = options.openDistance ?? 50;
  const state = reactive({ tracking: false, startX: 0, startY: 0 });

  function begin(clientX: number, clientY: number) {
    if (isOpen.value || clientX >= startWidth) {
      state.tracking = false;
      return;
    }
    state.startX = clientX;
    state.startY = clientY;
    state.tracking = true;
  }

  function move(clientX: number, clientY: number): boolean {
    if (!state.tracking) return false;
    const dx = clientX - state.startX;
    const dy = clientY - state.startY;
    if (dx < 0 && Math.abs(dy) > Math.abs(dx)) {
      state.tracking = false;
      return false;
    }
    if (dx <= openDistance) return false;

    open();
    state.tracking = false;
    return true;
  }

  function end() {
    state.tracking = false;
  }

  function onTouchStart(event: TouchEvent) {
    if (event.touches.length !== 1) {
      end();
      return;
    }
    begin(event.touches[0].clientX, event.touches[0].clientY);
  }

  function onTouchMove(event: TouchEvent) {
    if (event.touches.length !== 1) return;
    if (move(event.touches[0].clientX, event.touches[0].clientY)) event.preventDefault();
  }

  function onMouseDown(event: MouseEvent) {
    if (event.button === 0) begin(event.clientX, event.clientY);
  }

  function onMouseMove(event: MouseEvent) {
    move(event.clientX, event.clientY);
  }

  return {
    state,
    onTouchStart,
    onTouchMove,
    onTouchEnd: end,
    onMouseDown,
    onMouseMove,
    onMouseUp: end,
  };
}
