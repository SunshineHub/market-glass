<script setup lang="ts">
import { computed } from "vue";
import type { DataNature, Freshness } from "@/types/contracts";

const props = defineProps<{ nature: DataNature; freshness: Freshness }>();
const natureLabel: Record<DataNature, string> = {
  realtime: "实时",
  estimated: "估算",
  confirmed: "已确认",
  manual: "手动",
};
const freshnessLabel: Record<Freshness, string> = {
  fresh: "",
  delayed: " · 延迟",
  stale: " · 过期",
  offline: " · 离线",
};
const label = computed(() => `${natureLabel[props.nature]}${freshnessLabel[props.freshness]}`);
</script>

<template>
  <span class="status-pill" :class="[nature, freshness]">
    <i />
    {{ label }}
  </span>
</template>

<style scoped>
.status-pill {
  display: inline-flex;
  gap: 6px;
  align-items: center;
  width: fit-content;
  padding: 4px 8px;
  font-size: 11px;
  font-weight: 650;
  color: var(--text-muted);
  white-space: nowrap;
  background: var(--glass-subtle);
  border: 1px solid var(--hairline);
  border-radius: 999px;
}

.status-pill i {
  width: 5px;
  height: 5px;
  background: currentColor;
  border-radius: 99px;
}

.estimated,
.realtime {
  color: var(--accent);
}

.confirmed {
  color: var(--loss);
}

.manual,
.delayed {
  color: var(--warning);
}

.stale,
.offline {
  color: var(--text-muted);
}
</style>
