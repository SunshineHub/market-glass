<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import IconButton from "@/components/IconButton.vue";
import MarketStrip from "@/components/MarketStrip.vue";
import MoneyValue from "@/components/MoneyValue.vue";
import PercentValue from "@/components/PercentValue.vue";
import {
  hideCurrentWindow,
  showWindow,
  startDraggingCurrentWindow,
  toggleAlwaysOnTop,
} from "@/ipc/client";
import { usePortfolioStore } from "@/stores/portfolio";

const store = usePortfolioStore();
const pinned = ref(true);
const updatedTime = computed(() => {
  const date = new Date(store.overview.calculatedAt);
  return Number.isNaN(date.getTime())
    ? "--:--"
    : date.toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" });
});

async function togglePin() {
  pinned.value = !pinned.value;
  await toggleAlwaysOnTop(pinned.value);
}

function handleWindowPointerDown(event: PointerEvent) {
  if (event.button !== 0) return;
  const target = event.target;
  if (!(target instanceof HTMLElement)) return;
  if (!target.closest(".drag-region")) return;
  if (target.closest(".no-drag, button, a, input, select, textarea, [role='button']")) return;
  void startDraggingCurrentWindow();
}

onMounted(() => store.initialize());
onBeforeUnmount(() => store.dispose());
</script>

<template>
  <div class="app-background mini-shell" @pointerdown="handleWindowPointerDown">
    <div class="mini-glass window-surface">
      <header class="drag-region" data-tauri-drag-region>
        <div class="mini-brand">
          <span><svg viewBox="0 0 24 24"><path d="m3.5 17 5.8-5.6 4.4 2.9 6.6-8.8" /><circle cx="20.3" cy="5.5" r="1.4" /></svg></span>
          <div><strong>澄明行情</strong><small>Market Glass</small></div>
        </div>
        <div class="mini-actions no-drag">
          <IconButton :label="store.privacyMode ? '显示金额' : '隐藏金额'" size="small" :active="store.privacyMode" data-static-glass @click="store.togglePrivacy">
            <svg v-if="!store.privacyMode" viewBox="0 0 24 24"><path d="M2.5 12s3.5-6 9.5-6 9.5 6 9.5 6-3.5 6-9.5 6-9.5-6-9.5-6Z"/><circle cx="12" cy="12" r="2.5"/></svg>
            <svg v-else viewBox="0 0 24 24"><path d="m3 3 18 18"/><path d="M10.5 6.1A10.8 10.8 0 0 1 12 6c6 0 9.5 6 9.5 6a15 15 0 0 1-2.2 2.8M6.2 6.2C3.8 8 2.5 12 2.5 12s3.5 6 9.5 6a9 9 0 0 0 3-.5"/></svg>
          </IconButton>
          <IconButton label="置顶" size="small" :active="pinned" data-static-glass @click="togglePin">
            <svg viewBox="0 0 24 24"><path d="m9 4 6 0-1 5 3 3H7l3-3-1-5Z"/><path d="M12 12v8"/></svg>
          </IconButton>
          <IconButton label="打开主窗口" size="small" data-static-glass @click="showWindow('main')">
            <svg viewBox="0 0 24 24"><rect x="4" y="5" width="16" height="14" rx="2"/><path d="M8 9h8M8 13h5"/></svg>
          </IconButton>
          <IconButton label="隐藏" size="small" data-static-glass @click="hideCurrentWindow">
            <svg viewBox="0 0 24 24"><path d="m7 7 10 10M17 7 7 17"/></svg>
          </IconButton>
        </div>
      </header>

      <MarketStrip :indices="store.selectedIndices" compact />

      <section class="mini-summary material-card">
        <div v-if="!store.privacyMode" class="mini-assets">
          <span>总资产</span>
          <MoneyValue :value="store.overview.totalAssets" compact />
        </div>
        <div class="mini-profit">
          <span>当日盈亏</span>
          <MoneyValue v-if="!store.privacyMode" :value="store.overview.dayProfit" sign compact :class="store.overview.dayProfit >= 0 ? 'profit' : 'loss'" />
          <PercentValue :value="store.overview.dayProfitPercent" />
        </div>
        <div class="mini-profit">
          <span>总盈亏</span>
          <MoneyValue v-if="!store.privacyMode" :value="store.overview.totalProfit" sign compact :class="store.overview.totalProfit >= 0 ? 'profit' : 'loss'" />
          <PercentValue :value="store.overview.totalProfitPercent" />
        </div>
      </section>

      <footer>
        <span class="live-dot" :class="store.payload.sync.phase" />
        <span>{{ store.payload.sync.message }} · {{ updatedTime }}</span>
        <button type="button" :disabled="store.refreshing" data-liquid-glass @click="store.refresh">
          {{ store.refreshing ? "更新中" : "刷新" }}
        </button>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.mini-shell {
  padding: 0;
}

