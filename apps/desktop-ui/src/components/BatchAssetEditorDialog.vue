<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type {
  AssetSummary,
  PositionInput,
  PositionUpdateFailure,
} from "@/types/contracts";

const props = withDefaults(defineProps<{
  assets: AssetSummary[];
  saving?: boolean;
  failures?: PositionUpdateFailure[];
  successCount?: number;
}>(), {
  saving: false,
  failures: () => [],
  successCount: 0,
});

const emit = defineEmits<{
  close: [];
  save: [inputs: PositionInput[]];
}>();

interface EditableAsset {
  id: string;
  kind: AssetSummary["kind"];
  code?: string;
  name: string;
  holding: string;
  totalCost: string;
  strategy: string;
  provider: string;
  dayPercent: string;
}

const rows = ref<EditableAsset[]>(props.assets.map(toEditable));
const failureMap = computed(() => new Map(props.failures.map((item) => [item.id, item.message])));

watch(
  () => props.assets,
  (assets) => {
    if (!props.failures.length && !props.successCount) rows.value = assets.map(toEditable);
  },
);

watch(
  () => props.failures,
  (failures) => {
    if (!failures.length) return;
    const failedIds = new Set(failures.map((item) => item.id));
    rows.value = rows.value.filter((row) => failedIds.has(row.id));
  },
);

function toEditable(asset: AssetSummary): EditableAsset {
  return {
    id: asset.id,
    kind: asset.kind,
    code: asset.code,
    name: asset.name,
    holding: asset.kind === "fund" ? asset.units : String(asset.currentValue),
    totalCost: asset.totalCost,
    strategy: asset.strategy,
    provider: asset.provider,
    dayPercent: String(asset.dayProfitPercent),
  };
}

function normalized(value: string) {
  return value.replaceAll(",", "").trim();
}

function friendlyError(message: string) {
  if (message.includes("name is required")) return "资产名称不能为空";
  if (message.includes("units is not a decimal")) return "持有份额不是有效数字";
  if (message.includes("units cannot be negative")) return "持有份额不能为负数";
  if (message.includes("totalCost is not a decimal")) return "累计成本不是有效数字";
  if (message.includes("totalCost cannot be negative")) return "累计成本不能为负数";
  if (message.includes("manualValue is not a decimal")) return "当前资产不是有效数字";
  if (message.includes("manualValue cannot be negative")) return "当前资产不能为负数";
  if (message.includes("manualDayPercent is not a decimal")) return "当日涨跌幅不是有效数字";
  return message.replace(/^invalid input:\s*/i, "");
}

function submit() {
  if (props.saving || !rows.value.length) return;
  emit("save", rows.value.map((row) => ({
    id: row.id,
    kind: row.kind,
    code: row.code,
    name: row.name.trim(),
    units: row.kind === "fund" ? normalized(row.holding || "0") : undefined,
    totalCost: normalized(row.totalCost || "0"),
    manualValue: row.kind === "fund" ? undefined : normalized(row.holding || "0"),
    manualDayPercent: row.kind === "fund" ? undefined : normalized(row.dayPercent || "0"),
    provider: row.kind === "fund" ? "自动估值" : row.provider || "手动录入",
    strategy: row.strategy.trim() || (row.kind === "fund" ? "公募基金" : "现金管理"),
  })));
}
</script>

<template>
  <div class="dialog-backdrop no-drag" @mousedown.self="$emit('close')">
    <section class="batch-editor material-panel" role="dialog" aria-modal="true" aria-labelledby="batch-editor-title">
      <header class="dialog-header">
        <div>
          <span>BATCH HOLDINGS</span>
          <h2 id="batch-editor-title">批量修改持仓</h2>
          <p>每项独立保存；有错误的项目会留下，其余修改照常生效。</p>
        </div>
        <button class="close-button" type="button" aria-label="关闭" @click="$emit('close')">
          <svg viewBox="0 0 24 24"><path d="m7 7 10 10M17 7 7 17" /></svg>
        </button>
      </header>

      <div v-if="successCount || failures.length" class="batch-feedback" :class="{ warning: failures.length }">
        <i />
        <span v-if="failures.length">已保存 {{ successCount }} 项，{{ failures.length }} 项需要修正</span>
        <span v-else>全部修改已保存</span>
      </div>

      <div class="edit-columns" aria-hidden="true">
        <span>基金 / 资产</span>
        <span>份额 / 当前资产</span>
        <span>累计成本</span>
        <span>行业 / 分类</span>
      </div>

      <div class="edit-rows">
        <article v-for="row in rows" :key="row.id" class="edit-row" :class="{ invalid: failureMap.has(row.id) }">
          <div class="identity">
            <input v-model.trim="row.name" :aria-label="`${row.name}名称`" />
            <small>{{ row.code || (row.kind === 'cash' ? '现金管理' : '手动资产') }}</small>
          </div>
          <label>
            <span>{{ row.kind === "fund" ? "份额" : "当前资产" }}</span>
            <input v-model.trim="row.holding" inputmode="decimal" :aria-label="`${row.name}${row.kind === 'fund' ? '份额' : '当前资产'}`" />
          </label>
          <label>
            <span>成本</span>
            <input v-model.trim="row.totalCost" inputmode="decimal" :aria-label="`${row.name}累计成本`" />
          </label>
          <label>
            <span>分类</span>
            <input v-model.trim="row.strategy" :aria-label="`${row.name}行业分类`" />
          </label>
          <p v-if="failureMap.has(row.id)" class="row-error">{{ friendlyError(failureMap.get(row.id) || "保存失败") }}</p>
        </article>
      </div>

      <footer>
        <span class="local-note"><i />修改仅保存在本机</span>
        <button class="secondary" type="button" @click="$emit('close')">取消</button>
        <button class="primary" type="button" :disabled="saving || !rows.length" @click="submit">
          {{ saving ? "逐项保存中…" : `保存 ${rows.length} 项修改` }}
        </button>
      </footer>
    </section>
  </div>
