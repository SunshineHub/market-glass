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
const sortMode = ref<"default" | "day-desc" | "day-asc">("default");
const fundAssets = computed(() => {
  const assets = store.overview.assets.filter((asset) => asset.kind === "fund");
  if (sortMode.value === "default") return assets;
  const direction = sortMode.value === "day-desc" ? -1 : 1;
  return assets.slice().sort((left, right) => {
    const percentOrder = (left.dayProfitPercent - right.dayProfitPercent) * direction;
    if (percentOrder !== 0) return percentOrder;
    const amountOrder = (left.dayProfit - right.dayProfit) * direction;
    return amountOrder || left.name.localeCompare(right.name, "zh-CN");
  });
});
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

function cycleSort() {
  sortMode.value = sortMode.value === "day-desc" ? "day-asc" : "day-desc";
}

async function togglePin() {
  pinned.value = !pinned.value;
  await toggleAlwaysOnTop(pinned.value);
}

function handleWindowPointerDown(event: PointerEvent) {
  if (event.button !== 0) return;
  const target = event.target;
  if (!(target instanceof HTMLElement)) return;
  if (!target.closest("[data-window-drag]")) return;
  if (target.closest(".no-drag, button, a, input, select, textarea, [role='button']")) return;
  void startDraggingCurrentWindow();
}

onMounted(() => store.initialize());
onBeforeUnmount(() => store.dispose());
</script>

<template>
  <div class="app-background mini-shell" @pointerdown="handleWindowPointerDown">
    <div class="mini-glass window-surface">
      <header data-window-drag>
        <div class="mini-brand">
          <span><svg viewBox="0 0 24 24"><path d="m3.5 17 5.8-5.6 4.4 2.9 6.6-8.8" /><circle cx="20.3" cy="5.5" r="1.4" /></svg></span>
          <div><strong>澄明行情</strong><small>基金行情速览</small></div>
        </div>
        <div class="mini-actions no-drag" role="toolbar" aria-label="小窗操作">
          <IconButton :label="store.privacyMode ? '显示金额' : '隐藏金额'" size="small" :active="store.privacyMode" @click="store.togglePrivacy">
            <svg v-if="!store.privacyMode" viewBox="0 0 24 24"><path d="M2.5 12s3.5-6 9.5-6 9.5 6 9.5 6-3.5 6-9.5 6-9.5-6-9.5-6Z"/><circle cx="12" cy="12" r="2.5"/></svg>
            <svg v-else viewBox="0 0 24 24"><path d="m3 3 18 18"/><path d="M10.5 6.1A10.8 10.8 0 0 1 12 6c6 0 9.5 6 9.5 6a15 15 0 0 1-2.2 2.8M6.2 6.2C3.8 8 2.5 12 2.5 12s3.5 6 9.5 6a9 9 0 0 0 3-.5"/></svg>
          </IconButton>
          <IconButton label="置顶" size="small" :active="pinned" @click="togglePin">
            <svg viewBox="0 0 24 24"><path d="m9 4 6 0-1 5 3 3H7l3-3-1-5Z"/><path d="M12 12v8"/></svg>
          </IconButton>
          <IconButton label="打开主窗口" size="small" @click="showWindow('main')">
            <svg viewBox="0 0 24 24"><rect x="4" y="5" width="16" height="14" rx="2"/><path d="M8 9h8M8 13h5"/></svg>
          </IconButton>
          <IconButton label="隐藏" size="small" @click="hideCurrentWindow">
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
          <span class="fund-title">基金 <small>{{ fundAssets.length }}</small></span>
          <span>当前净值</span>
          <button
            type="button"
            class="fund-sort"
            :class="{ ascending: sortMode === 'day-asc', active: sortMode !== 'default' }"
            :aria-label="sortMode === 'default' ? '按当日盈亏百分比从高到低排序' : sortMode === 'day-asc' ? '当日盈亏百分比当前从低到高，点击切换为从高到低' : '当日盈亏百分比当前从高到低，点击切换为从低到高'"
            title="排序依据：当日盈亏百分比"
            @click="cycleSort"
          >
            <span>当日盈亏</span><i />
          </button>
          <span>总盈亏</span>
          <span>更新</span>
        </div>
        <div v-if="fundAssets.length" class="fund-scroll">
          <div v-for="asset in fundAssets" :key="asset.id" class="fund-row">
            <div class="fund-identity" :title="asset.name">
              <strong>{{ asset.name }}</strong>
              <small>{{ asset.code }}</small>
            </div>
            <span class="mono-numbers">{{ navLabel(asset) }}</span>
            <div class="fund-metric" :class="valueClass(asset.dayProfit)">
              <strong class="mono-numbers">{{ signedMoney(asset.dayProfit) }}</strong>
              <small class="mono-numbers">{{ signedPercent(asset.dayProfitPercent) }}</small>
            </div>
            <div v-if="asset.costKnown" class="fund-metric" :class="valueClass(asset.totalProfit)">
              <strong class="mono-numbers">{{ signedMoney(asset.totalProfit) }}</strong>
              <small class="mono-numbers">{{ signedPercent(asset.totalProfitPercent) }}</small>
            </div>
            <div v-else class="fund-metric unknown">
              <strong>—</strong>
              <small>未录成本</small>
            </div>
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
.mini-shell {
  padding: 0;
}

