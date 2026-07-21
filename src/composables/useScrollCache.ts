import { onActivated, onDeactivated, ref, type Ref, nextTick } from "vue";

export function useScrollCache(elRef: Ref<HTMLElement | null>) {
  const savedScrollTop = ref(0);

  onDeactivated(() => {
    if (elRef.value) {
      savedScrollTop.value = elRef.value.scrollTop;
    }
  });

  onActivated(() => {
    if (elRef.value && savedScrollTop.value > 0) {
      nextTick(() => {
        if (elRef.value) {
          elRef.value.scrollTop = savedScrollTop.value;
        }
      });
    }
  });
}