</template>

<style scoped>
.dialog-backdrop {
  position: fixed;
  z-index: 32;
  display: grid;
  padding: 22px;
  background: rgba(12, 17, 29, .24);
  inset: 0;
  place-items: center;
  backdrop-filter: blur(14px) saturate(120%);
}

.batch-editor {
  display: flex;
  flex-direction: column;
  width: min(920px, calc(100vw - 38px));
  max-height: min(720px, calc(100vh - 38px));
  padding: 22px;
  background: var(--glass-strong);
  border-radius: 26px;
  box-shadow: 0 34px 90px rgba(22, 29, 48, .28);
}

.dialog-header,
.dialog-header > div,
footer,
.local-note,
.batch-feedback,
.identity,
.edit-row label {
  display: flex;
}

.dialog-header { align-items: flex-start; justify-content: space-between; }
.dialog-header > div { flex-direction: column; }
.dialog-header span { font-size: var(--font-xs); color: var(--text-muted); letter-spacing: .09em; }
h2 { margin: 4px 0 0; font-size: var(--font-lg); color: var(--text-strong); }
.dialog-header p { margin: 6px 0 0; font-size: var(--font-xs); color: var(--text-muted); }

button { color: var(--text); cursor: pointer; background: var(--glass-subtle); border: 1px solid var(--hairline); }
button:disabled { cursor: default; opacity: .45; }
button svg { width: 17px; height: 17px; fill: none; stroke: currentColor; stroke-width: 1.8; stroke-linecap: round; stroke-linejoin: round; }
.close-button { display: grid; width: 31px; height: 31px; padding: 0; border-radius: 10px; place-items: center; }

.batch-feedback {
  gap: 8px;
  align-items: center;
  padding: 8px 11px;
  margin-top: 14px;
  font-size: var(--font-xs);
  color: var(--loss);
  background: var(--loss-soft);
  border-radius: 10px;
}
.batch-feedback.warning { color: var(--warning); background: color-mix(in srgb, var(--warning) 11%, transparent); }
.batch-feedback i,
.local-note i { width: 6px; height: 6px; background: currentColor; border-radius: 99px; }

.edit-columns,
.edit-row {
  display: grid;
  grid-template-columns: minmax(190px, 1.35fr) minmax(125px, .86fr) minmax(125px, .86fr) minmax(150px, 1fr);
  gap: 10px;
}
.edit-columns {
  padding: 9px 12px 7px;
  margin-top: 12px;
  font-size: 10px;
  font-weight: 620;
  color: var(--text-muted);
}
.edit-rows {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 5px;
  min-height: 110px;
  overflow: auto;
  scrollbar-width: thin;
}
.edit-row {
  position: relative;
  align-items: center;
  padding: 10px 12px;
  background: color-mix(in srgb, var(--glass-subtle) 72%, transparent);
  border: 1px solid transparent;
  border-radius: 13px;
  transition: background 180ms ease, border-color 180ms ease, transform 180ms ease;
}
.edit-row:hover { background: var(--material-hover); transform: translateY(-1px); }
.edit-row.invalid { padding-bottom: 28px; border-color: color-mix(in srgb, var(--warning) 34%, transparent); }
.identity,
.edit-row label { flex-direction: column; min-width: 0; }
.identity small,
.edit-row label span { margin: 0 0 5px; font-size: 9px; color: var(--text-muted); }
.identity small { order: -1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
input {
  min-width: 0;
  width: 100%;
  padding: 8px 9px;
  font: inherit;
  font-size: var(--font-sm);
  color: var(--text-strong);
  outline: none;
  background: color-mix(in srgb, var(--material-elevated) 68%, transparent);
  border: 1px solid var(--hairline);
  border-radius: 9px;
  user-select: text;
}
input:focus { border-color: color-mix(in srgb, var(--accent) 45%, transparent); box-shadow: 0 0 0 3px var(--accent-soft); }
.row-error { position: absolute; bottom: 7px; left: 12px; margin: 0; font-size: 9px; color: var(--warning); }

footer { gap: 8px; align-items: center; justify-content: flex-end; padding-top: 15px; margin-top: 10px; border-top: 1px solid var(--hairline); }
footer button { padding: 9px 14px; font-size: 11px; border-radius: 10px; }
footer .primary { min-width: 112px; color: white; background: var(--accent); border-color: transparent; }
.local-note { gap: 7px; align-items: center; margin-right: auto; font-size: 9px; color: var(--text-muted); }
.local-note i { color: var(--loss); }

@media (max-width: 760px) {
  .dialog-backdrop { padding: 10px; }
  .batch-editor { width: 100%; max-height: calc(100vh - 20px); padding: 16px; }
  .edit-columns { display: none; }
  .edit-rows { margin-top: 12px; }
  .edit-row { grid-template-columns: 1fr 1fr; }
  .identity { grid-column: 1 / -1; }
}
</style>
