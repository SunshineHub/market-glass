<script setup lang="ts">
import { computed, onBeforeUnmount, reactive, ref, watch } from "vue";
import { lookupFund } from "@/ipc/client";
import {
  parseFundConfig,
  type FundImportDraft,
} from "@/features/import/fundConfig";
import type { AssetSummary, FundMetadata, PositionInput } from "@/types/contracts";
import { useDelayedBusy } from "@/features/ui/useDelayedBusy";

const props = withDefaults(defineProps<{ saving?: boolean; asset?: AssetSummary }>(), { saving: false });
const savingVisible = useDelayedBusy(() => props.saving);
const emit = defineEmits<{
  close: [];
  save: [input: PositionInput];
  import: [inputs: PositionInput[]];
}>();

type Mode = "single" | "config";
const mode = ref<Mode>("single");
const editing = computed(() => Boolean(props.asset));
const form = reactive({
  kind: (props.asset?.kind === "cash" ? "cash" : "fund") as "fund" | "cash",
  code: props.asset?.code ?? "",
  name: props.asset?.name ?? "",
  units: props.asset?.units ?? "",
  unitCost: "",
  totalCost: props.asset?.totalCost ?? "",
  manualValue: props.asset?.kind === "cash" ? String(props.asset.currentValue) : "",
  manualDayPercent: props.asset?.kind === "cash" ? String(props.asset.dayProfitPercent) : "",
  provider: props.asset?.kind === "cash" ? props.asset.provider : "自动估值",
  strategy: props.asset?.strategy ?? "",
});
const submitted = ref(false);
const drafts = ref<FundImportDraft[]>([]);
const importError = ref("");
const fileName = ref("");
const fundMetadata = ref<FundMetadata>();
const lookupPhase = ref<"idle" | "loading" | "success" | "not-found" | "error">("idle");
let lookupTimer: number | undefined;
let lookupSequence = 0;
let lastAutoName = "";
let lastAutoStrategy = "";

const isFund = computed(() => form.kind === "fund");
const automaticInvestmentCost = computed(() => {
  if (!isFund.value || form.unitCost.trim() === "") return "";
  const units = Number(normalized(form.units || "0"));
  const unitCost = Number(normalized(form.unitCost));
  if (!Number.isFinite(units) || !Number.isFinite(unitCost) || units < 0 || unitCost < 0) return "";
  return (units * unitCost).toFixed(2);
});
const currentAverageUnitCost = computed(() => {
  if (!editing.value || !isFund.value) return "";
  const units = Number(normalized(props.asset?.units || "0"));
  const totalCost = Number(normalized(props.asset?.totalCost || "0"));
  if (!Number.isFinite(units) || !Number.isFinite(totalCost) || units <= 0 || totalCost <= 0) return "";
  return (totalCost / units).toFixed(4);
});
const metadataBadges = computed(() => {
  const metadata = fundMetadata.value;
  if (!metadata) return [];
  const badges = [metadata.company, metadata.fundType, metadata.indexName];
  if (metadata.latestNav) {
    const date = metadata.navDate ? ` · ${metadata.navDate.slice(5)}` : "";
    badges.push(`净值 ${metadata.latestNav}${date}`);
  }
  return badges.filter((item): item is string => Boolean(item));
});
const valid = computed(() => {
  if (!form.name.trim()) return false;
  if (!optionalNonNegative(form.totalCost)) return false;
  if (isFund.value) {
    return /^\d{6}$/.test(form.code)
      && optionalNonNegative(form.units)
      && optionalNonNegative(form.unitCost);
  }
  return optionalNonNegative(form.manualValue) && optionalNumber(form.manualDayPercent);
});
const selectedDrafts = computed(() => drafts.value.filter((draft) => draft.selected));
const invalidSelected = computed(() => selectedDrafts.value.some((draft) => !draftValid(draft)));
const canImport = computed(() => selectedDrafts.value.length > 0 && !invalidSelected.value && !props.saving);
const validCount = computed(() => drafts.value.filter((draft) => draftValid(draft)).length);
const missingCostCount = computed(() => selectedDrafts.value.filter((draft) => Number(normalized(draft.totalCost || "0")) === 0).length);

