<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(defineProps<{ value: number; digits?: number }>(), { digits: 2 });
const tone = computed(() => (props.value > 0 ? "profit" : props.value < 0 ? "loss" : "flat"));
const text = computed(() => `${props.value > 0 ? "+" : ""}${props.value.toFixed(props.digits)}%`);
</script>

<template>
  <span class="mono-numbers" :class="tone">{{ text }}</span>
</template>
