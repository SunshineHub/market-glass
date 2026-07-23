<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import AllocationCard from "@/components/AllocationCard.vue";
import AppSidebar from "@/components/AppSidebar.vue";
import AssetEditorDialog from "@/components/AssetEditorDialog.vue";
import AssetList from "@/components/AssetList.vue";
import BatchAssetEditorDialog from "@/components/BatchAssetEditorDialog.vue";
import IconButton from "@/components/IconButton.vue";
import IndexPickerDialog from "@/components/IndexPickerDialog.vue";
import MarketStrip from "@/components/MarketStrip.vue";
import PercentValue from "@/components/PercentValue.vue";
import StatCard from "@/components/StatCard.vue";
import {
  saveExportFile,
  hideCurrentWindow,
  minimizeCurrentWindow,
  showWindow,
  startDraggingCurrentWindow,
} from "@/ipc/client";
import { createFundExport } from "@/features/export/fundConfig";
import { usePortfolioStore } from "@/stores/portfolio";
import type { AssetSummary, PositionInput, PositionUpdateFailure } from "@/types/contracts";

const store = usePortfolioStore();
const active = ref("overview");
const editorOpen = ref(false);
const editorSaving = ref(false);
const editingAsset = ref<AssetSummary>();
const batchEditorOpen = ref(false);
const batchEditorSaving = ref(false);
const batchEditingAssets = ref<AssetSummary[]>([]);
const batchFailures = ref<PositionUpdateFailure[]>([]);
const batchSuccessCount = ref(0);
const indexEditorOpen = ref(false);
const indexEditorSaving = ref(false);
const marketEditorOpen = ref(false);
const marketEditorSaving = ref(false);
const assetDeleting = ref(false);
const assetExportState = ref<"idle" | "saving" | "done" | "error">("idle");
const titleMap: Record<string, [string, string]> = {
  overview: ["资产总览", "基金与全球市场的一站式观察"],
  analysis: ["资产分析", "行业分布与持仓数据覆盖"],
  funds: ["基金", "管理公募基金、成本与交易流水"],
  market: ["大盘", "跟踪主要指数与市场状态"],
};

const pageTitle = computed(() => titleMap[active.value] ?? titleMap.overview);
const dayTone = computed(() => (store.overview.dayProfit >= 0 ? "profit" : "loss"));
const totalTone = computed(() => (store.overview.totalProfit >= 0 ? "profit" : "loss"));
const fundAssets = computed(() => store.overview.assets.filter((asset) => asset.kind === "fund"));
const categoryAssets = fundAssets;
const categoryStats = computed(() => summarizeAssets(categoryAssets.value));
const categoryName = "公募基金";
const zeroValueAssets = computed(() =>
  store.overview.assets.filter((asset) => asset.currentValue === 0).length,
);
const freshAssets = computed(() =>
  store.overview.assets.filter((asset) => asset.freshness === "fresh").length,
);
const knownCostAssets = computed(() =>
  fundAssets.value.filter((asset) => asset.costKnown).length,
);
const totalProfitLabel = computed(() =>
  knownCostAssets.value < fundAssets.value.length ? "总盈亏 · 已录成本" : "总盈亏",
);
const marketIndices = computed(() =>
  store.payload.marketIndexCodes
    .map((code) => store.overview.indices.find((quote) => quote.code === code))
    .filter((quote): quote is NonNullable<typeof quote> => Boolean(quote)),
);
const marketRegionGroups = computed(() => {
  const groups = new Map<string, typeof store.overview.indices>();
  for (const quote of marketIndices.value) {
    const region = regionLabel(quote.code);
    const items = groups.get(region) ?? [];
    items.push(quote);
    groups.set(region, items);
  }
  const order = ["中国内地", "中国香港", "美国", "欧洲", "亚太", "其他市场"];
  return [...groups.entries()]
    .sort(([left], [right]) => order.indexOf(left) - order.indexOf(right))
    .map(([region, items]) => ({ region, items }));
});

function regionLabel(code: string) {
  return store.payload.indexOptions.find((option) => option.code === code)?.region ?? "其他市场";
}
const statusTime = computed(() => {
  const raw = store.payload.sync.lastSuccessAt ?? store.overview.calculatedAt;
  const date = new Date(raw);
  return Number.isNaN(date.getTime())
    ? "--:--"
    : date.toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" });
});

