<script setup lang="ts">
import { computed, ref } from "vue";
import type { IndexOption } from "@/types/contracts";

const props = withDefaults(
  defineProps<{
    options: IndexOption[];
    selected: string[];
    saving?: boolean;
    maxSelections?: number;
    eyebrow?: string;
    title?: string;
    description?: string;
    saveLabel?: string;
  }>(),
  {
    saving: false,
    maxSelections: 4,
    eyebrow: "OVERVIEW INDICES",
    title: "编辑总览指数",
    description: "选择 1–4 个指数；完整全球行情仍保留在“大盘”页。",
    saveLabel: "保存自选指数",
  },
);
const emit = defineEmits<{ close: []; save: [codes: string[]] }>();
const draft = ref([...props.selected]);
const groups = computed(() => {
  const result = new Map<string, IndexOption[]>();
  for (const option of props.options) {
    const group = result.get(option.region) ?? [];
    group.push(option);
    result.set(option.region, group);
  }
  return [...result.entries()];
});

function toggle(code: string) {
  if (draft.value.includes(code)) {
    if (draft.value.length > 1) draft.value = draft.value.filter((item) => item !== code);
    return;
  }
  if (draft.value.length < props.maxSelections) draft.value = [...draft.value, code];
}
</script>

<template>
  <div class="picker-backdrop no-drag" @mousedown.self="$emit('close')">
    <section class="picker material-panel" role="dialog" aria-modal="true" aria-labelledby="index-picker-title">
      <header>
        <div>
          <span>{{ eyebrow }}</span>
          <h2 id="index-picker-title">{{ title }}</h2>
          <p>{{ description }}</p>
        </div>
        <button type="button" aria-label="关闭" @click="$emit('close')">
          <svg viewBox="0 0 24 24"><path d="m7 7 10 10M17 7 7 17" /></svg>
        </button>
      </header>

      <div class="selection-count"><strong>{{ draft.length }}</strong> / {{ maxSelections }} 已选择</div>
      <div class="region-list">
        <section v-for="[region, items] in groups" :key="region">
          <h3>{{ region }}</h3>
          <div class="option-grid">
            <button
              v-for="option in items"
              :key="option.code"
              type="button"
              :class="{ selected: draft.includes(option.code) }"
              :disabled="!draft.includes(option.code) && draft.length >= maxSelections"
              @click="toggle(option.code)"
            >
              <i><svg viewBox="0 0 24 24"><path d="m6 12 4 4 8-9" /></svg></i>
              <span><strong>{{ option.name }}</strong><small>{{ option.code }}</small></span>
            </button>
          </div>
        </section>
      </div>

      <footer>
        <button type="button" class="secondary" @click="$emit('close')">取消</button>
        <button type="button" class="primary" :disabled="saving" @click="$emit('save', draft)">
          {{ saving ? "保存中…" : saveLabel }}
        </button>
      </footer>
    </section>
  </div>
</template>

<style scoped>
.picker-backdrop { position: fixed; z-index: 35; display: grid; padding: 24px; background: rgba(12,17,29,.26); inset: 0; place-items: center; backdrop-filter: blur(16px) saturate(125%); }
.picker { display: flex; flex-direction: column; width: min(720px, calc(100vw - 40px)); max-height: min(650px, calc(100vh - 40px)); padding: 24px; overflow: hidden; background: var(--glass-strong); border-radius: 26px; box-shadow: 0 34px 90px rgba(22,29,48,.28); }
header,footer,header button,.option-grid button,.option-grid button i,.selection-count { display: flex; align-items: center; }
header { align-items: flex-start; justify-content: space-between; }
header > div > span { font-size: var(--font-xs); font-weight: 700; color: var(--accent); letter-spacing: .1em; }
h2 { margin: 5px 0 0; font-size: var(--font-lg); color: var(--text-strong); }
header p { margin: 7px 0 0; font-size: var(--font-sm); color: var(--text-muted); }
button { color: var(--text); cursor: pointer; background: var(--glass-subtle); border: 1px solid var(--hairline); }
header button { justify-content: center; width: 34px; height: 34px; padding: 0; border-radius: 11px; }
button svg { width: 16px; height: 16px; fill: none; stroke: currentColor; stroke-width: 2; stroke-linecap: round; stroke-linejoin: round; }
.selection-count { align-self: flex-end; gap: 4px; padding: 6px 10px; margin: 12px 0 4px; font-size: var(--font-xs); color: var(--text-muted); background: var(--glass-subtle); border-radius: 99px; }
.selection-count strong { color: var(--accent); }
.region-list { min-height: 0; padding-right: 4px; overflow: auto; }
.region-list section { padding: 10px 0 13px; }
.region-list h3 { margin: 0 0 8px; font-size: var(--font-xs); color: var(--text-muted); }
.option-grid { display: grid; grid-template-columns: repeat(3,minmax(0,1fr)); gap: 8px; }
.option-grid button { gap: 9px; min-width: 0; padding: 10px; text-align: left; border-radius: 12px; transition: background-color 160ms ease,border-color 160ms ease,transform 160ms ease; }
.option-grid button:not(:disabled):hover { background: var(--glass-hover); }
.option-grid button:disabled { cursor: default; opacity: .42; }
.option-grid button i { justify-content: center; flex: 0 0 22px; width: 22px; height: 22px; color: transparent; border: 1px solid var(--hairline-strong); border-radius: 7px; }
.option-grid button.selected { color: var(--accent); background: var(--accent-soft); border-color: color-mix(in srgb,var(--accent) 30%,transparent); }
.option-grid button.selected i { color: white; background: var(--accent); border-color: var(--accent); }
.option-grid button span { display: flex; flex-direction: column; min-width: 0; }
.option-grid strong { overflow: hidden; font-size: var(--font-sm); color: var(--text-strong); text-overflow: ellipsis; white-space: nowrap; }
.option-grid small { margin-top: 3px; font: 500 var(--font-xs)/1 var(--font-mono); color: var(--text-muted); }
footer { gap: 8px; justify-content: flex-end; padding-top: 16px; margin-top: 12px; border-top: 1px solid var(--hairline); }
footer button { padding: 9px 14px; font-size: var(--font-sm); border-radius: 10px; }
footer .primary { min-width: 120px; color: white; background: var(--accent); border-color: transparent; }
@media (max-width: 760px) { .option-grid { grid-template-columns: repeat(2,minmax(0,1fr)); }.picker { padding: 18px; } }
</style>
