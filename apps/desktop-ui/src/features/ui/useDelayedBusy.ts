import { onBeforeUnmount, ref, watch, type Ref } from "vue";

export function useDelayedBusy(
  source: () => boolean,
  delay = 180,
): Ref<boolean> {
  const visible = ref(false);
  let timer: number | undefined;

  function clearTimer() {
    if (timer !== undefined) window.clearTimeout(timer);
    timer = undefined;
  }

  watch(
    source,
    (busy) => {
      clearTimer();
      if (!busy) {
        visible.value = false;
        return;
      }
      timer = window.setTimeout(() => {
        visible.value = true;
        timer = undefined;
      }, delay);
    },
    { immediate: true, flush: "sync" },
  );

  onBeforeUnmount(clearTimer);
  return visible;
}
