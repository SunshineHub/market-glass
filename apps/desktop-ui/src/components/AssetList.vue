<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { AssetSummary } from "@/types/contracts";
import ConfirmDialog from "@/components/ConfirmDialog.vue";
import MoneyValue from "@/components/MoneyValue.vue";
import PercentValue from "@/components/PercentValue.vue";

const props = withDefaults(
  defineProps<{
    assets: AssetSummary[];
    private: boolean;
    eyebrow?: string;
    title?: string;
    emptyTitle?: string;
    emptyDescription?: string;
    sortable?: boolean;
    deleting?: boolean;
    exporting?: boolean;
    exportLabel?: string;
  }>(),
  {
    eyebrow: "资产列表",
    title: "基金资产",
    emptyTitle: "还没有资产",
    emptyDescription: "添加第一只基金后，这里会显示收益概览。",
    sortable: false,
    deleting: false,
    exporting: false,
    exportLabel: "导出配置",
  },
);
const emit = defineEmits<{
  manage: [];
  remove: [ids: string[]];
  export: [];
  edit: [asset: AssetSummary];
  batchEdit: [assets: AssetSummary[]];
}>();
const sortMode = ref<"default" | "day-desc" | "day-asc">("default");
const selectionMode = ref(false);
const selectedIds = ref<string[]>([]);
const confirmAction = ref<"single" | "selected" | "all">();
const singleDeleteId = ref<string>();
const displayedAssets = computed(() => {
  if (sortMode.value === "default") return props.assets;
  const direction = sortMode.value === "day-desc" ? -1 : 1;
  return [...props.assets].sort((left, right) => {
    const percentOrder = (left.dayProfitPercent - right.dayProfitPercent) * direction;
    if (percentOrder !== 0) return percentOrder;
    const amountOrder = (left.dayProfit - right.dayProfit) * direction;
    return amountOrder || left.name.localeCompare(right.name, "zh-CN");
  });
});
const allSelected = computed(
  () => props.assets.length > 0 && props.assets.every((asset) => selectedIds.value.includes(asset.id)),
);
const pendingDeleteIds = computed(() =>
  confirmAction.value === "all"
    ? props.assets.map((asset) => asset.id)
    : confirmAction.value === "single" && singleDeleteId.value
      ? [singleDeleteId.value]
      : selectedIds.value,
);
const selectedAssets = computed(() =>
  props.assets.filter((asset) => selectedIds.value.includes(asset.id)),
);

watch(
  () => props.assets.map((asset) => asset.id),
  (ids) => {
    selectedIds.value = selectedIds.value.filter((id) => ids.includes(id));
    if (!ids.length) selectionMode.value = false;
  },
);

function toggleSelectionMode() {
  selectionMode.value = !selectionMode.value;
  if (!selectionMode.value) selectedIds.value = [];
}

function cycleSort() {
  sortMode.value = sortMode.value === "day-desc" ? "day-asc" : "day-desc";
}

function toggleAsset(id: string, checked: boolean) {
  selectedIds.value = checked
    ? [...new Set([...selectedIds.value, id])]
    : selectedIds.value.filter((item) => item !== id);
}

function toggleAll(checked: boolean) {
  selectedIds.value = checked ? props.assets.map((asset) => asset.id) : [];
}

function confirmDelete() {
  const ids = [...pendingDeleteIds.value];
  if (!ids.length) return;
  emit("remove", ids);
  confirmAction.value = undefined;
  singleDeleteId.value = undefined;
  selectedIds.value = [];
}

function requestSingleDelete(id: string) {
  singleDeleteId.value = id;
  confirmAction.value = "single";
}

function timeLabel(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "--:--";
  const today = new Date();
  const sameDay = date.getFullYear() === today.getFullYear()
    && date.getMonth() === today.getMonth()
    && date.getDate() === today.getDate();
  return sameDay
    ? date.toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" })
    : `${date.getMonth() + 1}/${date.getDate()} ${date.toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" })}`;
}