.mini-glass {
  --mini-fund-grid: minmax(0, 1fr) 50px 72px 72px 46px;
  display: flex;
  flex-direction: column;
  gap: 9px;
  width: 100%;
  height: 100%;
  padding: 10px 10px 8px;
  overflow: hidden;
  border-radius: 22px;
  box-shadow: none;
}

:global(html[data-platform="windows"] .mini-glass) {
  border: 0;
  border-radius: 0;
  box-shadow: none;
}

:global(html[data-platform="windows"] .mini-actions),
:global(html[data-platform="windows"] .mini-markets),
:global(html[data-platform="windows"] .fund-panel),
:global(html[data-platform="windows"] .mini-totals) {
  background: var(--material-elevated);
}

:global(html[data-platform="windows"] .fund-head) {
  background: var(--glass-subtle);
}

:global(html[data-platform="windows"] .mini-actions),
:global(html[data-platform="windows"] .mini-markets),
:global(html[data-platform="windows"] .fund-panel),
:global(html[data-platform="windows"] .mini-totals) {
  box-shadow: 0 5px 16px rgba(43, 57, 82, 0.07), 0 1px 0 var(--material-highlight) inset;
}

header,
.mini-actions,
.mini-brand,
footer,
.mini-totals,
.mini-totals > div {
  display: flex;
  align-items: center;
}

header {
  flex: 0 0 38px;
  justify-content: space-between;
  min-width: 0;
  padding-left: 2px;
}

.mini-brand {
  gap: 8px;
  min-width: 0;
}

.mini-brand > span {
  display: grid;
  flex: 0 0 30px;
  width: 30px;
  height: 30px;
  color: white;
  background: linear-gradient(145deg, rgba(255,116,104,.94), rgba(96,120,244,.94));
  border: 1px solid rgba(255,255,255,.34);
  border-radius: 10px;
  box-shadow: 0 6px 15px color-mix(in srgb, var(--accent) 22%, transparent);
  place-items: center;
}