function summarizeAssets(assets: AssetSummary[]) {
  const totalAssets = assets.reduce((total, asset) => total + asset.currentValue, 0);
  const dayProfit = assets.reduce((total, asset) => total + asset.dayProfit, 0);
  const totalProfit = assets.reduce((total, asset) => total + asset.totalProfit, 0);
  const previousAssets = totalAssets - dayProfit;
  const totalCost = assets
    .filter((asset) => asset.costKnown)
    .reduce((total, asset) => total + asset.currentValue - asset.totalProfit, 0);
  return {
    totalAssets,
    dayProfit,
    dayProfitPercent: previousAssets === 0 ? 0 : (dayProfit / previousAssets) * 100,
    totalProfit,
    totalProfitPercent: totalCost === 0 ? 0 : (totalProfit / totalCost) * 100,
  };
}

function signedNumber(value: number) {
  return `${value >= 0 ? "+" : ""}${value.toFixed(2)}`;
}

function timeLabel(value: string) {
  const date = new Date(value);
  return Number.isNaN(date.getTime())
    ? "--:--"
    : date.toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" });
}

function handleWindowPointerDown(event: PointerEvent) {
  if (event.button !== 0) return;
  const target = event.target;
  if (!(target instanceof HTMLElement)) return;
  if (!target.closest(".drag-region")) return;
  if (target.closest(".no-drag, button, a, input, select, textarea, [role='button']")) return;
  void startDraggingCurrentWindow();
}

function openEditor(asset?: AssetSummary) {
  editingAsset.value = asset;
  editorOpen.value = true;
}

function closeEditor() {
  editorOpen.value = false;
  editingAsset.value = undefined;
}

function openBatchEditor(assets: AssetSummary[]) {
  if (!assets.length) return;
  batchEditingAssets.value = assets;
  batchFailures.value = [];
  batchSuccessCount.value = 0;
  batchEditorOpen.value = true;
}

function closeBatchEditor() {
  batchEditorOpen.value = false;
  batchEditingAssets.value = [];
  batchFailures.value = [];
  batchSuccessCount.value = 0;
}

async function saveAsset(input: PositionInput) {
  if (editorSaving.value) return;
  editorSaving.value = true;
  try {
    await store.savePosition(input);
    closeEditor();
  } finally {
    editorSaving.value = false;
  }
}

async function saveBatchAssets(inputs: PositionInput[]) {
  if (batchEditorSaving.value || !inputs.length) return;
  batchEditorSaving.value = true;
  batchFailures.value = [];
  batchSuccessCount.value = 0;
  try {
    const result = await store.updatePositions(inputs);
    batchFailures.value = result.failures;
    batchSuccessCount.value = result.succeededIds.length;
    if (!result.failures.length) closeBatchEditor();
  } finally {
    batchEditorSaving.value = false;
  }
}

async function importAssets(inputs: PositionInput[]) {
  if (editorSaving.value) return;
  editorSaving.value = true;
  try {
    await store.savePositions(inputs);
    editorOpen.value = false;
  } finally {
    editorSaving.value = false;
  }
}

async function saveSelectedIndices(codes: string[]) {
  if (indexEditorSaving.value) return;
  indexEditorSaving.value = true;
  try {
    await store.updateSelectedIndices(codes);
    indexEditorOpen.value = false;
  } finally {
    indexEditorSaving.value = false;
  }
}

async function saveMarketIndices(codes: string[]) {
  if (marketEditorSaving.value) return;
  marketEditorSaving.value = true;
  try {
    await store.updateMarketIndices(codes);
    marketEditorOpen.value = false;
  } finally {
    marketEditorSaving.value = false;
  }
}

async function deleteAssets(ids: string[]) {
  if (assetDeleting.value || !ids.length) return;
  assetDeleting.value = true;
  try {
    await store.removePositions(ids);
  } finally {
    assetDeleting.value = false;
  }
}

async function exportAssets() {
  if (assetExportState.value === "saving") return;
  assetExportState.value = "saving";
  try {
    const exported = createFundExport(fundAssets.value);
    if (!exported.count) throw new Error("没有可导出的基金资产");
    await saveExportFile(exported.filename, exported.content);
    assetExportState.value = "done";
  } catch {
    assetExportState.value = "error";
  }
  window.setTimeout(() => {
    assetExportState.value = "idle";
  }, 2400);
}

