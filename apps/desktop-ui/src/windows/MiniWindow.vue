<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import IconButton from "@/components/IconButton.vue";
import {
  hideCurrentWindow,
  showWindow,
  startDraggingCurrentWindow,
  toggleAlwaysOnTop,
} from "@/ipc/client";
import { usePortfolioStore } from "@/stores/portfolio";
import type { AssetSummary } from "@/types/contracts";

const store = usePortfolioStore();
const pinned = ref(true);
const fundAssets = computed(() =>
  store.overview.assets
    .filter((asset) => asset.kind === "fund")
    .slice()
    .sort((left, right) => right.dayProfitPercent - left.dayProfitPercent),
);
const updatedTime = computed(() => timeLabel(store.overview.calculatedAt));

function timeLabel(value: string) {
  const date = new Date(value);
  return Number.isNaN(date.getTime())
    ? "--:--"
    : date.toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" });
}

function navLabel(asset: AssetSummary) {
  const units = Number(asset.units);
  const nav = asset.currentNav ?? (units > 0 ? asset.currentValue / units : undefined);
  return nav === undefined || !Number.isFinite(nav) ? "—" : nav.toFixed(4);
}

function valueLabel(value: number, fractionDigits = 2) {
  return new Intl.NumberFormat("zh-CN", {
    minimumFractionDigits: fractionDigits,
    maximumFractionDigits: fractionDigits,
  }).format(Math.abs(value));
}

function signedPercent(value: number) {
  if (!Number.isFinite(value)) return "—";
  return `${value >= 0 ? "+" : "−"}${valueLabel(value)}%`;
}

function signedMoney(value: number) {
  if (store.privacyMode) return "••••";
  return `${value >= 0 ? "+" : "−"}¥${valueLabel(value)}`;
}

function valueClass(value: number) {
  return value >= 0 ? "profit" : "loss";
}

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
      <header>
        <div class="mini-brand drag-region" data-tauri-drag-region>
          <span><svg viewBox="0 0 24 24"><path d="m3.5 17 5.8-5.6 4.4 2.9 6.6-8.8" /><circle cx="20.3" cy="5.5" r="1.4" /></svg></span>
          <div><strong>澄明行情</strong><small>基金行情速览</small></div>
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

      <section class="mini-markets" aria-label="自选大盘指数">
        <article v-for="index in store.selectedIndices" :key="index.code">
          <span>{{ index.name }}</span>
          <strong class="mono-numbers">{{ valueLabel(index.value, 2) }}</strong>
          <small class="mono-numbers" :class="valueClass(index.changePercent)">{{ signedPercent(index.changePercent) }}</small>
        </article>
      </section>

      <section class="fund-panel">
        <div class="fund-head">
          <span>基金名称（{{ fundAssets.length }}）</span>
          <span>净值</span>
          <span>涨跌</span>
          <span>当日收益</span>
          <span>更新</span>
        </div>
        <div v-if="fundAssets.length" class="fund-scroll">
          <div v-for="asset in fundAssets" :key="asset.id" class="fund-row">
            <div class="fund-identity" :title="asset.name">
              <strong>{{ asset.name }}</strong>
              <small>{{ asset.code }}</small>
            </div>
            <span class="mono-numbers">{{ navLabel(asset) }}</span>
            <strong class="mono-numbers" :class="valueClass(asset.dayProfitPercent)">{{ signedPercent(asset.dayProfitPercent) }}</strong>
            <span class="mono-numbers" :class="valueClass(asset.dayProfit)">{{ signedMoney(asset.dayProfit) }}</span>
            <time class="mono-numbers">{{ timeLabel(asset.updatedAt) }}</time>
          </div>
        </div>
        <div v-else class="fund-empty">
          <strong>还没有基金</strong>
          <span>在主窗口添加基金后，这里会展示实时行情。</span>
        </div>
      </section>

      <section class="mini-totals">
        <div>
          <span>当日盈亏</span>
          <strong class="mono-numbers" :class="valueClass(store.overview.dayProfit)">{{ signedMoney(store.overview.dayProfit) }}</strong>
          <small class="mono-numbers" :class="valueClass(store.overview.dayProfitPercent)">{{ signedPercent(store.overview.dayProfitPercent) }}</small>
        </div>
        <div>
          <span>总盈亏</span>
          <strong class="mono-numbers" :class="valueClass(store.overview.totalProfit)">{{ signedMoney(store.overview.totalProfit) }}</strong>
          <small class="mono-numbers" :class="valueClass(store.overview.totalProfitPercent)">{{ signedPercent(store.overview.totalProfitPercent) }}</small>
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
.mini-shell { padding: 0; }
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
:global(html[data-platform="windows"] .mini-glass) { border: 1px solid var(--hairline-strong); }
header, .mini-actions, .mini-brand, footer, .mini-totals, .mini-totals > div { display: flex; align-items: center; }
header { flex: 0 0 34px; justify-content: space-between; padding-left: 2px; }
.mini-brand { gap: 8px; }
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
.mini-actions :deep(.icon-button)::before, .mini-actions :deep(.icon-button)::after { display: none; }