function isNumber(value: string) {
  return value.trim() !== "" && Number.isFinite(Number(value.replaceAll(",", "")));
}

function optionalNumber(value: string) {
  return value.trim() === "" || isNumber(value);
}

function optionalNonNegative(value: string) {
  return value.trim() === "" || (isNumber(value) && Number(value.replaceAll(",", "")) >= 0);
}

function draftValid(draft: FundImportDraft) {
  return /^\d{6}$/.test(draft.code)
    && Boolean(draft.name.trim())
    && optionalNonNegative(draft.units)
    && optionalNonNegative(draft.totalCost);
}

function submitSingle() {
  submitted.value = true;
  if (!valid.value || props.saving) return;
  emit("save", {
    id: props.asset?.id,
    kind: form.kind,
    code: isFund.value ? form.code : undefined,
    name: form.name,
    units: isFund.value ? normalized(form.units || "0") : undefined,
    unitCost: isFund.value && form.unitCost.trim()
      ? normalized(form.unitCost)
      : undefined,
    totalCost: normalized(automaticInvestmentCost.value || form.totalCost || "0"),
    manualValue: isFund.value ? undefined : normalized(form.manualValue || "0"),
    manualDayPercent: isFund.value ? undefined : normalized(form.manualDayPercent || "0"),
    provider: isFund.value ? "自动估值" : form.provider || "手动录入",
    strategy: form.strategy || (isFund.value ? "公募基金" : "现金管理"),
  });
}

function normalized(value: string) {
  return value.replaceAll(",", "").trim();
}

function submitImport() {
  if (!canImport.value) return;
  emit("import", selectedDrafts.value.map((draft) => ({
    kind: "fund",
    code: draft.code,
    name: draft.name.trim(),
    units: normalized(draft.units || "0"),
    totalCost: normalized(draft.totalCost || "0"),
    provider: "插件配置导入",
    strategy: draft.strategy.trim() || "公募基金",
  })));
}

async function readConfig(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  importError.value = "";
  fileName.value = file.name;
  try {
    drafts.value = parseFundConfig(await file.text());
  } catch (error) {
    drafts.value = [];
    importError.value = error instanceof Error ? error.message : "配置解析失败";
  } finally {
    input.value = "";
  }
}

function removeDraft(key: string) {
  drafts.value = drafts.value.filter((draft) => draft.key !== key);
}

function resetLookup() {
  lookupSequence += 1;
  fundMetadata.value = undefined;
  lookupPhase.value = "idle";
  if (lookupTimer !== undefined) window.clearTimeout(lookupTimer);
  lookupTimer = undefined;
}

async function performFundLookup(code: string, sequence: number) {
  lookupPhase.value = "loading";
  try {
    const metadata = await lookupFund(code);
    if (sequence !== lookupSequence || form.code !== code || !isFund.value) return;
    if (!metadata) {
      fundMetadata.value = undefined;
      lookupPhase.value = "not-found";
      return;
    }
    fundMetadata.value = metadata;
    lookupPhase.value = "success";

    if (!form.name.trim() || form.name === lastAutoName) {
      form.name = metadata.name;
    }
    lastAutoName = metadata.name;

    const strategy = metadata.industry || metadata.indexName || metadata.fundType || "";
    if (strategy && (!form.strategy.trim() || form.strategy === lastAutoStrategy)) {
      form.strategy = strategy;
    }
    lastAutoStrategy = strategy;
  } catch {
    if (sequence !== lookupSequence || form.code !== code || !isFund.value) return;
    fundMetadata.value = undefined;
    lookupPhase.value = "error";
  }
}

watch(
  [() => form.code, () => form.kind],
  ([rawCode, kind]) => {
    if (lastAutoName && form.name === lastAutoName) form.name = "";
    if (lastAutoStrategy && form.strategy === lastAutoStrategy) form.strategy = "";
    if (kind !== "fund") {
      resetLookup();
      return;
    }
    const code = rawCode.replace(/\D/g, "").slice(0, 6);
    if (code !== rawCode) {
      form.code = code;
      return;
    }
    resetLookup();
    if (!/^\d{6}$/.test(code)) return;
    const sequence = lookupSequence;
    lookupTimer = window.setTimeout(() => {
      lookupTimer = undefined;
      void performFundLookup(code, sequence);
    }, 360);
  },
  { immediate: true },
);