const exportButtonLabel = computed(() => {
  if (assetExportState.value === "saving") return "正在导出…";
  if (assetExportState.value === "done") return "已保存到下载目录";
  if (assetExportState.value === "error") return "导出失败";
  return "导出配置";
});

onMounted(() => store.initialize());
onBeforeUnmount(() => store.dispose());
</script>

<template>
  <div class="app-background main-shell" @pointerdown="handleWindowPointerDown">
    <div class="main-glass glass-panel window-surface">
      <AppSidebar :active="active" @select="active = $event" />

      <main>
        <header class="topbar">
          <div class="topbar-title drag-region" data-tauri-drag-region>
            <h1>{{ pageTitle[0] }}</h1>
            <p>{{ pageTitle[1] }}</p>
          </div>
          <div class="top-actions no-drag">
            <span class="sync-state" :class="store.payload.sync.phase">
              <i />{{ store.payload.sync.message }} · {{ statusTime }}
            </span>
            <IconButton label="立即刷新" :active="store.refreshing" @click="store.refresh">
              <svg viewBox="0 0 24 24" :class="{ spinning: store.refreshing }"><path d="M20 11a8 8 0 1 0-2.3 5.7"/><path d="M20 5v6h-6"/></svg>
            </IconButton>
            <IconButton :label="store.privacyMode ? '显示金额' : '隐藏金额'" :active="store.privacyMode" @click="store.togglePrivacy">
              <svg v-if="!store.privacyMode" viewBox="0 0 24 24"><path d="M2.5 12s3.5-6 9.5-6 9.5 6 9.5 6-3.5 6-9.5 6-9.5-6-9.5-6Z"/><circle cx="12" cy="12" r="2.5"/></svg>
              <svg v-else viewBox="0 0 24 24"><path d="m3 3 18 18"/><path d="M10.5 6.1A10.8 10.8 0 0 1 12 6c6 0 9.5 6 9.5 6a15 15 0 0 1-2.2 2.8M6.2 6.2C3.8 8 2.5 12 2.5 12s3.5 6 9.5 6a9 9 0 0 0 3-.5"/></svg>
            </IconButton>
            <IconButton label="打开极简窗口" @click="showWindow('mini')">
              <svg viewBox="0 0 24 24"><rect x="4" y="5" width="16" height="14" rx="2"/><path d="M8 9h8M8 13h5"/></svg>
            </IconButton>
            <span class="window-controls">
              <span class="window-divider" />
              <IconButton label="最小化" size="small" @click="minimizeCurrentWindow">
                <svg viewBox="0 0 24 24"><path d="M6 12h12"/></svg>
              </IconButton>
              <IconButton label="关闭并保留后台刷新" size="small" @click="hideCurrentWindow">
                <svg viewBox="0 0 24 24"><path d="m7 7 10 10M17 7 7 17"/></svg>
              </IconButton>
            </span>
          </div>
        </header>

        <Transition name="page-shift" mode="out-in">
        <div :key="active" class="page-stage">
        <div v-if="active === 'overview'" class="overview-page">
          <section class="overview-market">
            <header>
              <div><span>自选市场</span><strong>关注指数</strong></div>
              <button type="button" class="no-drag" @click="indexEditorOpen = true">
                <svg viewBox="0 0 24 24"><path d="M4 20h4l11-11-4-4L4 16z" /><path d="m13.5 6.5 4 4" /></svg>
                编辑
              </button>
            </header>
            <MarketStrip :indices="store.selectedIndices" />
          </section>

          <section class="stat-grid">
            <StatCard label="总资产" :value="store.overview.totalAssets" :private="store.privacyMode" />
            <StatCard label="当日盈亏" :value="store.overview.dayProfit" :percent="store.overview.dayProfitPercent" :private="store.privacyMode" :accent="dayTone" />
            <StatCard :label="totalProfitLabel" :value="store.overview.totalProfit" :percent="store.overview.totalProfitPercent" :private="store.privacyMode" :accent="totalTone" />
          </section>

          <AssetList
            :assets="store.overview.assets"
            :private="store.privacyMode"
            :deleting="assetDeleting"
            sortable
            :exporting="assetExportState === 'saving'"
            :export-label="exportButtonLabel"
            @manage="openEditor()"
            @edit="openEditor"
            @batch-edit="openBatchEditor"
            @remove="deleteAssets"
            @export="exportAssets"
          />
        </div>

        <div v-else-if="active === 'analysis'" class="analysis-page" data-testid="analysis-page">
          <section class="analysis-hero material-card">
            <div><span>PORTFOLIO INTELLIGENCE</span><h2>看清资产分布</h2><p>基于本地持仓标签汇总行业方向，并检查行情覆盖与观察资产。</p></div>
            <strong>{{ store.overview.assets.length }}<small> 项资产</small></strong>
          </section>
          <section class="overview-insights">
            <AllocationCard :slices="store.overview.allocation" />
            <section class="portfolio-pulse material-card">
              <header><div><span>持仓概览</span><h3>数据覆盖</h3></div><small>本地资产库</small></header>
              <div class="pulse-grid">
                <div><strong>{{ fundAssets.length }}</strong><span>基金</span></div>
                <div><strong>{{ store.overview.assets.length }}</strong><span>资产条目</span></div>
                <div><strong>{{ freshAssets }}</strong><span>行情新鲜</span></div>
                <div><strong>{{ zeroValueAssets }}</strong><span>仅观察</span></div>
              </div>
            </section>
          </section>
          <section class="analysis-note material-card">
            <span>口径说明</span>
            <p>行业比例优先采用你填写的“行业 / 策略标签”，未填写时根据基金名称归类；0 份额基金会保留在观察清单，但不进入行业金额占比。</p>
          </section>
        </div>

        <div
          v-else-if="active === 'funds'"
          class="category-page"
          :data-testid="`${active}-page`"
        >
          <section class="category-hero material-card">
            <div>
              <span>FUND PORTFOLIO</span>
              <h2>{{ categoryName }}</h2>
              <p>集中查看基金估值、当日收益与累计表现。</p>
            </div>
            <strong>{{ categoryAssets.length }}<small> 项资产</small></strong>
          </section>

          <section class="stat-grid">
            <StatCard label="总资产" :value="categoryStats.totalAssets" :private="store.privacyMode" />
            <StatCard
              label="当日盈亏"
              :value="categoryStats.dayProfit"
              :percent="categoryStats.dayProfitPercent"
              :private="store.privacyMode"
              :accent="categoryStats.dayProfit >= 0 ? 'profit' : 'loss'"
            />
            <StatCard
              :label="totalProfitLabel"
              :value="categoryStats.totalProfit"
              :percent="categoryStats.totalProfitPercent"
              :private="store.privacyMode"
              :accent="categoryStats.totalProfit >= 0 ? 'profit' : 'loss'"
            />
          </section>

          <AssetList
            :assets="categoryAssets"
            :private="store.privacyMode"
            eyebrow="分类资产"
            :title="categoryName"
            :empty-title="`还没有${categoryName}`"
            empty-description="当前配置中没有基金，可通过添加或配置导入补充。"
            sortable
            :deleting="assetDeleting"
            :exporting="assetExportState === 'saving'"
            :export-label="exportButtonLabel"
            @manage="openEditor()"
            @edit="openEditor"
            @batch-edit="openBatchEditor"
            @remove="deleteAssets"
            @export="exportAssets"
          />
        </div>

        <div v-else-if="active === 'market'" class="market-page" data-testid="market-page">
          <section class="global-market-hero material-card">
            <div><span>GLOBAL MARKETS</span><h2>全球主要股票市场</h2><p>覆盖中国、美国、欧洲与亚太主要指数，按各市场交易时段刷新。</p></div>
            <div class="market-hero-actions">
              <strong>{{ marketIndices.length }}<small> 个市场指数</small></strong>
              <button type="button" data-liquid-glass @click="marketEditorOpen = true">
                <svg viewBox="0 0 24 24"><path d="M12 5v14M5 12h14" /></svg>
                添加 / 管理指数
              </button>
            </div>
          </section>
          <section class="market-groups" aria-label="全球市场分区">
            <section v-for="group in marketRegionGroups" :key="group.region" class="market-region-section">
              <header class="market-region-heading">
                <div><i /><h3>{{ group.region }}</h3></div>
                <span>{{ group.items.length }} 个指数</span>
              </header>
              <div class="market-detail-grid">
                <article
                  v-for="index in group.items"
                  :key="index.code"
                  class="market-detail-card material-card"
                >
                  <header>
                    <div><span>{{ index.code }}</span><h2>{{ index.name }}</h2></div>
                  </header>
                  <div class="market-detail-value">
                    <strong class="mono-numbers">{{ index.value.toFixed(2) }}</strong>
                    <PercentValue :value="index.changePercent" />
                  </div>
                  <footer>
                    <span :class="index.changePercent >= 0 ? 'profit' : 'loss'" class="mono-numbers">{{ signedNumber(index.change) }}</span>
                    <strong>{{ timeLabel(index.updatedAt) }}</strong>
                  </footer>
                </article>
              </div>
            </section>
          </section>
          <section v-if="!marketIndices.length" class="market-empty glass-panel">
            暂时无法获取大盘行情，应用会保留上次成功数据并继续自动重试。
          </section>
        </div>

        <div v-else class="placeholder-page">
          <div class="placeholder-icon">设</div>
          <h2>{{ pageTitle[0] }}</h2>
          <p>{{ pageTitle[1] }}</p>
          <span>该模块已预留稳定接口，将在后续迭代中逐步启用。</span>
        </div>
        </div>
        </Transition>
      </main>
    </div>
    <AssetEditorDialog
      v-if="editorOpen"
      :asset="editingAsset"
      :saving="editorSaving"
      @close="closeEditor"
      @save="saveAsset"
      @import="importAssets"
    />
    <BatchAssetEditorDialog
      v-if="batchEditorOpen"
      :assets="batchEditingAssets"
      :saving="batchEditorSaving"
      :failures="batchFailures"
      :success-count="batchSuccessCount"
      @close="closeBatchEditor"
      @save="saveBatchAssets"
    />
    <IndexPickerDialog
      v-if="indexEditorOpen"
      :options="store.payload.indexOptions"
      :selected="store.payload.selectedIndexCodes"
      :saving="indexEditorSaving"
      @close="indexEditorOpen = false"
      @save="saveSelectedIndices"
    />
    <IndexPickerDialog
      v-if="marketEditorOpen"
      :options="store.payload.indexOptions"
      :selected="store.payload.marketIndexCodes"
      :saving="marketEditorSaving"
      :max-selections="Math.max(store.payload.indexOptions.length, 1)"
      eyebrow="GLOBAL WATCHLIST"
      title="管理大盘指数"
      description="按区域添加或移除大盘指数；这里只展示已接入实时行情的数据。"
      save-label="保存大盘指数"
      @close="marketEditorOpen = false"
      @save="saveMarketIndices"
    />
  </div>
