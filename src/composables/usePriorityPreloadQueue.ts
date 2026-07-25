import { onUnmounted } from "vue";

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
      void task(index)
        .then(() => {
          if (generation === currentGeneration) completed.add(index);
        })
        .catch(() => {
          // Loading is best-effort. A failed index deliberately remains out of
          // `completed` so a later visibility/current-page priority can retry
          // it without producing an unhandled promise rejection.
        })
        .finally(() => {
          if (generation !== currentGeneration) return;
          running.delete(index);
          active--;
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
