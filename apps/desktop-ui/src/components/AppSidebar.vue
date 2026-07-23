<script setup lang="ts">
const props = defineProps<{ active: string }>();
const emit = defineEmits<{ select: [id: string] }>();

const items = [
  { id: "overview", label: "总览", icon: "overview" },
  { id: "analysis", label: "资产分析", icon: "analysis" },
  { id: "funds", label: "基金", icon: "fund" },
  { id: "market", label: "大盘", icon: "market" },
] as const;
</script>

<template>
  <aside class="sidebar">
    <div class="brand drag-region" data-tauri-drag-region>
      <div class="brand-mark"><svg viewBox="0 0 32 32"><path d="m5.5 23 7.7-7.5 5.8 3.8 8.7-11.8" /><circle cx="27.7" cy="7.5" r="2" /></svg></div>
      <div><strong>澄明行情</strong><span>Market Glass</span></div>
    </div>

    <nav aria-label="主导航">
      <button v-for="item in items" :key="item.id" type="button" :class="{ active: props.active === item.id }" data-liquid-glass @click="emit('select', item.id)">
        <i>
          <svg v-if="item.icon === 'overview'" viewBox="0 0 24 24"><rect x="4" y="4" width="6" height="6" rx="2" /><rect x="14" y="4" width="6" height="6" rx="2" /><rect x="4" y="14" width="6" height="6" rx="2" /><rect x="14" y="14" width="6" height="6" rx="2" /></svg>
          <svg v-else-if="item.icon === 'analysis'" viewBox="0 0 24 24"><path d="M12 3a9 9 0 1 0 9 9h-9z" /><path d="M15 3.5A8.5 8.5 0 0 1 20.5 9H15z" /><path d="M8 14.5h2M8 18h6" /></svg>
          <svg v-else-if="item.icon === 'fund'" viewBox="0 0 24 24"><path d="M12 3a9 9 0 1 0 9 9h-9z" /><path d="M15 3.5A8.5 8.5 0 0 1 20.5 9H15z" /></svg>
          <svg v-else viewBox="0 0 24 24"><path d="M5 19V9M12 19V5M19 19v-7" /><path d="M3 19h18" /></svg>
        </i>
        <span>{{ item.label }}</span>
      </button>
    </nav>

    <div class="sidebar-bottom">
      <div class="local-badge"><i /><span>本地优先</span></div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar { position: relative; z-index: 1; display: flex; flex: 0 0 190px; flex-direction: column; width: 190px; min-width: 0; padding: 20px 14px 16px; background: radial-gradient(ellipse at 12% 72%, var(--sidebar-glow), transparent 52%), linear-gradient(155deg, color-mix(in srgb, var(--sidebar-surface) 94%, white 6%), color-mix(in srgb, var(--sidebar-surface) 84%, var(--bg-a))); box-shadow: 1px 0 0 color-mix(in srgb, var(--hairline-strong) 72%, transparent) inset, 16px 0 42px color-mix(in srgb, var(--sidebar-glow) 28%, transparent); backdrop-filter: blur(32px) saturate(142%); -webkit-backdrop-filter: blur(32px) saturate(142%); transition: width 180ms ease, flex-basis 180ms ease, padding 180ms ease; }
.sidebar::before { position: absolute; top: -12%; right: -84px; bottom: -12%; z-index: 0; width: 148px; content: ""; pointer-events: none; background: linear-gradient(90deg, color-mix(in srgb, var(--sidebar-surface) 64%, transparent), color-mix(in srgb, var(--sidebar-glow) 36%, transparent) 48%, transparent 92%); filter: blur(17px); opacity: .74; }
.sidebar::after { position: absolute; top: 0; right: -58px; bottom: 0; z-index: 0; width: 92px; content: ""; pointer-events: none; background: linear-gradient(90deg, color-mix(in srgb, var(--sidebar-surface) 30%, transparent), transparent); }
.sidebar > * { position: relative; z-index: 1; }
.brand { display: flex; gap: 10px; align-items: center; min-height: 42px; padding: 0 9px; }
.brand-mark { display: grid; width: 34px; height: 34px; color: white; background: linear-gradient(145deg, rgba(255,116,104,.92), rgba(96,120,244,.92)); border: 1px solid rgba(255,255,255,.34); border-radius: 12px; box-shadow: 0 8px 22px rgba(96,120,244,.22); place-items: center; }
.brand-mark svg { width: 23px; height: 23px; fill: none; stroke: currentColor; stroke-width: 2.2; stroke-linecap: round; stroke-linejoin: round; }
.brand > div:last-child { display: flex; flex-direction: column; }.brand strong { font-size: var(--font-md); color: var(--text-strong); }.brand span { margin-top: 2px; font-size: var(--font-xs); color: var(--text-muted); letter-spacing: .08em; }
nav { margin-top: 27px; }nav,.sidebar-bottom { display: flex; flex-direction: column; gap: 5px; }
button { display: flex; gap: 11px; align-items: center; width: 100%; padding: 9px; font-size: var(--font-sm); font-weight: 560; color: var(--text-muted); text-align: left; cursor: pointer; background: transparent; border: 1px solid transparent; border-radius: 12px; -webkit-backdrop-filter: none; backdrop-filter: none; transition: color 170ms ease, background-color 170ms ease, border-color 170ms ease, transform 170ms ease; }
button:hover { color: var(--text-strong); background: color-mix(in srgb, var(--sidebar-surface) 58%, transparent); border-color: color-mix(in srgb, var(--hairline) 66%, transparent); }
button.active { color: var(--text-strong); background: linear-gradient(110deg, color-mix(in srgb, var(--sidebar-surface) 78%, white 22%), color-mix(in srgb, var(--sidebar-surface) 88%, var(--bg-a))); border-color: color-mix(in srgb, var(--material-highlight) 38%, var(--hairline)); box-shadow: 0 7px 18px color-mix(in srgb, var(--sidebar-glow) 18%, transparent), 0 1px 0 color-mix(in srgb, var(--material-highlight) 52%, transparent) inset; }
button i { display: grid; width: 26px; height: 26px; font-style: normal; background: color-mix(in srgb, var(--sidebar-surface) 52%, transparent); border: 1px solid color-mix(in srgb, var(--hairline) 78%, transparent); border-radius: 9px; place-items: center; }
button i svg { width: 15px; height: 15px; fill: none; stroke: currentColor; stroke-width: 1.8; stroke-linecap: round; stroke-linejoin: round; }
button.active i { color: white; background: linear-gradient(145deg,var(--accent),#7c69ee); border-color: transparent; box-shadow: 0 6px 16px rgba(96,120,244,.23); }
.sidebar-bottom { margin-top: auto; }.local-badge { display: flex; gap: 7px; align-items: center; padding: 11px 12px 2px; font-size: var(--font-xs); color: var(--text-muted); }.local-badge i { width: 6px; height: 6px; background: var(--loss); border-radius: 99px; box-shadow: 0 0 0 3px var(--loss-soft); }

@media (max-width: 980px) {
  .sidebar { flex-basis: 166px; width: 166px; padding-right: 10px; padding-left: 10px; }
  .brand { padding-right: 5px; padding-left: 5px; }
  button { gap: 9px; }
}
</style>
