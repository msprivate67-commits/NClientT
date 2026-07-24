import { onUnmounted } from "vue";

export function preloadImage(url: string): Promise<void> {
  if (!url) return Promise.resolve();
  return new Promise((resolve) => {
    const image = new Image();
    image.decoding = "async";
    image.onload = () => resolve();
    image.onerror = () => resolve();
    image.src = url;
  });
}

/**
 * Runs a small, continuous preload queue. Normal work proceeds in document
 * order, while newly visible indices can be moved to the front immediately.
 */
export function usePriorityPreloadQueue(
  task: (index: number) => Promise<void>,
  concurrency = 2,
) {
  let generation = 0;
  let active = 0;
  let pending: number[] = [];
  const queued = new Set<number>();
  const running = new Set<number>();
  const completed = new Set<number>();

  function pump() {
    const currentGeneration = generation;
    while (active < concurrency && pending.length > 0) {
      const index = pending.shift()!;
      queued.delete(index);
      running.add(index);
      active++;
      void task(index).finally(() => {
        if (generation !== currentGeneration) return;
        running.delete(index);
        active--;
        completed.add(index);
        pump();
      });
    }
  }

  function enqueue(indices: Iterable<number>, priority = false) {
    const additions: number[] = [];
    for (const index of indices) {
      if (
        !Number.isInteger(index) ||
        index < 0 ||
        completed.has(index) ||
        running.has(index)
      ) {
        continue;
      }
      if (queued.has(index)) {
        if (!priority) continue;
        pending = pending.filter((value) => value !== index);
      } else {
        queued.add(index);
      }
      additions.push(index);
    }
    if (priority) pending.unshift(...additions);
    else pending.push(...additions);
    pump();
  }

  function reset() {
    generation++;
    active = 0;
    pending = [];
    queued.clear();
    running.clear();
    completed.clear();
  }

  onUnmounted(reset);
  return { enqueue, reset };
}