function sourceLabel(provider: string) {
  if (provider.includes("新浪")) return "新浪";
  if (provider.includes("东方财富") || provider.includes("东财")) return "东财";
  if (provider.includes("持仓")) return "持仓估算";
  if (provider.includes("插件") || provider.includes("导入")) return "导入";
  if (provider.includes("手动")) return "手动";
  if (provider.includes("自动") || provider.includes("混合")) return "自动";
  return provider.length > 6 ? `${provider.slice(0, 5)}…` : provider;
}
</script>

<template>
  <section class="asset-list material-card">
    <header>
      <div>
        <span class="eyebrow">{{ eyebrow }}</span>
        <h3>{{ title }}</h3>
      </div>
      <div class="list-actions">
        <button v-if="assets.length" type="button" class="batch-button" data-liquid-glass @click="toggleSelectionMode">
          <svg v-if="!selectionMode" viewBox="0 0 24 24"><path d="M5 7h2M10 7h9M5 12h2M10 12h9M5 17h2M10 17h9" /></svg>
          <svg v-else viewBox="0 0 24 24"><path d="m7 7 10 10M17 7 7 17" /></svg>
          {{ selectionMode ? "结束批量" : "批量管理" }}
        </button>
        <button v-if="assets.length" type="button" :disabled="exporting" data-liquid-glass @click="$emit('export')">
          <svg viewBox="0 0 24 24"><path d="M12 4v10"/><path d="m8 10 4 4 4-4"/><path d="M5 17v2h14v-2"/></svg>
          {{ exportLabel }}
        </button>
        <button type="button" data-liquid-glass @click="$emit('manage')">
          <svg viewBox="0 0 24 24"><path d="M12 5v14M5 12h14" /></svg>
          添加 / 导入
        </button>
      </div>
    </header>

    <div v-if="selectionMode && assets.length" class="bulk-bar">
      <label class="select-all">
        <input type="checkbox" :checked="allSelected" @change="toggleAll(($event.target as HTMLInputElement).checked)" />
        <i><svg viewBox="0 0 24 24"><path d="m6 12 4 4 8-9" /></svg></i>
        <span>{{ allSelected ? "取消全选" : "全选当前列表" }}</span>
      </label>
      <span class="selected-count">已选 <strong>{{ selectedIds.length }}</strong> 项</span>
      <div>
        <button type="button" class="edit-selected" :disabled="!selectedIds.length || deleting" @click="$emit('batchEdit', selectedAssets)">批量修改</button>
        <button type="button" :disabled="!selectedIds.length || deleting" @click="confirmAction = 'selected'">删除已选</button>
        <button type="button" class="clear-button" :disabled="deleting" @click="confirmAction = 'all'">清空当前列表</button>
      </div>
    </div>

    <div v-if="assets.length" class="asset-columns" :class="{ selecting: selectionMode }">
      <span v-if="selectionMode" />
      <span class="column-name">基金</span>
      <span class="column-number">当前资产</span>
      <button
        v-if="sortable"
        type="button"
        class="column-sort column-number"
        :class="{ ascending: sortMode === 'day-asc', active: sortMode !== 'default' }"
        :aria-label="sortMode === 'default' ? '按当日盈亏百分比从高到低排序' : sortMode === 'day-asc' ? '当日盈亏百分比当前从低到高，点击切换为从高到低' : '当日盈亏百分比当前从高到低，点击切换为从低到高'"
        title="排序依据：当日盈亏百分比"
        @click="cycleSort"
      >
        <span>当日盈亏</span><i />
      </button>
      <span v-else class="column-number">当日盈亏</span>
      <span class="column-number">总盈亏</span>
      <span class="column-number">来源 / 更新</span>
      <span class="column-actions">操作</span>
    </div>

    <div v-if="assets.length" class="rows">
      <article v-for="asset in displayedAssets" :key="asset.id" class="asset-row" :class="{ selecting: selectionMode }">
        <label v-if="selectionMode" class="row-check" :aria-label="`选择 ${asset.name}`">
          <input type="checkbox" :checked="selectedIds.includes(asset.id)" @change="toggleAsset(asset.id, ($event.target as HTMLInputElement).checked)" />
          <i><svg viewBox="0 0 24 24"><path d="m6 12 4 4 8-9" /></svg></i>
        </label>
        <div class="asset-identity">
          <div>
            <strong>{{ asset.name }}</strong>
            <span>{{ asset.code ? `${asset.code} · ` : "" }}{{ asset.strategy }}</span>
          </div>
        </div>
        <div class="asset-metric asset-value">
          <MoneyValue :value="asset.currentValue" :private="private" />
        </div>
        <div class="asset-metric">
          <div>
            <MoneyValue
              :class="asset.dayProfit >= 0 ? 'profit' : 'loss'"
              :value="asset.dayProfit"
              :private="private"
              sign
            />
            <PercentValue :value="asset.dayProfitPercent" />
          </div>
        </div>
        <div class="asset-metric">
          <div v-if="asset.costKnown">
            <MoneyValue
              :class="asset.totalProfit >= 0 ? 'profit' : 'loss'"
              :value="asset.totalProfit"
              :private="private"
              sign
            />
            <PercentValue :value="asset.totalProfitPercent" />
          </div>
          <div v-else class="unknown-cost">
            <strong>—</strong><small>未录入成本</small>
          </div>
        </div>
        <div class="asset-source">
          <span :title="asset.provider">
            <svg viewBox="0 0 24 24"><path d="M7 7.5h8.5a4 4 0 0 1 0 8H13" /><path d="m10 12 3 3-3 3" /></svg>
            {{ sourceLabel(asset.provider) }}
          </span>
          <small>{{ timeLabel(asset.updatedAt) }}</small>
        </div>
        <div class="row-actions">
          <button
            type="button"
            class="row-action edit-action"
            :disabled="selectionMode || deleting"
            :aria-label="`编辑 ${asset.name}`"
            title="编辑资产"
            @click="$emit('edit', asset)"
          >
            <svg viewBox="0 0 24 24"><path d="M4 20h4l11-11-4-4L4 16z"/><path d="m13.5 6.5 4 4"/></svg>
          </button>
          <button
            type="button"
            class="row-action delete-action"
            :disabled="selectionMode || deleting"
            :aria-label="`删除 ${asset.name}`"
            title="删除资产"
            @click="requestSingleDelete(asset.id)"
          >
            <svg viewBox="0 0 24 24"><path d="M5 7h14"/><path d="M9 7V4h6v3"/><path d="m8 10 .7 9h6.6l.7-9"/><path d="M11 11v5M13 11v5"/></svg>
          </button>
        </div>
      </article>
    </div>

    <div v-else class="empty-state">
      <strong>{{ emptyTitle }}</strong>
      <span>{{ emptyDescription }}</span>
    </div>
  </section>
  <ConfirmDialog
    v-if="confirmAction"
    :title="confirmAction === 'all'
      ? `清空${title}？`
      : confirmAction === 'single'
        ? '删除这项资产？'
        : `删除 ${pendingDeleteIds.length} 项资产？`"
    :description="confirmAction === 'all'
      ? `将从本机永久删除当前列表中的 ${pendingDeleteIds.length} 项资产，此操作无法撤销。`
      : confirmAction === 'single'
        ? '该基金的份额、投入成本与本地持仓信息会一并永久删除，此操作无法撤销。'
        : '所选资产的份额、投入成本与本地持仓信息会一并永久删除，此操作无法撤销。'"
    :confirm-label="confirmAction === 'all' ? '确认清空' : '确认删除'"
    :busy="deleting"
    @close="confirmAction = undefined; singleDeleteId = undefined"
    @confirm="confirmDelete"
  />