.mini-markets {
  display: grid;
  flex: 0 0 62px;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 6px;
}
.mini-markets article {
  display: flex;
  flex-direction: column;
  justify-content: center;
  min-width: 0;
  padding: 7px 8px;
  background: color-mix(in srgb, var(--material-content) 74%, transparent);
  border: 1px solid color-mix(in srgb, var(--hairline) 62%, transparent);
  border-radius: 11px;
  box-shadow: 0 5px 14px color-mix(in srgb, var(--text-strong) 4%, transparent), 0 1px 0 var(--material-highlight) inset;
}
.mini-markets span { overflow: hidden; font-size: 8px; color: var(--text-muted); text-overflow: ellipsis; white-space: nowrap; }
.mini-markets strong { margin-top: 4px; font-size: 11px; color: var(--text-strong); }
.mini-markets small { margin-top: 2px; font-size: 9px; font-weight: 680; }

.fund-panel {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
  background: color-mix(in srgb, var(--material-content) 82%, transparent);
  border: 1px solid color-mix(in srgb, var(--hairline) 74%, transparent);
  border-radius: 14px;
  box-shadow: 0 8px 24px color-mix(in srgb, var(--text-strong) 4%, transparent), 0 1px 0 var(--material-highlight) inset;
}
.fund-head, .fund-row {
  display: grid;
  grid-template-columns: minmax(0, 1.55fr) .55fr .55fr .72fr .44fr;
  gap: 7px;
  align-items: center;
}
.fund-head {
  flex: 0 0 29px;
  padding: 0 10px;
  font-size: 8px;
  color: var(--text-muted);
  background: color-mix(in srgb, var(--glass-subtle) 78%, transparent);
  border-bottom: 1px solid var(--hairline);
}
.fund-head span:not(:first-child) { text-align: right; }
.fund-scroll { min-height: 0; overflow: auto; scrollbar-width: thin; scrollbar-color: color-mix(in srgb, var(--text-muted) 22%, transparent) transparent; }
.fund-row {
  min-height: 42px;
  padding: 5px 10px;
  border-bottom: 1px solid color-mix(in srgb, var(--hairline) 68%, transparent);
}
.fund-row:last-child { border-bottom: 0; }
.fund-row > :not(:first-child) { overflow: hidden; font-size: 9px; text-align: right; text-overflow: ellipsis; white-space: nowrap; }
.fund-row > strong { font-weight: 710; }
.fund-row time { color: var(--text-muted); }
.fund-identity { display: flex; flex-direction: column; min-width: 0; }
.fund-identity strong { overflow: hidden; font-size: 9px; font-weight: 640; color: var(--text-strong); text-overflow: ellipsis; white-space: nowrap; }
.fund-identity small { margin-top: 2px; font-size: 7px; color: var(--text-muted); letter-spacing: .04em; }
.fund-empty { display: grid; flex: 1; color: var(--text-muted); place-content: center; text-align: center; }
.fund-empty strong { font-size: 11px; color: var(--text-strong); }
.fund-empty span { margin-top: 5px; font-size: 8px; }

.mini-totals { flex: 0 0 45px; gap: 7px; }
.mini-totals > div {
  flex: 1;
  gap: 6px;
  min-width: 0;
  height: 100%;
  padding: 0 10px;
  background: color-mix(in srgb, var(--material-content) 76%, transparent);
  border: 1px solid color-mix(in srgb, var(--hairline) 68%, transparent);
  border-radius: 11px;
}
.mini-totals span { margin-right: auto; font-size: 8px; color: var(--text-muted); }
.mini-totals strong { font-size: 10px; }
.mini-totals small { font-size: 9px; font-weight: 700; }

footer { flex: 0 0 24px; gap: 7px; padding: 0 3px; font-size: 8px; color: var(--text-muted); }
.live-dot { width: 6px; height: 6px; background: var(--loss); border-radius: 99px; box-shadow: 0 0 0 3px var(--loss-soft); }
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
footer button:disabled { cursor: default; opacity: .55; }
</style>
