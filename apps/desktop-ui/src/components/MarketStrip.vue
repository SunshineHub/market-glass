<script setup lang="ts">
import type { IndexQuote } from "@/types/contracts";
import PercentValue from "@/components/PercentValue.vue";

withDefaults(defineProps<{ indices: IndexQuote[]; compact?: boolean }>(), { compact: false });
</script>

<template>
  <section
    class="market-strip"
    :class="{ compact }"
    :style="{ '--market-count': Math.max(indices.length, 1) }"
    aria-label="大盘指数"
  >
    <article v-for="index in indices" :key="index.code" class="market-item material-card">
      <div class="market-copy">
        <span class="market-name">{{ index.name }}</span>
        <strong class="mono-numbers">{{ index.value.toFixed(2) }}</strong>
      </div>
      <div class="market-move">
        <PercentValue :value="index.changePercent" />
        <small>涨跌幅</small>
      </div>
    </article>
  </section>
</template>

<style scoped>
.market-strip {
  display: grid;
  grid-template-columns: repeat(var(--market-count), minmax(0, 1fr));
  gap: 10px;
}

.market-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-width: 0;
  padding: 13px 14px;
  background: var(--glass-subtle);
  border: 1px solid var(--hairline);
  border-radius: var(--radius-md);
  transition: padding 180ms ease, border-radius 180ms ease, background-color 180ms ease;
}

.market-copy,
.market-move {
  display: flex;
  flex-direction: column;
}

.market-copy {
  gap: 5px;
  min-width: 0;
}

.market-name {
  overflow: hidden;
  font-size: var(--font-xs);
  font-weight: 560;
  color: var(--text-muted);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.market-copy strong {
  font-size: var(--font-md);
  font-weight: 680;
  color: var(--text-strong);
}

.market-move {
  gap: 4px;
  align-items: flex-end;
  font-size: var(--font-sm);
}

.market-move small {
  font-size: var(--font-xs);
  color: var(--text-muted);
}

.compact {
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}

.compact .market-item {
  padding: 9px 10px;
  border-radius: 12px;
}

.compact .market-copy strong {
  font-size: var(--font-md);
}

.compact .market-move small {
  display: none;
}
</style>
