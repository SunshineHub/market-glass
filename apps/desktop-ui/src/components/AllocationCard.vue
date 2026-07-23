<script setup lang="ts">
import { computed } from "vue";
import type { AllocationSlice } from "@/types/contracts";

const props = defineProps<{ slices: AllocationSlice[] }>();
const normalizedSlices = computed(() => {
  const valid = props.slices.filter((slice) => Number.isFinite(slice.value) && slice.value > 0);
  const total = valid.reduce((sum, slice) => sum + slice.value, 0);
  if (total <= 0) return [];
  return valid.map((slice) => ({ ...slice, value: (slice.value / total) * 100 }));
});
const gradient = computed(() => {
  if (!normalizedSlices.value.length) return "conic-gradient(var(--hairline) 0 100%)";
  let cursor = 0;
  const stops = normalizedSlices.value.map((slice) => {
    const start = cursor;
    cursor += slice.value;
    return `${slice.color} ${start}% ${cursor}%`;
  });
  return `conic-gradient(${stops.join(", ")})`;
});
const leadingSlice = computed(() => normalizedSlices.value[0]);
const leadingValue = computed(() => leadingSlice.value?.value.toFixed(1) ?? "0.0");
</script>

<template>
  <section class="allocation-card material-card">
    <header>
      <div>
        <span class="eyebrow">行业比例</span>
        <h3>持仓行业分布</h3>
      </div>
      <span class="period">当前</span>
    </header>
    <div class="allocation-content">
      <div class="donut" :style="{ background: gradient }">
        <div><strong>{{ leadingValue }}%</strong><span>{{ leadingSlice?.label ?? "暂无" }}</span></div>
      </div>
      <ul>
        <li v-for="slice in normalizedSlices" :key="slice.key">
          <i :style="{ background: slice.color }" />
          <span>{{ slice.label }}</span>
          <strong class="mono-numbers">{{ slice.value.toFixed(1) }}%</strong>
        </li>
      </ul>
    </div>
  </section>
</template>

<style scoped>
.allocation-card {
  height: 100%;
  padding: 18px;
  background: var(--glass-subtle);
  border: 1px solid var(--hairline);
  border-radius: var(--radius-lg);
}

header,
.allocation-content,
li {
  display: flex;
  align-items: center;
}

header {
  justify-content: space-between;
}

.eyebrow,
.period {
  font-size: var(--font-xs);
  font-weight: 650;
  color: var(--text-muted);
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

h3 {
  margin: 4px 0 0;
  font-size: var(--font-md);
  color: var(--text-strong);
}

.period {
  padding: 5px 8px;
  letter-spacing: 0;
  background: var(--glass-subtle);
  border: 1px solid var(--hairline);
  border-radius: 99px;
}

.allocation-content {
  gap: 22px;
  min-height: 0;
  margin-top: 20px;
}

.donut {
  position: relative;
  display: grid;
  flex: 0 0 104px;
  width: 104px;
  height: 104px;
  border-radius: 50%;
  box-shadow: inset 0 0 0 1px var(--hairline);
  place-items: center;
}

.donut::before {
  width: 68px;
  height: 68px;
  content: "";
  background: var(--glass-strong);
  border: 1px solid var(--hairline);
  border-radius: 50%;
  backdrop-filter: blur(18px);
}

.donut > div {
  position: absolute;
  top: 50%;
  left: 50%;
  display: flex;
  flex-direction: column;
  align-items: center;
  transform: translate(-50%, -50%);
}

.donut strong {
  font: 700 var(--font-lg)/1 var(--font-mono);
  color: var(--text-strong);
}

.donut span {
  margin-top: 4px;
  font-size: var(--font-xs);
  color: var(--text-muted);
}

ul {
  flex: 1;
  min-width: 0;
  max-height: 146px;
  padding: 0;
  margin: 0;
  overflow: auto;
  scrollbar-width: thin;
  list-style: none;
}

li {
  display: grid;
  grid-template-columns: 7px minmax(0, 1fr) 48px;
  gap: 8px;
  padding: 5px 0;
  font-size: var(--font-sm);
  color: var(--text-muted);
}

li i {
  width: 7px;
  height: 7px;
  border-radius: 99px;
}

li span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

li strong {
  font-size: var(--font-sm);
  color: var(--text-strong);
  text-align: right;
}
</style>