</template>

<style scoped>
.asset-list {
  display: flex;
  flex-direction: column;
  min-height: 0;
  min-width: 0;
  padding: 17px 18px 8px;
  background: var(--glass-subtle);
  border: 1px solid var(--hairline);
  border-radius: var(--radius-lg);
}

header,
.asset-row,
.asset-identity,
.asset-metric > div {
  display: flex;
  align-items: center;
}

header {
  justify-content: space-between;
  margin-bottom: 10px;
}

.list-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.bulk-bar,
.bulk-bar > div,
.select-all,
.row-check,
.row-check i,
.select-all i {
  display: flex;
  align-items: center;
}

.bulk-bar {
  gap: 12px;
  padding: 8px 10px;
  margin: 2px 0 7px;
  font-size: var(--font-xs);
  color: var(--text-muted);
  background: var(--glass-subtle);
  border: 1px solid var(--hairline);
  border-radius: 11px;
}

.select-all { gap: 7px; cursor: pointer; }
.select-all input,
.row-check input { position: absolute; opacity: 0; }
.select-all i,
.row-check i {
  justify-content: center;
  width: 18px;
  height: 18px;
  color: transparent;
  border: 1px solid var(--hairline-strong);
  border-radius: 6px;
}
.select-all i svg,
.row-check i svg { width: 12px; height: 12px; fill: none; stroke: currentColor; stroke-width: 2.3; }
.select-all input:checked + i,
.row-check input:checked + i { color: white; background: var(--accent); border-color: var(--accent); }
.selected-count { margin-right: auto; }
.selected-count strong { color: var(--accent); }
.bulk-bar > div { gap: 6px; }
.bulk-bar button { padding: 5px 8px; font-size: var(--font-xs); color: var(--profit); cursor: pointer; background: var(--profit-soft); border: 1px solid color-mix(in srgb, var(--profit) 16%, transparent); border-radius: 8px; }
.bulk-bar button:disabled { cursor: default; opacity: .4; }
.bulk-bar .clear-button { color: white; background: var(--profit); }
.bulk-bar .edit-selected { color: var(--accent); background: var(--accent-soft); border-color: color-mix(in srgb, var(--accent) 16%, transparent); }

