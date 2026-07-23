<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(
  defineProps<{ values: number[]; positive?: boolean; width?: number; height?: number }>(),
  { positive: true, width: 96, height: 30 },
);

const points = computed(() => {
  if (props.values.length < 2) return "";
  const min = Math.min(...props.values);
  const max = Math.max(...props.values);
  const range = max - min || 1;
  return props.values
    .map((value, index) => {
      const x = (index / (props.values.length - 1)) * props.width;
      const y = props.height - 2 - ((value - min) / range) * (props.height - 4);
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
});
</script>

<template>
  <svg class="sparkline" :viewBox="`0 0 ${width} ${height}`" aria-hidden="true">
    <polyline :points="points" :class="positive ? 'spark-profit' : 'spark-loss'" />
  </svg>
</template>

<style scoped>
.sparkline {
  display: block;
  width: 100%;
  height: 100%;
  overflow: visible;
}

polyline {
  fill: none;
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
  vector-effect: non-scaling-stroke;
}

.spark-profit {
  stroke: var(--profit);
}

.spark-loss {
  stroke: var(--loss);
}
</style>
