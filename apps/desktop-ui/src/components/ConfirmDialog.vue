<script setup lang="ts">
withDefaults(
  defineProps<{
    title: string;
    description: string;
    confirmLabel?: string;
    busy?: boolean;
  }>(),
  { confirmLabel: "确认删除", busy: false },
);
defineEmits<{ close: []; confirm: [] }>();
</script>

<template>
  <div class="confirm-backdrop no-drag" @mousedown.self="$emit('close')">
    <section class="confirm-dialog material-panel" role="alertdialog" aria-modal="true" aria-labelledby="confirm-title">
      <div class="danger-icon">
        <svg viewBox="0 0 24 24"><path d="M4 7h16M9 7V4h6v3M7 7l1 13h8l1-13M10 11v5M14 11v5" /></svg>
      </div>
      <h2 id="confirm-title">{{ title }}</h2>
      <p>{{ description }}</p>
      <footer>
        <button type="button" class="secondary" :disabled="busy" @click="$emit('close')">取消</button>
        <button type="button" class="danger" :disabled="busy" @click="$emit('confirm')">
          {{ busy ? "处理中…" : confirmLabel }}
        </button>
      </footer>
    </section>
  </div>
</template>

<style scoped>
.confirm-backdrop {
  position: fixed;
  z-index: 50;
  display: grid;
  padding: 20px;
  background: rgba(12, 17, 29, .28);
  inset: 0;
  place-items: center;
  backdrop-filter: blur(16px) saturate(120%);
}

.confirm-dialog {
  width: min(390px, calc(100vw - 40px));
  padding: 24px;
  text-align: center;
  border-radius: 24px;
}

.danger-icon {
  display: grid;
  width: 46px;
  height: 46px;
  margin: 0 auto 13px;
  color: var(--profit);
  background: var(--profit-soft);
  border: 1px solid color-mix(in srgb, var(--profit) 20%, transparent);
  border-radius: 15px;
  place-items: center;
}

.danger-icon svg {
  width: 21px;
  height: 21px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.8;
  stroke-linecap: round;
  stroke-linejoin: round;
}

h2 {
  margin: 0;
  font-size: var(--font-lg);
  color: var(--text-strong);
}

p {
  margin: 9px 0 20px;
  font-size: var(--font-sm);
  line-height: 1.65;
  color: var(--text-muted);
}

footer {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 9px;
}

button {
  padding: 10px 14px;
  font-size: var(--font-sm);
  cursor: pointer;
  border: 1px solid var(--hairline);
  border-radius: 11px;
}

button:disabled { cursor: default; opacity: .55; }
.secondary { color: var(--text); background: var(--glass-subtle); }
.danger { color: white; background: var(--profit); border-color: transparent; }
</style>