.eyebrow {
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

header button {
  display: inline-flex;
  gap: 6px;
  align-items: center;
  padding: 7px 10px;
  font-size: var(--font-sm);
  color: var(--text);
  cursor: pointer;
  background: var(--glass-subtle);
  border: 1px solid var(--hairline);
  border-radius: 10px;
}

header button:disabled {
  cursor: default;
  opacity: .5;
}

header button svg {
  width: 13px;
  height: 13px;
  fill: none;
  stroke: currentColor;
  stroke-width: 2;
  stroke-linecap: round;
}

.rows {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 2px;
  min-height: 0;
  min-width: 0;
  padding-top: 3px;
  overflow-x: hidden;
  overflow-y: auto;
  scrollbar-width: thin;
}

.asset-columns,
.asset-row {
  display: grid;
  grid-template-columns: minmax(184px, 1.5fr) minmax(96px, .76fr) repeat(2, minmax(110px, .9fr)) 82px 64px;
  gap: 10px;
}

.asset-columns {
  align-items: center;
  min-height: 34px;
  padding: 7px 12px;
  overflow: hidden;
  font-size: var(--font-xs);
  font-weight: 620;
  color: var(--text-muted);
  background: linear-gradient(90deg, color-mix(in srgb, var(--material-elevated) 68%, transparent), color-mix(in srgb, var(--glass-subtle) 42%, transparent));
  border-radius: 9px;
  box-shadow: 0 1px 0 color-mix(in srgb, var(--material-highlight) 58%, transparent) inset;
}

.asset-columns > .column-number {
  justify-self: end;
  text-align: right;
}

.column-actions {
  justify-self: center;
  text-align: center;
}

.column-sort {
  display: inline-flex;
  gap: 5px;
  align-items: center;
  justify-self: end;
  padding: 0;
  font-size: inherit;
  font-weight: inherit;
  color: inherit;
  cursor: pointer;
  overflow: visible;
  background: transparent;
  border: 0;
  border-radius: 0;
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
}

.column-sort::before,
.column-sort::after {
  display: none;
}

.column-sort i {
  width: 0;
  height: 0;
  border-right: 3.5px solid transparent;
  border-bottom: 5px solid currentColor;
  border-left: 3.5px solid transparent;
  opacity: .38;
  transform: rotate(180deg);
  transition: opacity 160ms ease, transform 200ms ease;
}

.column-sort.active { color: var(--text-strong); }
.column-sort.active i { opacity: .82; }
.column-sort.ascending i { transform: rotate(0); }

.asset-row {
  padding: 12px;
  border-radius: 12px;
  transition: background-color 160ms ease, box-shadow 160ms ease;
}

.asset-columns.selecting,
.asset-row.selecting {
  grid-template-columns: 26px minmax(164px, 1.4fr) minmax(88px, .7fr) repeat(2, minmax(102px, .86fr)) 74px 58px;
}

.row-check { justify-content: center; cursor: pointer; }

.asset-row:hover {
  background: var(--material-hover);
  box-shadow: 0 7px 18px color-mix(in srgb, var(--text-strong) 5%, transparent), 0 1px 0 var(--material-highlight) inset;
}

.asset-identity {
  min-width: 0;
}

.asset-identity > div,
.asset-metric,
.asset-source {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.asset-metric {
  align-items: flex-end;
  text-align: right;
}

.asset-identity strong {
  overflow: hidden;
  font-size: var(--font-sm);
  font-weight: 670;
  color: var(--text-strong);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.asset-identity span,
.asset-source span,
.asset-source small {
  margin-top: 4px;
  overflow: hidden;
  font-size: var(--font-xs);
  color: var(--text-muted);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.asset-metric > div {
  flex-direction: column;
  gap: 3px;
  align-items: flex-end;
  margin-top: 0;
  font-size: var(--font-sm);
}

.asset-metric > .unknown-cost {
  gap: 3px;
  color: var(--text-muted);
}

.unknown-cost strong {
  font-size: var(--font-sm);
  font-weight: 650;
}

.unknown-cost small {
  font-size: var(--font-xs);
  color: var(--text-muted);
}

.asset-value > :last-child {
  margin-top: 0;
  font-size: var(--font-sm);
  color: var(--text-strong);
}

.asset-source {
  align-items: flex-end;
  justify-content: center;
  gap: 5px;
}

.asset-source > span {
  display: inline-flex;
  gap: 4px;
  align-items: center;
  max-width: 100%;
  padding: 4px 6px;
  margin: 0;
  color: var(--text);
  background: var(--glass-subtle);
  border: 1px solid var(--hairline);
  border-radius: 7px;
}

.asset-source svg {
  flex: 0 0 10px;
  width: 10px;
  height: 10px;
  fill: none;
  stroke: var(--accent);
  stroke-width: 1.9;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.asset-source small {
  margin: 0;
}

.row-actions {
  display: flex;
  gap: 5px;
  align-items: center;
  justify-content: center;
}

.row-action {
  display: grid;
  flex: 0 0 27px;
  width: 27px;
  height: 27px;
  padding: 0;
  color: var(--text-muted);
  cursor: pointer;
  background: color-mix(in srgb, var(--material-elevated) 70%, transparent);
  border: 1px solid var(--hairline);
  border-radius: 8px;
  box-shadow: 0 1px 0 var(--material-highlight) inset;
  place-items: center;
  transition: color 150ms ease, background-color 150ms ease, border-color 150ms ease;
  -webkit-backdrop-filter: none;
  backdrop-filter: none;
}

.row-action::before,
.row-action::after {
  display: none;
}

.row-action svg {
  width: 13px;
  height: 13px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.8;
  stroke-linecap: round;
  stroke-linejoin: round;
  pointer-events: none;
}

.row-action:hover:not(:disabled),
.row-action:focus-visible {
  color: var(--accent);
  background: var(--accent-soft);
  border-color: color-mix(in srgb, var(--accent) 28%, var(--hairline));
}

.row-action.delete-action:hover:not(:disabled),
.row-action.delete-action:focus-visible {
  color: var(--profit);
  background: var(--profit-soft);
  border-color: color-mix(in srgb, var(--profit) 28%, var(--hairline));
}

.row-action:disabled {
  cursor: default;
  opacity: .36;
}

.empty-state {
  display: grid;
  min-height: 150px;
  color: var(--text-muted);
  text-align: center;
  place-content: center;
}

.empty-state strong {
  margin-bottom: 7px;
  color: var(--text-strong);
}

.empty-state span {
  max-width: 360px;
  font-size: var(--font-sm);
}

@media (max-width: 1060px) {
  .asset-columns,
  .asset-row {
    grid-template-columns: minmax(148px, 1.34fr) minmax(82px, .68fr) repeat(2, minmax(92px, .86fr)) 68px 60px;
    gap: 7px;
  }

  .asset-columns.selecting,
  .asset-row.selecting {
    grid-template-columns: 24px minmax(136px, 1.25fr) minmax(76px, .64fr) repeat(2, minmax(86px, .82fr)) 64px 56px;
  }
}
</style>
