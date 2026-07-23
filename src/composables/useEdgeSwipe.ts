import { reactive, type Ref } from "vue";

interface EdgeSwipeOptions {
  /** Width of the content area that may start the gesture. */
  startWidth?: number;
  /** CSS width of the drawer before its viewport max-width is applied. */
  drawerWidth?: number;
  /** Fraction of the drawer that must be pulled before it opens. */
  openThreshold?: number;
  /** Restrict the gesture to contexts such as the compact/mobile layout. */
  enabled?: Ref<boolean>;
}

type SwipeAxis = "pending" | "horizontal";

export function useEdgeSwipe(
  isOpen: Ref<boolean>,
  open: () => void,
  options: EdgeSwipeOptions = {},
) {
  const startWidth = options.startWidth ?? 96;
  const drawerWidth = options.drawerWidth ?? 240;
  const openThreshold = options.openThreshold ?? 0.4;
  const intentDistance = 8;
  const state = reactive({
    tracking: false,
    dragging: false,
    startX: 0,
    startY: 0,
    progress: 0,
    axis: "pending" as SwipeAxis,
  });

  function renderedDrawerWidth(): number {
    const viewportWidth = typeof window === "undefined" ? drawerWidth : window.innerWidth;
    return Math.min(drawerWidth, viewportWidth * 0.8);
  }

  function reset() {
    state.tracking = false;
    state.dragging = false;
    state.progress = 0;
    state.axis = "pending";
  }

  function begin(clientX: number, clientY: number) {
    if (options.enabled?.value === false || isOpen.value || clientX > startWidth) {
      reset();
      return;
    }
    state.startX = clientX;
    state.startY = clientY;
    state.tracking = true;
    state.dragging = false;
    state.progress = 0;
    state.axis = "pending";
  }

  /** Returns true once the gesture owns this movement and scrolling should stop. */
  function move(clientX: number, clientY: number): boolean {
    if (!state.tracking) return false;

    const dx = clientX - state.startX;
    const dy = clientY - state.startY;

    if (state.axis === "pending") {
      if (Math.hypot(dx, dy) < intentDistance) return false;
      if (dx <= 0 || Math.abs(dy) > Math.abs(dx)) {
        reset();
        return false;
      }
      state.axis = "horizontal";
      state.dragging = true;
    }

    state.progress = Math.min(1, Math.max(0, dx / renderedDrawerWidth()));
    if (state.progress >= openThreshold) {
      reset();
      open();
    }
    return true;
  }

  function end() {
    // Crossing the threshold opens immediately in move(); otherwise clearing
    // the drag state lets the drawer transition back off-screen.
    reset();
  }

  function onTouchStart(event: TouchEvent) {
    if (event.touches.length !== 1) {
      reset();
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
    onTouchCancel: end,
    onMouseDown,
    onMouseMove,
    onMouseUp: end,
  };
}