</template>

<style scoped>
.main-shell {
  padding: 12px;
}

:global(html[data-platform="macos"] .main-shell) {
  padding: 0;
}

.main-glass {
  display: flex;
  width: 100%;
  height: 100%;
  overflow: hidden;
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow);
}

:global(html[data-platform="macos"] .main-glass) {
  border: 0;
  border-radius: 0;
  box-shadow: none;
}

main {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  background:
    radial-gradient(circle at 86% 0%, color-mix(in srgb, var(--bg-b) 38%, transparent), transparent 42%),
    radial-gradient(circle at 22% 108%, color-mix(in srgb, var(--bg-a) 24%, transparent), transparent 40%),
    linear-gradient(118deg, color-mix(in srgb, var(--content-surface) 90%, transparent), color-mix(in srgb, var(--content-surface) 68%, var(--bg-b)));
}

.page-stage {
  display: flex;
  flex: 1;
  min-width: 0;
  min-height: 0;
}

.page-shift-enter-active,
.page-shift-leave-active {
  transition: opacity 150ms ease, transform 190ms cubic-bezier(.2, .8, .2, 1), filter 180ms ease;
}

.page-shift-enter-from {
  opacity: 0;
  filter: saturate(.82);
  transform: translateX(7px);
}

.page-shift-leave-to {
  opacity: 0;
  filter: saturate(.88);
  transform: translateX(-5px);
}

