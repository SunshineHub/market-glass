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
  >
    <span class="icon-layer" aria-hidden="true">
      <slot />
    </span>
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
  contain: paint;
  place-items: center;
  transform: none;
  transition:
    color 150ms ease,
    background-color 150ms ease,
    border-color 150ms ease,
    box-shadow 180ms ease;
}

.icon-button::before,
.icon-button::after {
  display: none;
}

.icon-button:hover,
.icon-button.active {
  color: var(--text-strong);
  background: var(--glass-hover);
  border-color: var(--hairline-strong);
  box-shadow: 0 5px 14px color-mix(in srgb, var(--text-strong) 6%, transparent), 0 1px 0 var(--material-highlight) inset;
}

.icon-button:active {
  transform: none;
}

.icon-button.small {
  width: 30px;
  height: 30px;
  border-radius: 10px;
}

.icon-layer {
  position: relative;
  z-index: 1;
  display: grid;
  width: 100%;
  height: 100%;
  pointer-events: none;
  place-items: center;
}

.icon-layer :deep(svg) {
  width: 17px;
  height: 17px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.8;
  stroke-linecap: round;
  stroke-linejoin: round;
  pointer-events: none;
}

.icon-layer :deep(svg *) {
  pointer-events: none;
}
</style>