.mini-glass {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;
  height: 100%;
  padding: 10px 11px 9px;
  overflow: hidden;
  border-radius: 22px;
  box-shadow: none;
}

:global(html[data-platform="windows"] .mini-glass) {
  border: 1px solid var(--hairline-strong);
}

header,
.mini-actions,
.mini-brand,
.mini-summary,
.mini-assets,
.mini-profit,
footer {
  display: flex;
  align-items: center;
}

header {
  flex: 0 0 34px;
  justify-content: space-between;
  padding-left: 2px;
}

.mini-brand {
  gap: 8px;
}

.mini-brand > span {
  display: grid;
  width: 27px;
  height: 27px;
  color: white;
  background: linear-gradient(145deg, rgba(255,116,104,.94), rgba(96,120,244,.94));
  border: 1px solid rgba(255,255,255,.34);
  border-radius: 9px;
  box-shadow: 0 6px 15px color-mix(in srgb, var(--accent) 22%, transparent);
  place-items: center;
}

.mini-brand > span svg { width: 18px; height: 18px; fill: none; stroke: currentColor; stroke-width: 1.9; stroke-linecap: round; stroke-linejoin: round; }
.mini-brand > div { display: flex; flex-direction: column; gap: 1px; }
.mini-brand strong { font-size: 11px; color: var(--text-strong); }
.mini-brand small { font-size: 8px; color: var(--text-muted); letter-spacing: .04em; }

.mini-actions { gap: 5px; }
.mini-actions :deep(.icon-button)::before,
.mini-actions :deep(.icon-button)::after { display: none; }

.mini-summary {
  flex: 1;
  gap: 4px;
  min-height: 62px;
  padding: 5px;
  background: color-mix(in srgb, var(--material-content) 84%, transparent);
  border: 0;
  border-radius: 14px;
  box-shadow: 0 8px 22px color-mix(in srgb, var(--text-strong) 5%, transparent), 0 1px 0 var(--material-highlight) inset;
}

.mini-assets,
.mini-profit {
  flex: 1;
  flex-direction: column;
  justify-content: center;
  min-width: 0;
  height: 100%;
  padding: 7px 8px;
  background: transparent;
  border: 0;
  border-radius: 10px;
}

.mini-assets > span:first-child,
.mini-profit > span:first-child {
  margin-bottom: 5px;
  font-size: 9px;
  color: var(--text-muted);
}

.mini-assets > :last-child {
  font-size: 14px;
  font-weight: 710;
  color: var(--text-strong);
}

.mini-profit > :not(:first-child) {
  font-size: 11px;
}

.mini-profit > :last-child {
  margin-top: 3px;
  font-size: 12px;
  font-weight: 700;
}

footer {
  flex: 0 0 24px;
  gap: 7px;
  padding: 0 3px;
  font-size: 9px;
  color: var(--text-muted);
}

:deep(.market-strip.compact) { gap: 6px; }
:deep(.market-strip.compact .market-item) {
  padding: 7px 9px;
  background: color-mix(in srgb, var(--material-content) 72%, transparent);
  border-color: color-mix(in srgb, var(--hairline) 58%, transparent);
  border-radius: 11px;
  box-shadow: 0 5px 14px color-mix(in srgb, var(--text-strong) 4%, transparent), 0 1px 0 var(--material-highlight) inset;
}

.live-dot {
  width: 6px;
  height: 6px;
  background: var(--loss);
  border-radius: 99px;
  box-shadow: 0 0 0 3px var(--loss-soft);
}

.live-dot.refreshing { background: var(--accent); box-shadow: 0 0 0 3px var(--accent-soft); }
.live-dot.degraded, .live-dot.offline { background: var(--warning); }

footer button {
  padding: 4px 8px;
  margin-left: auto;
  font-size: 9px;
  color: var(--text);
  cursor: pointer;
  background: var(--glass-subtle);
  border: 1px solid var(--hairline);
  border-radius: 8px;
}

footer button:disabled { cursor: default; opacity: 0.55; }
</style>
