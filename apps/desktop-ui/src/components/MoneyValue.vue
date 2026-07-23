<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    value: number;
    private?: boolean;
    sign?: boolean;
    compact?: boolean;
  }>(),
  { private: false, sign: false, compact: false },
);

const formatted = computed(() => {
  if (props.private) return "••••••";
  const abs = Math.abs(props.value);
  const prefix = props.sign && props.value > 0 ? "+" : props.value < 0 ? "-" : "";
  if (props.compact && abs >= 10_000) {
    return `${prefix}¥${(abs / 10_000).toFixed(abs >= 100_000 ? 1 : 2)}万`;
  }
  return `${prefix}¥${abs.toLocaleString("zh-CN", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  })}`;
});
</script>

<template>
  <span class="mono-numbers" :aria-label="private ? '金额已隐藏' : formatted">{{ formatted }}</span>
</template>