.topbar {
  display: flex;
  flex: 0 0 80px;
  align-items: center;
  justify-content: space-between;
  padding: 14px 20px 10px 24px;
  background: linear-gradient(96deg, color-mix(in srgb, var(--content-surface) 72%, transparent), color-mix(in srgb, var(--bg-b) 18%, transparent));
  border-bottom: 1px solid var(--hairline);
}

.topbar-title {
  display: flex;
  align-self: stretch;
  flex: 1;
  flex-direction: column;
  justify-content: center;
  min-width: 0;
}

h1 {
  margin: 0;
  font-size: var(--font-lg);
  font-weight: 710;
  color: var(--text-strong);
  letter-spacing: -0.03em;
}

.topbar p {
  margin: 4px 0 0;
  font-size: var(--font-sm);
  color: var(--text-muted);
}

.top-actions {
  display: flex;
  gap: 7px;
  align-items: center;
}

.window-controls {
  display: flex;
  gap: 7px;
  align-items: center;
}

:global(html[data-platform="macos"] .window-controls) {
  display: none;
}

:global(html[data-platform="macos"] .sidebar) {
  padding-top: 48px;
}

.sync-state {
  display: inline-flex;
  gap: 7px;
  align-items: center;
  margin-right: 4px;
  font-size: var(--font-xs);
  color: var(--text-muted);
}

