<script setup lang="ts">
withDefaults(
  defineProps<{
    label: string;
    active?: boolean;
    size?: "small" | "normal";
  }>(),
  { active: false, size: "normal" },
);
</script>

<template>
  <button
    type="button"
    class="icon-button no-drag"
    :class="[{ active }, size]"
    :aria-label="label"
    :title="label"
    data-liquid-glass
  >
    <slot />
  </button>
</template>

<style scoped>
.icon-button {
  display: inline-grid;
  width: 36px;
  height: 36px;
  padding: 0;
  color: var(--text);
  cursor: pointer;
  background: var(--glass-subtle);
  border: 1px solid var(--hairline);
  border-radius: 12px;
  place-items: center;
  transition: 160ms ease;
}

.icon-button:hover,
.icon-button.active {
  color: var(--text-strong);
  background: var(--glass-hover);
  border-color: var(--hairline-strong);
}

.icon-button.small {
  width: 30px;
  height: 30px;
  border-radius: 10px;
}

.icon-button :deep(svg) {
  width: 17px;
  height: 17px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.8;
  stroke-linecap: round;
  stroke-linejoin: round;
  pointer-events: none;
}

.icon-button :deep(svg *) {
  pointer-events: none;
}
</style>
