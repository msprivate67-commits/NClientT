import { reactive, ref } from "vue";

interface Position {
  x: number;
  y: number;
}

interface DraggablePositionOptions {
  storageKey: string;
  rightMargin: number;
  bottomMargin: number;
  minimumMaxX?: number;
  minimumMaxY?: number;
}

export function useDraggablePosition(options: DraggablePositionOptions) {
  const position = ref<Position | null>(loadPosition(options.storageKey));
  const drag = reactive({
    active: false,
    pointerId: -1,
    startX: 0,
    startY: 0,
    originX: 0,
    originY: 0,
  });

  function clamp(x: number, y: number): Position {
    const maxX = Math.max(options.minimumMaxX ?? 0, window.innerWidth - options.rightMargin);
    const maxY = Math.max(options.minimumMaxY ?? 0, window.innerHeight - options.bottomMargin);
    return { x: Math.max(0, Math.min(x, maxX)), y: Math.max(0, Math.min(y, maxY)) };
  }

  function style() {
    const current = position.value;
    return current
      ? { left: `${current.x}px`, top: `${current.y}px`, bottom: "auto" }
      : {};
  }

  function onPointerDown(event: PointerEvent) {
    if (event.button !== 0) return;
    const element = event.currentTarget as HTMLElement;
    const rect = element.getBoundingClientRect();
    const origin = position.value ?? { x: rect.left, y: rect.top };
    position.value = origin;
    Object.assign(drag, {
      active: true,
      pointerId: event.pointerId,
      startX: event.clientX,
      startY: event.clientY,
      originX: origin.x,
      originY: origin.y,
    });
    element.setPointerCapture?.(event.pointerId);
    event.preventDefault();
  }

  function onPointerMove(event: PointerEvent) {
    if (!drag.active) return;
    position.value = clamp(
      drag.originX + event.clientX - drag.startX,
      drag.originY + event.clientY - drag.startY,
    );
  }

  function onPointerUp(event: PointerEvent) {
    if (!drag.active) return;
    drag.active = false;
    (event.currentTarget as HTMLElement).releasePointerCapture?.(drag.pointerId);
    drag.pointerId = -1;
    if (!position.value) return;
    position.value = clamp(position.value.x, position.value.y);
    try {
      localStorage.setItem(options.storageKey, JSON.stringify(position.value));
    } catch {
      // Persistence is optional when local storage is unavailable.
    }
  }

  return { position, drag, style, onPointerDown, onPointerMove, onPointerUp };
}

function loadPosition(storageKey: string): Position | null {
  try {
    const raw = localStorage.getItem(storageKey);
    if (!raw) return null;
    const value: unknown = JSON.parse(raw);
    if (
      value &&
      typeof value === "object" &&
      typeof (value as Position).x === "number" &&
      typeof (value as Position).y === "number"
    ) {
      return value as Position;
    }
  } catch {
    // Ignore corrupt or inaccessible storage entries.
  }
  return null;
}