.sync-state i {
  width: 6px;
  height: 6px;
  background: var(--loss);
  border-radius: 50%;
  box-shadow: 0 0 0 3px var(--loss-soft);
}

.sync-state.refreshing i {
  background: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-soft);
}

.sync-state.degraded i,
.sync-state.offline i {
  background: var(--warning);
  box-shadow: 0 0 0 3px rgba(216, 139, 44, 0.12);
}

.window-divider {
  width: 1px;
  height: 22px;
  margin: 0 2px;
  background: var(--hairline);
}

.spinning {
  animation: spin 800ms linear infinite;
}

@keyframes spin { to { transform: rotate(360deg); } }

.overview-page {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 14px;
  min-height: 0;
  padding: 16px 20px 18px;
  overflow: auto;
}

.analysis-page {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 14px;
  min-height: 0;
  padding: 16px 20px 18px;
  overflow: auto;
}

.analysis-hero {
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-height: 118px;
  padding: 21px 24px;
  background:
    radial-gradient(circle at 82% 14%, var(--accent-soft), transparent 38%),
    var(--glass-subtle);
  border-radius: var(--radius-lg);
}

.analysis-hero span {
  font-size: var(--font-xs);
  font-weight: 700;
  color: var(--accent);
  letter-spacing: .12em;
}

.analysis-hero h2 {
  margin: 5px 0 0;
  font-size: var(--font-lg);
  color: var(--text-strong);
}

.analysis-hero p {
  margin: 7px 0 0;
  font-size: var(--font-sm);
  color: var(--text-muted);
}

.analysis-hero > strong {
  font: 720 var(--font-display)/1 var(--font-mono);
  color: var(--text-strong);
}

.analysis-hero > strong small {
  font: 500 var(--font-xs)/1 var(--font-mono);
  color: var(--text-muted);
}

.analysis-page .overview-insights {
  flex: 0 0 250px;
  min-height: 0;
}

.analysis-note {
  padding: 15px 18px;
  background: var(--glass-subtle);
  border-radius: var(--radius-md);
}

.analysis-note span {
  font-size: var(--font-xs);
  font-weight: 680;
  color: var(--text-strong);
}

.analysis-note p {
  margin: 5px 0 0;
  font-size: var(--font-sm);
  line-height: 1.6;
  color: var(--text-muted);
}

.stat-grid {
  display: grid;
  grid-template-columns: 1.2fr 1fr 1fr;
  gap: 12px;
}

.overview-market {
  display: flex;
  flex-direction: column;
  gap: 9px;
}

.overview-market > header,
.overview-market > header > div,
.overview-market > header button,
.portfolio-pulse header {
  display: flex;
  align-items: center;
}

.overview-market > header {
  justify-content: space-between;
  padding: 0 2px;
}

.overview-market > header > div {
  gap: 9px;
}

.overview-market > header span,
.portfolio-pulse header span {
  font-size: var(--font-xs);
  color: var(--text-muted);
}

.overview-market > header strong {
  font-size: var(--font-sm);
  color: var(--text-strong);
}

.overview-market > header button {
  gap: 6px;
  padding: 6px 9px;
  font-size: var(--font-xs);
  color: var(--text);
  cursor: pointer;
  background: var(--glass-subtle);
  border: 1px solid var(--hairline);
  border-radius: 9px;
}