watch(automaticInvestmentCost, (value) => {
  if (value) form.totalCost = value;
});

onBeforeUnmount(resetLookup);
</script>

<template>
  <div class="dialog-backdrop no-drag" @mousedown.self="$emit('close')">
    <section class="editor material-panel" role="dialog" aria-modal="true" aria-labelledby="asset-editor-title">
      <header class="dialog-header">
        <div>
          <span>资产管理</span>
          <h2 id="asset-editor-title">{{ editing ? "编辑资产" : "添加与导入资产" }}</h2>
        </div>
        <button class="close-button" type="button" aria-label="关闭" @click="$emit('close')">
          <svg viewBox="0 0 24 24"><path d="m7 7 10 10M17 7 7 17" /></svg>
        </button>
      </header>

      <div v-if="!editing" class="mode-tabs" :class="{ 'config-active': mode === 'config' }" role="tablist" aria-label="资产录入方式">
        <button type="button" role="tab" :aria-selected="mode === 'single'" :class="{ active: mode === 'single' }" @click="mode = 'single'">
          <svg viewBox="0 0 24 24"><path d="M12 5v14M5 12h14" /></svg><span>单项添加</span>
        </button>
        <button type="button" role="tab" :aria-selected="mode === 'config'" :class="{ active: mode === 'config' }" @click="mode = 'config'">
          <svg viewBox="0 0 24 24"><path d="M7 3h8l4 4v14H7z" /><path d="M15 3v5h5M10 13h6M10 17h6" /></svg><span>基金配置导入</span>
        </button>
      </div>

      <form v-if="mode === 'single'" class="single-form" :class="{ editing }" @submit.prevent="submitSingle">
        <div class="kind-tabs" :class="{ 'cash-active': form.kind === 'cash' }">
          <button v-for="option in ([['fund', '公募基金'], ['cash', '现金管理']] as const)" :key="option[0]" type="button" :disabled="editing" :class="{ active: form.kind === option[0] }" @click="form.kind = option[0]">
            {{ option[1] }}
          </button>
        </div>

        <div v-if="isFund" class="field-row">
          <label class="fund-code-field">
            <span>基金代码</span>
            <div class="code-input">
              <input v-model.trim="form.code" :disabled="editing" maxlength="6" inputmode="numeric" placeholder="例如 005827" />
              <i v-if="lookupPhase === 'loading'" class="lookup-spinner" aria-label="正在查询基金资料" />
              <svg v-else-if="lookupPhase === 'success'" class="lookup-success" viewBox="0 0 24 24" aria-label="基金资料已查询"><path d="m6 12 4 4 8-9" /></svg>
            </div>
          </label>
          <label><span>持有份额（可选）</span><input v-model.trim="form.units" inputmode="decimal" placeholder="默认 0，仅观察行情" /></label>
        </div>
        <div v-if="isFund && lookupPhase !== 'idle' && lookupPhase !== 'loading'" class="lookup-result" :class="{ warning: lookupPhase !== 'success' }" aria-live="polite">
          <template v-if="lookupPhase === 'success'">
            <span class="lookup-source">{{ fundMetadata?.provider }}</span>
            <span v-for="badge in metadataBadges" :key="badge">{{ badge }}</span>
            <small>已回填名称{{ fundMetadata?.industry ? '与行业标签' : '' }}</small>
          </template>
          <template v-else-if="lookupPhase === 'not-found'">没有查到这个基金，请核对代码；也可以继续手动填写。</template>
          <template v-else>基金资料暂时查询失败，不影响手动录入和保存。</template>
        </div>
        <label><span>资产名称</span><input v-model.trim="form.name" :placeholder="isFund ? '基金简称' : '例如：现金管理'" /></label>
        <div v-if="isFund" class="field-row cost-row">
          <label>
            <span>单位持仓成本（可选）</span>
            <input
              v-model.trim="form.unitCost"
              inputmode="decimal"
              :placeholder="currentAverageUnitCost ? `当前折算约 ${currentAverageUnitCost}` : '例如 1.2345'"
            />
          </label>
          <label>
            <span>{{ automaticInvestmentCost ? (editing ? "累计投入成本（自动计算）" : "本次投入成本（自动计算）") : (editing ? "累计投入 / 成本（可直接修改）" : "本次投入成本（可选）") }}</span>
            <div class="calculated-input" :class="{ calculated: automaticInvestmentCost }">
              <input
                v-model.trim="form.totalCost"
                inputmode="decimal"
                :readonly="Boolean(automaticInvestmentCost)"
                placeholder="默认 0，也可直接输入"
              />
              <small v-if="automaticInvestmentCost">份额 × 单位成本</small>
            </div>
          </label>
        </div>
        <div v-else class="field-row">
          <label><span>累计投入 / 成本（可选）</span><input v-model.trim="form.totalCost" inputmode="decimal" placeholder="默认 0" /></label>
          <label><span>当前资产（可选）</span><input v-model.trim="form.manualValue" inputmode="decimal" placeholder="默认 0" /></label>
        </div>
        <label v-if="isFund"><span>行业 / 策略标签</span><input v-model.trim="form.strategy" placeholder="例如：科技、医药、红利" /></label>
        <div v-if="!isFund" class="field-row">
          <label><span>当日盈亏百分比</span><input v-model.trim="form.manualDayPercent" inputmode="decimal" placeholder="例如 0.35" /></label>
          <label><span>数据来源</span><input v-model.trim="form.provider" placeholder="例如：手动录入、银行" /></label>
        </div>
        <p v-if="submitted && !valid" class="form-error">请填写名称并检查基金代码和数字格式；份额、单位成本和投入成本均可留空。</p>
        <p v-else class="form-note">{{ editing ? (isFund ? '可直接修改累计成本；输入单位持仓成本后，将按当前份额重新计算累计投入。' : '保存后会覆盖当前现金资产、累计成本和分类。') : isFund ? '填写单位持仓成本后，本次投入按“份额 × 单位成本”计算；同代码再次新增会累计每一笔份额与投入成本。' : '现金金额可为 0，并保留为本地观察资产。' }}</p>
      </form>

      <div v-else class="import-workspace">
        <section class="drop-zone material-card" :class="{ compact: drafts.length }">
          <div class="drop-copy">
            <svg viewBox="0 0 24 24"><path d="M7 3h8l4 4v14H7z" /><path d="M15 3v5h5M12 11v6M9 14h6" /></svg>
            <div>
              <strong>导入自选基金助手配置</strong>
              <span>支持 3.x 导出的 JSON；累计成本按“份额 × 单份成本价”换算</span>
            </div>
          </div>
          <label class="file-button">
            {{ drafts.length ? '重新选择' : '选择文件' }}
            <input type="file" accept="application/json,.json" @change="readConfig" />
          </label>
        </section>

        <p v-if="importError" class="form-error import-error">{{ importError }}</p>

        <section v-if="drafts.length" class="preview-section">
          <header>
            <div><strong>导入预览</strong><span>{{ fileName }} · 识别 {{ drafts.length }} 项 · 可直接入库 {{ validCount }} 项</span></div>
            <small>已勾选 {{ selectedDrafts.length }} 项<span v-if="missingCostCount"> · {{ missingCostCount }} 项未录入成本</span></small>
          </header>
          <div class="preview-table">
            <div class="preview-head"><span /><span>基金代码 / 名称</span><span>持有份额</span><span>累计成本</span><span>分组</span><span /></div>
            <div v-for="draft in drafts" :key="draft.key" class="preview-row" :class="{ invalid: draft.selected && !draftValid(draft) }">
              <label class="check"><input v-model="draft.selected" type="checkbox" /><i /></label>
              <div class="identity-fields"><input v-model.trim="draft.code" maxlength="6" inputmode="numeric" aria-label="基金代码" /><input v-model.trim="draft.name" aria-label="基金名称" /></div>
              <input v-model.trim="draft.units" inputmode="decimal" aria-label="持有份额" placeholder="可选，默认 0" />
              <input v-model.trim="draft.totalCost" inputmode="decimal" aria-label="累计成本" placeholder="0" />
              <div class="strategy-field"><input v-model.trim="draft.strategy" aria-label="分组" /><small v-if="draft.warning">{{ draft.warning }}</small></div>
              <button class="remove-row" type="button" aria-label="移除" @click="removeDraft(draft.key)"><svg viewBox="0 0 24 24"><path d="M6 12h12" /></svg></button>
            </div>
          </div>
          <p v-if="invalidSelected" class="form-error">红色行仍缺少有效的 6 位代码、名称，或包含无效数字，请修正后再导入。</p>
        </section>
      </div>

      <footer>
        <span v-if="mode !== 'single'" class="privacy-note"><i />配置仅在本机处理</span>
        <button class="secondary" type="button" @click="$emit('close')">取消</button>
        <button v-if="mode === 'single'" class="primary" type="button" :disabled="props.saving" @click="submitSingle">
          <i v-if="savingVisible" class="button-spinner" />
          <span>{{ savingVisible ? '正在保存' : editing ? '保存修改' : '保存资产' }}</span>
        </button>
        <button v-else class="primary" type="button" :disabled="!canImport" @click="submitImport">
          <i v-if="savingVisible" class="button-spinner" />
          <span>{{ savingVisible ? '正在导入' : `导入 ${selectedDrafts.length} 项` }}</span>
        </button>
      </footer>
    </section>
  </div>
