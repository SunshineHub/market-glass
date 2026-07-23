<script setup lang="ts">
import MoneyValue from "@/components/MoneyValue.vue";
import PercentValue from "@/components/PercentValue.vue";
import Sparkline from "@/components/Sparkline.vue";

withDefaults(
  defineProps<{
    label: string;
    value: number;
    percent?: number;
    private?: boolean;
    trend?: number[];
    accent?: "neutral" | "profit" | "loss";
  }>(),
  { percent: undefined, private: false, trend: () => [], accent: "neutral" },
);
</script>

<template>
  <article class="stat-card material-card" :class="accent">
    <div class="stat-heading">
      <span>{{ label }}</span>
      <span v-if="percent !== undefined" class="percent-badge">
        <PercentValue :value="percent" />
      </span>
    </div>
    <div class="stat-body">
      <MoneyValue class="stat-value" :value="value" :private="private" :sign="label !== '总资产'" />
      <div v-if="trend.length" class="stat-spark">
        <Sparkline :values="trend" :positive="trend.at(-1)! >= trend[0]!" />
      </div>
    </div>
  </article>
</template>

<style scoped>
.stat-card {
  min-width: 0;
  padding: 17px 18px 16px;
  background: var(--glass-subtle);
  border: 1px solid var(--hairline);
  border-radius: var(--radius-lg);
  transition: padding 180ms ease, border-radius 180ms ease, background-color 180ms ease;
}

.stat-card::after {
  position: absolute;
  right: -42px;
  bottom: -62px;
  z-index: -1;
  width: 136px;
  height: 136px;
  content: "";
  background: var(--accent-soft);
  border-radius: 50%;
  filter: blur(4px);
  opacity: .38;
}

.stat-card.profit::after { background: var(--profit-soft); }
.stat-card.loss::after { background: var(--loss-soft); }

.stat-heading,
.stat-body {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.stat-heading {
  min-height: 24px;
  font-size: var(--font-sm);
  font-weight: 560;
  color: var(--text-muted);
}

.percent-badge {
  padding: 4px 7px;
  font-size: var(--font-sm);
  background: var(--glass-subtle);
  border: 1px solid var(--hairline);
  border-radius: 999px;
}

.stat-body {
  gap: 12px;
  margin-top: 10px;
}

.stat-value {
  overflow: hidden;
  font-size: clamp(19px, 2vw, 27px);
  font-weight: 710;
  line-height: 1.15;
  color: var(--text-strong);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.stat-card.profit .stat-value {
  color: var(--profit);
}

.stat-card.loss .stat-value {
  color: var(--loss);
}

.stat-spark {
  flex: 0 0 72px;
  height: 27px;
}
</style>