.mini-brand > span svg {
  width: 19px;
  height: 19px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.9;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.mini-brand > div {
  display: flex;
  flex-direction: column;
  min-width: 0;
  gap: 1px;
}

.mini-brand strong {
  overflow: hidden;
  font-size: var(--mini-font-md);
  color: var(--text-strong);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.mini-brand small {
  overflow: hidden;
  font-size: var(--mini-font-xs);
  color: var(--text-muted);
  letter-spacing: .04em;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.mini-actions {
  flex: 0 0 auto;
  gap: 0;
  padding: 2px;
  cursor: pointer;
  background: color-mix(in srgb, var(--material-content) 72%, transparent);
  border: 1px solid color-mix(in srgb, var(--hairline-strong) 72%, transparent);
  border-radius: 12px;
  box-shadow: 0 7px 18px color-mix(in srgb, var(--text-strong) 5%, transparent), 0 1px 0 var(--material-highlight) inset;
  -webkit-app-region: no-drag;
}

.mini-actions :deep(.icon-button) {
  width: 28px;
  height: 28px;
  background: transparent;
  border: 0;
  border-radius: 9px;
  box-shadow: none;
}

.mini-actions :deep(.icon-button:hover) {
  background: color-mix(in srgb, var(--glass-hover) 86%, transparent);
  box-shadow: none;
}

.mini-actions :deep(.icon-button.active) {
  color: var(--accent);
  background: var(--accent-soft);
  box-shadow: none;
}

.mini-markets {
  display: grid;
  flex: 0 0 66px;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  overflow: hidden;
  background: color-mix(in srgb, var(--material-content) 76%, transparent);
  border: 1px solid color-mix(in srgb, var(--hairline-strong) 66%, transparent);
  border-radius: 13px;
  box-shadow: 0 7px 20px color-mix(in srgb, var(--text-strong) 4%, transparent), 0 1px 0 var(--material-highlight) inset;
}

.mini-markets article {
  position: relative;
  display: flex;
  flex-direction: column;
  justify-content: center;
  min-width: 0;
  padding: 8px 10px;
}

.mini-markets article + article::before {
  position: absolute;
  top: 12px;
  bottom: 12px;
  left: 0;
  width: 1px;
  content: "";
  background: color-mix(in srgb, var(--hairline-strong) 58%, transparent);
}

.mini-markets span {
  overflow: hidden;
  font-size: var(--mini-font-xs);
  color: var(--text-muted);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.mini-markets strong {
  margin-top: 4px;
  font-size: var(--mini-font-md);
  color: var(--text-strong);
}

.mini-markets small {
  margin-top: 2px;
  font-size: var(--mini-font-sm);
  font-weight: 690;
}

.fund-panel {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
  background: color-mix(in srgb, var(--material-content) 72%, transparent);
  border: 1px solid color-mix(in srgb, var(--hairline-strong) 68%, transparent);
  border-radius: 15px;
  box-shadow: 0 10px 26px color-mix(in srgb, var(--text-strong) 4%, transparent), 0 1px 0 var(--material-highlight) inset;
}

.fund-head,
.fund-row {
  display: grid;
  grid-template-columns: var(--mini-fund-grid);
  gap: 5px;
  align-items: center;
}

.fund-head {
  flex: 0 0 32px;
  padding: 0 10px;
  font-size: var(--mini-font-xs);
  font-weight: 640;
  line-height: 1;
  color: var(--text-muted);
  letter-spacing: .01em;
  background: color-mix(in srgb, var(--glass-subtle) 48%, transparent);
}

.fund-head > *,
.fund-row > * {
  min-width: 0;
}

.fund-head > * {
  align-self: center;
  justify-self: stretch;
  text-align: center;
}

:global(html[data-platform="macos"] .fund-head) {
  align-items: stretch;
}

:global(html[data-platform="macos"] .fund-head > *) {
  display: flex;
  height: 100%;
  align-items: center;
  justify-content: flex-end;
  text-align: right;
}

:global(html[data-platform="macos"] .fund-head .fund-title) {
  justify-content: flex-start;
}

:global(html[data-platform="macos"] .fund-head .fund-sort) {
  justify-content: flex-end;
}

.fund-title {
  display: inline-flex;
  gap: 5px;
  align-items: center;
  justify-content: center;
  width: 100%;
  color: var(--text);
}

.fund-title small {
  display: inline-grid;
  min-width: 15px;
  height: 15px;
  padding: 0 4px;
  font-size: var(--mini-font-micro);
  font-weight: 700;
  color: var(--accent);
  background: var(--accent-soft);
  border-radius: 99px;
  place-items: center;
}

.fund-sort {
  display: inline-flex;
  width: 100%;
  gap: 4px;
  align-items: center;
  justify-content: center;
  justify-self: stretch;
  padding: 0;
  overflow: visible;
  font-size: inherit;
  font-weight: inherit;
  color: inherit;
  cursor: pointer;
  background: transparent;
  border: 0;
  border-radius: 0;
}

.fund-sort::before,
.fund-sort::after {
  display: none;
}

.fund-sort i {
  width: 0;
  height: 0;
  border-right: 3px solid transparent;
  border-bottom: 4.5px solid currentColor;
  border-left: 3px solid transparent;
  opacity: .38;
  transform: rotate(180deg);
  transition: opacity 150ms ease, transform 180ms ease;
}

.fund-sort.active {
  color: var(--text-strong);
}

.fund-sort.active i {
  opacity: .82;
}

.fund-sort.ascending i {
  transform: rotate(0);
}

.fund-scroll {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-height: 0;
  padding: 2px 4px 4px;
  overflow-x: hidden;
  overflow-y: auto;
  scrollbar-color: color-mix(in srgb, var(--text-muted) 22%, transparent) transparent;
  scrollbar-width: thin;
}

.fund-row {
  min-height: var(--mini-row-height);
  padding: 4px 6px;
  border-radius: 10px;
  transition: background-color 150ms ease, box-shadow 170ms ease;
}

.fund-row:hover {
  background: color-mix(in srgb, var(--material-hover) 72%, transparent);
  box-shadow: 0 5px 14px color-mix(in srgb, var(--text-strong) 4%, transparent), 0 1px 0 var(--material-highlight) inset;
}

.fund-row > :not(:first-child) {
  justify-self: stretch;
  overflow: hidden;
  font-size: var(--mini-font-sm);
  text-align: right;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fund-row time {
  font-size: var(--mini-font-xs);
  color: var(--text-muted);
}

.fund-identity {
  display: flex;
  flex-direction: column;
  justify-content: center;
  min-width: 0;
  min-height: 30px;
}

.fund-identity strong {
  overflow: hidden;
  font-size: var(--mini-font-sm);
  font-weight: 660;
  line-height: 1.15;
  color: var(--text-strong);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fund-identity small {
  margin-top: 2px;
  font-size: var(--mini-font-micro);
  line-height: 1.15;
  color: var(--text-muted);
  letter-spacing: .04em;
}

.fund-metric {
  display: flex;
  flex-direction: column;
  gap: 1px;
  align-items: flex-end;
  justify-content: center;
  min-width: 0;
  min-height: 30px;
}

.fund-metric strong {
  overflow: hidden;
  max-width: 100%;
  font-size: var(--mini-font-xs);
  font-weight: 710;
  line-height: 1.15;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fund-metric small {
  font-size: var(--mini-font-xs);
  font-weight: 680;
  line-height: 1.15;
}

.fund-metric.unknown {
  color: var(--text-muted);
}

.fund-metric.unknown small {
  font-size: var(--mini-font-micro);
  font-weight: 500;
}

.fund-empty {
  display: grid;
  flex: 1;
  color: var(--text-muted);
  text-align: center;
  place-content: center;
}

.fund-empty strong {
  font-size: var(--mini-font-md);
  color: var(--text-strong);
}

.fund-empty span {
  margin-top: 5px;
  font-size: var(--mini-font-xs);
}

.mini-totals {
  flex: 0 0 48px;
  gap: 0;
  overflow: hidden;
  background: color-mix(in srgb, var(--material-content) 75%, transparent);
  border: 1px solid color-mix(in srgb, var(--hairline-strong) 66%, transparent);
  border-radius: 13px;
  box-shadow: 0 7px 20px color-mix(in srgb, var(--text-strong) 4%, transparent), 0 1px 0 var(--material-highlight) inset;
}

.mini-totals > div {
  flex: 1;
  gap: 5px;
  min-width: 0;
  height: 100%;
  padding: 0 11px;
}

.mini-totals > div + div {
  border-left: 1px solid color-mix(in srgb, var(--hairline-strong) 58%, transparent);
}

.mini-totals span {
  margin-right: auto;
  font-size: var(--mini-font-xs);
  color: var(--text-muted);
}

.mini-totals strong {
  font-size: var(--mini-font-md);
}

.mini-totals small {
  font-size: var(--mini-font-sm);
  font-weight: 710;
}

footer {
  flex: 0 0 22px;
  gap: 7px;
  padding: 0 3px;
  font-size: var(--mini-font-xs);
  color: var(--text-muted);
}

.live-dot {
  width: 6px;
  height: 6px;
  background: var(--loss);
  border-radius: 99px;
  box-shadow: 0 0 0 3px var(--loss-soft);
}

.live-dot.refreshing {
  background: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-soft);
}

.live-dot.degraded,
.live-dot.offline {
  background: var(--warning);
}

footer button {
  padding: 4px 8px;
  margin-left: auto;
  font-size: var(--mini-font-sm);
  color: var(--text);
  cursor: pointer;
  background: transparent;
  border: 0;
  border-radius: 8px;
  transition: color 150ms ease, background-color 150ms ease;
}

footer button:hover {
  color: var(--text-strong);
  background: var(--glass-hover);
}

footer button:disabled {
  cursor: default;
  opacity: .55;
}
</style>