.overview-market > header button svg {
  width: 13px;
  height: 13px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.8;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.overview-insights {
  display: grid;
  grid-template-columns: minmax(360px, 1.12fr) minmax(300px, 0.88fr);
  gap: 12px;
  min-height: 168px;
  transition: grid-template-columns 180ms ease;
}

.portfolio-pulse {
  padding: 18px 20px;
  border-radius: var(--radius-lg);
}

.portfolio-pulse header {
  justify-content: space-between;
}

.portfolio-pulse header > div {
  display: flex;
  flex-direction: column;
}

.portfolio-pulse h3 {
  margin: 4px 0 0;
  font-size: var(--font-md);
  color: var(--text-strong);
}

.portfolio-pulse header small {
  padding: 5px 8px;
  font-size: var(--font-xs);
  color: var(--text-muted);
  background: var(--glass-subtle);
  border: 1px solid var(--hairline);
  border-radius: 99px;
}

.pulse-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 8px;
  margin-top: 24px;
}

.pulse-grid > div {
  display: flex;
  flex-direction: column;
  gap: 5px;
  min-width: 0;
  padding: 11px 8px;
  text-align: center;
  background: var(--glass-subtle);
  border: 1px solid var(--hairline);
  border-radius: 12px;
}

.pulse-grid strong {
  font: 720 var(--font-lg)/1 var(--font-mono);
  color: var(--text-strong);
}

.pulse-grid span {
  overflow: hidden;
  font-size: var(--font-xs);
  color: var(--text-muted);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.category-page,
.market-page {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 14px;
  min-height: 0;
  padding: 16px 20px 18px;
  overflow: auto;
}

.category-hero {
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-height: 106px;
  padding: 19px 22px;
  overflow: hidden;
  background:
    radial-gradient(circle at 84% 12%, var(--accent-soft), transparent 36%),
    var(--glass-subtle);
  border-radius: var(--radius-lg);
}

.category-hero span {
  font-size: var(--font-xs);
  font-weight: 700;
  color: var(--accent);
  letter-spacing: 0.12em;
}

.category-hero h2 {
  margin: 5px 0 0;
  font-size: var(--font-lg);
  color: var(--text-strong);
}

.category-hero p {
  margin: 7px 0 0;
  font-size: var(--font-sm);
  color: var(--text-muted);
}

.category-hero > strong {
  font: 720 var(--font-display)/1 var(--font-mono);
  color: var(--text-strong);
}

.category-hero > strong small {
  font: 500 var(--font-xs)/1 var(--font-mono);
  color: var(--text-muted);
}

.category-page :deep(.asset-list) {
  flex: 1;
}

.market-detail-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 8px;
}

.global-market-hero {
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-height: 72px;
  padding: 11px 16px;
  background:
    radial-gradient(circle at 82% 14%, var(--accent-soft), transparent 38%),
    var(--glass-subtle);
  border-radius: var(--radius-lg);
}

.global-market-hero span {
  font-size: var(--font-xs);
  font-weight: 700;
  color: var(--accent);
  letter-spacing: .12em;
}

.global-market-hero h2 {
  margin: 5px 0 0;
  font-size: var(--font-lg);
  color: var(--text-strong);
}

.global-market-hero p {
  margin: 4px 0 0;
  font-size: var(--font-sm);
  color: var(--text-muted);
}

.global-market-hero > strong {
  font: 720 var(--font-display)/1 var(--font-mono);
  color: var(--text-strong);
}

.market-hero-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
  align-items: flex-end;
}

.market-hero-actions > strong {
  font: 720 var(--font-display)/1 var(--font-mono);
  color: var(--text-strong);
}

.market-hero-actions > strong small {
  font: 500 var(--font-xs)/1 var(--font-mono);
  color: var(--text-muted);
}

.market-hero-actions button {
  display: inline-flex;
  gap: 6px;
  align-items: center;
  padding: 6px 9px;
  font-size: var(--font-xs);
  color: var(--text);
  cursor: pointer;
  background: var(--glass-subtle);
  border: 1px solid var(--hairline);
  border-radius: 9px;
}

.market-hero-actions button svg {
  width: 13px;
  height: 13px;
  fill: none;
  stroke: currentColor;
  stroke-width: 2;
  stroke-linecap: round;
}