</template>

<style scoped>
.dialog-backdrop { position: fixed; z-index: 30; display: grid; padding: 24px; background: rgba(12, 17, 29, .24); inset: 0; place-items: center; backdrop-filter: blur(14px) saturate(120%); }
.editor { display: flex; flex-direction: column; width: min(880px, calc(100vw - 40px)); max-height: min(680px, calc(100vh - 40px)); padding: 22px; overflow: hidden; background: var(--glass-strong); border-radius: 26px; box-shadow: 0 34px 90px rgba(22, 29, 48, .28); }
.dialog-header, .field-row, footer, .kind-tabs, .mode-tabs, .drop-zone, .drop-copy, .progress-card > div, .preview-section > header { display: flex; }
.dialog-header { align-items: flex-start; justify-content: space-between; }
.dialog-header span { font-size: var(--font-xs); color: var(--text-muted); letter-spacing: .08em; }
h2 { margin: 4px 0 0; font-size: var(--font-lg); color: var(--text-strong); }
button { color: var(--text); cursor: pointer; background: var(--glass-subtle); border: 1px solid var(--hairline); }
button:disabled { cursor: default; opacity: .45; }
button svg, .drop-copy > svg { width: 18px; height: 18px; fill: none; stroke: currentColor; stroke-width: 1.8; stroke-linecap: round; stroke-linejoin: round; }
.close-button { display: grid; width: 31px; height: 31px; padding: 0; border-radius: 10px; place-items: center; }
.mode-tabs { position: relative; align-self: center; width: min(380px, 100%); padding: 0; margin: 18px 0; overflow: visible; background: transparent; border: 0; border-bottom: 1px solid var(--hairline); border-radius: 0; box-shadow: none; }
.mode-tabs::before { position: absolute; bottom: -1px; left: 0; z-index: 2; width: 50%; height: 2px; content: ""; background: linear-gradient(90deg, transparent 5%, color-mix(in srgb, var(--accent) 78%, white) 28% 72%, transparent 95%); border: 0; border-radius: 99px; box-shadow: 0 3px 10px color-mix(in srgb, var(--accent) 32%, transparent); transform: translateX(0); transition: transform 360ms cubic-bezier(.2,.9,.22,1); }
.mode-tabs.config-active::before { transform: translateX(100%); }
.mode-tabs button { position: relative; z-index: 1; display: flex; flex: 1; gap: 7px; align-items: center; justify-content: center; padding: 9px 14px 11px; font-size: var(--font-sm); background: transparent; border-color: transparent; border-radius: 0; transition: color 220ms ease; }
.mode-tabs button.active { color: var(--accent); background: transparent; border-color: transparent; box-shadow: none; }
.single-form, form, label { display: flex; flex-direction: column; }
.single-form { flex: 1; gap: 13px; width: min(620px, 100%); min-height: 0; padding: 0 5px 4px; overflow-x: hidden; overflow-y: auto; align-self: center; }
.single-form.editing { margin-top: 18px; }
.field-row { gap: 12px; }.field-row label { flex: 1; }
.kind-tabs { position: relative; padding: 0; overflow: visible; background: transparent; border: 0; border-bottom: 1px solid var(--hairline); border-radius: 0; }
.kind-tabs::before { position: absolute; bottom: -1px; left: 0; width: 50%; height: 2px; content: ""; background: linear-gradient(90deg, transparent 10%, var(--accent) 28% 72%, transparent 90%); border: 0; border-radius: 99px; transform: translateX(0); transition: transform 340ms cubic-bezier(.2,.9,.22,1); }
.kind-tabs.cash-active::before { transform: translateX(100%); }
.kind-tabs button { position: relative; z-index: 1; flex: 1; padding: 8px 8px 10px; font-size: var(--font-sm); background: transparent; border-color: transparent; border-radius: 0; transition: color 200ms ease; }
.kind-tabs button.active { color: var(--accent); background: transparent; border-color: transparent; }
label > span { margin-bottom: 6px; font-size: var(--font-xs); color: var(--text-muted); }
input { min-width: 0; padding: 9px 10px; font: inherit; font-size: var(--font-sm); color: var(--text-strong); outline: none; background: color-mix(in srgb, var(--glass-subtle) 86%, transparent); border: 1px solid var(--hairline); border-radius: 9px; user-select: text; }
input:focus { border-color: color-mix(in srgb, var(--accent) 45%, transparent); box-shadow: 0 0 0 3px var(--accent-soft); }
input[readonly] { color: var(--text); cursor: default; background: color-mix(in srgb, var(--accent-soft) 32%, var(--glass-subtle)); }
.calculated-input { position: relative; }
.calculated-input input { width: 100%; }
.calculated-input.calculated input { padding-right: 104px; }
.calculated-input small { position: absolute; top: 50%; right: 10px; font-size: var(--font-xs); color: var(--accent); pointer-events: none; transform: translateY(-50%); }
.code-input { position: relative; }
.code-input input { width: 100%; padding-right: 34px; }
.code-input svg { width: 17px; height: 17px; fill: none; stroke: currentColor; stroke-width: 2; stroke-linecap: round; stroke-linejoin: round; }
.lookup-spinner, .lookup-success { position: absolute; top: 50%; right: 11px; pointer-events: none; transform: translateY(-50%); }
.lookup-spinner { width: 14px; height: 14px; border: 1.5px solid color-mix(in srgb, var(--accent) 22%, transparent); border-top-color: var(--accent); border-radius: 50%; animation: lookup-spin 720ms linear infinite; }
.lookup-success { color: var(--loss); }
.lookup-result { display: flex; gap: 7px; align-items: center; min-height: 26px; padding: 4px 8px; margin-top: -5px; overflow: hidden; font-size: var(--font-xs); color: var(--text-muted); background: color-mix(in srgb, var(--glass-subtle) 72%, transparent); border: 1px solid var(--hairline); border-radius: 8px; }
.lookup-result > span { flex: none; padding-right: 7px; border-right: 1px solid var(--hairline); }
.lookup-result .lookup-source { color: var(--loss); }
.lookup-result small { margin-left: auto; color: var(--text-muted); white-space: nowrap; }
.lookup-result.warning { color: var(--warning); border-color: color-mix(in srgb, var(--warning) 20%, var(--hairline)); }
@keyframes lookup-spin { to { transform: translateY(-50%) rotate(360deg); } }
.import-workspace { min-height: 0; overflow: auto; }
.drop-zone { align-items: center; justify-content: space-between; min-height: 110px; padding: 20px; background: var(--glass-subtle); border: 1px dashed var(--hairline-strong); border-radius: 18px; }
.drop-zone.compact { min-height: 82px; padding: 14px 16px; }
.drop-copy { gap: 13px; align-items: center; }.drop-copy > svg { width: 28px; height: 28px; color: var(--accent); }
.drop-copy div { display: flex; flex-direction: column; }.drop-copy strong { font-size: var(--font-md); color: var(--text-strong); }.drop-copy span { margin-top: 5px; font-size: var(--font-xs); color: var(--text-muted); }
.file-button { display: inline-flex; flex-direction: row; padding: 9px 13px; font-size: var(--font-sm); color: white; cursor: pointer; background: var(--accent); border-radius: 10px; }.file-button input { display: none; }
.form-note, .form-error { min-height: 16px; margin: 0; font-size: var(--font-xs); color: var(--text-muted); }.form-error { color: var(--profit); }.import-error { padding: 10px 12px; margin-top: 10px; background: var(--profit-soft); border-radius: 10px; }
.preview-section { margin-top: 12px; }.preview-section > header { align-items: center; justify-content: space-between; margin-bottom: 8px; }.preview-section > header div { display: flex; flex-direction: column; }.preview-section > header strong { font-size: var(--font-sm); color: var(--text-strong); }.preview-section > header span, .preview-section > header small { margin-top: 3px; font-size: var(--font-xs); color: var(--text-muted); }
.preview-table { overflow: hidden; border: 1px solid var(--hairline); border-radius: 13px; }
.preview-head, .preview-row { display: grid; grid-template-columns: 32px minmax(210px, 1.6fr) minmax(90px, .65fr) minmax(100px, .72fr) minmax(120px, .8fr) 32px; gap: 8px; align-items: center; }
.preview-head { padding: 7px 10px; font-size: var(--font-xs); color: var(--text-muted); background: var(--glass-subtle); }
.preview-row { padding: 8px 10px; border-top: 1px solid var(--hairline); transition: 150ms ease; }.preview-row.invalid { background: var(--profit-soft); }
.identity-fields { display: grid; grid-template-columns: 72px 1fr; gap: 6px; }.strategy-field { min-width: 0; }.strategy-field small { display: block; margin-top: 3px; overflow: hidden; font-size: var(--font-xs); color: var(--warning); text-overflow: ellipsis; white-space: nowrap; }
.check { display: grid; width: 18px; height: 18px; cursor: pointer; place-items: center; }.check input { position: absolute; opacity: 0; }.check i { display: block; width: 15px; height: 15px; border: 1px solid var(--hairline-strong); border-radius: 5px; }.check input:checked + i { background: var(--accent); border-color: var(--accent); box-shadow: inset 0 0 0 3px var(--glass-strong); }
.remove-row { display: grid; width: 28px; height: 28px; padding: 0; background: transparent; border-color: transparent; border-radius: 8px; place-items: center; }.remove-row:hover { background: var(--glass-subtle); }
footer { flex: none; gap: 8px; align-items: center; justify-content: flex-end; padding-top: 16px; margin-top: auto; border-top: 1px solid var(--hairline); }footer button { padding: 9px 14px; font-size: var(--font-sm); border-radius: 10px; }footer .primary { display: inline-flex; gap: 7px; align-items: center; justify-content: center; min-width: 100px; color: white; background: var(--accent); border-color: transparent; }.privacy-note { display: flex; gap: 7px; align-items: center; margin-right: auto; font-size: var(--font-xs); color: var(--text-muted); }.privacy-note i { width: 6px; height: 6px; background: var(--loss); border-radius: 99px; box-shadow: 0 0 0 3px var(--loss-soft); }
.button-spinner { width: 13px; height: 13px; border: 1.5px solid rgba(255, 255, 255, .34); border-top-color: white; border-radius: 50%; animation: button-spin 680ms linear infinite; }
@keyframes button-spin { to { transform: rotate(360deg); } }
@media (max-width: 760px) { .preview-head { display: none; }.preview-row { grid-template-columns: 28px 1fr 28px; }.preview-row > input, .strategy-field { grid-column: 2; }.dialog-backdrop { padding: 10px; }.editor { width: 100%; max-height: calc(100vh - 20px); }.mode-tabs button { padding-inline: 9px; }.mode-tabs svg { display: none; } }
</style>