.global-market-hero > strong small {
  font: 500 var(--font-xs)/1 var(--font-mono);
  color: var(--text-muted);
}

.market-groups {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.market-region-section {
  min-width: 0;
}

.market-region-heading,
.market-region-heading > div {
  display: flex;
  align-items: center;
}

.market-region-heading {
  justify-content: space-between;
  min-height: 22px;
  padding: 0 3px 5px;
}

.market-region-heading > div {
  gap: 7px;
}

.market-region-heading i {
  width: 6px;
  height: 6px;
  background: var(--accent);
  border-radius: 50%;
  box-shadow: 0 0 0 3px var(--accent-soft);
}

.market-region-heading h3 {
  margin: 0;
  font-size: var(--font-sm);
  color: var(--text-strong);
}

.market-region-heading span {
  font-size: var(--font-xs);
  color: var(--text-muted);
}

.market-detail-card {
  min-width: 0;
  padding: 8px 10px 6px;
  background: var(--glass-subtle);
  border: 1px solid var(--hairline);
  border-radius: 14px;
  transition: padding 180ms ease, border-radius 180ms ease, transform 180ms ease,
    background-color 180ms ease;
}

.market-detail-card header,
.market-detail-value,
.market-detail-value > div,
.market-detail-card footer,
.market-live {
  display: flex;
  align-items: center;
}

.market-detail-card header,
.market-detail-value,
.market-detail-card footer {
  justify-content: space-between;
}

.market-detail-card header > div span {
  font: 600 var(--font-xs)/1 var(--font-mono);
  color: var(--text-muted);
}

.market-detail-card h2 {
  margin: 3px 0 0;
  font-size: var(--font-sm);
  color: var(--text-strong);
}

.market-live {
  gap: 6px;
  padding: 5px 8px;
  font-size: var(--font-xs);
  color: var(--loss);
  background: var(--loss-soft);
  border-radius: 99px;
}

.market-live i {
  width: 5px;
  height: 5px;
  background: currentColor;
  border-radius: 50%;
}

.market-live.stale,
.market-live.offline,
.market-live.delayed {
  color: var(--warning);
}

.market-detail-value {
  margin-top: 4px;
}

.market-detail-value > strong {
  font-size: clamp(16px, 1.45vw, 20px);
  color: var(--text-strong);
}

.market-detail-value > div {
  gap: 10px;
  font-size: var(--font-sm);
}

.market-detail-card footer {
  padding-top: 5px;
  margin-top: 5px;
  font-size: var(--font-xs);
  color: var(--text-muted);
  border-top: 1px solid var(--hairline);
}

.market-detail-card footer strong {
  color: var(--text);
}

.market-empty {
  display: grid;
  min-height: 150px;
  padding: 30px;
  font-size: var(--font-sm);
  color: var(--text-muted);
  text-align: center;
  border-radius: var(--radius-lg);
  place-content: center;
}

.placeholder-page {
  display: grid;
  flex: 1;
  color: var(--text-muted);
  text-align: center;
  place-content: center;
}

.placeholder-icon {
  display: grid;
  width: 56px;
  height: 56px;
  margin: 0 auto 14px;
  font-size: 17px;
  font-weight: 750;
  color: var(--accent);
  background: var(--accent-soft);
  border: 1px solid color-mix(in srgb, var(--accent) 24%, transparent);
  border-radius: 19px;
  place-items: center;
}

.placeholder-page h2 { margin: 0; font-size: var(--font-lg); color: var(--text-strong); }
.placeholder-page p { margin: 8px 0 5px; font-size: var(--font-sm); }
.placeholder-page span { font-size: var(--font-xs); }

@media (max-width: 1160px) {
  .sync-state { display: none; }
  .overview-insights { grid-template-columns: minmax(320px, 1fr) minmax(260px, .82fr); }
  .pulse-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); margin-top: 15px; }
}

@media (max-width: 980px) {
  .topbar { padding-right: 14px; padding-left: 18px; }
  .overview-page,
  .analysis-page,
  .category-page,
  .market-page { gap: 11px; padding: 13px 14px 15px; }
  .stat-grid { gap: 9px; }
  .category-hero,
  .global-market-hero { padding-right: 17px; padding-left: 17px; }
}
</style>
