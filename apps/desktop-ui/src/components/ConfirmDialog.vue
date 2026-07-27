<script setup lang="ts">
import { useDelayedBusy } from "@/features/ui/useDelayedBusy";

const props = withDefaults(
  defineProps<{
    title: string;
    description: string;
    confirmLabel?: string;
    busy?: boolean;
  }>(),
  { confirmLabel: "确认删除", busy: false },
);
const emit = defineEmits<{ close: []; confirm: [] }>();
const busyVisible = useDelayedBusy(() => props.busy);

function close() {
  if (!props.busy) emit("close");
}
</script>

<template>
  <div class="confirm-backdrop no-drag" @mousedown.self="close">
    <section class="confirm-dialog material-panel" role="alertdialog" aria-modal="true" aria-labelledby="confirm-title">
      <div class="danger-icon">
        <svg viewBox="0 0 24 24"><path d="M4 7h16M9 7V4h6v3M7 7l1 13h8l1-13M10 11v5M14 11v5" /></svg>
      </div>
      <h2 id="confirm-title">{{ title }}</h2>
      <p>{{ description }}</p>
      <footer>
        <button type="button" class="secondary" :disabled="busy" @click="close">取消</button>
        <button type="button" class="danger" :disabled="busy" @click="$emit('confirm')">
          <i v-if="busyVisible" class="button-spinner" />
          <span>{{ busyVisible ? "正在删除" : confirmLabel }}</span>
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
.danger { display: inline-flex; gap: 7px; align-items: center; justify-content: center; color: white; background: var(--profit); border-color: transparent; }
.button-spinner { width: 13px; height: 13px; border: 1.5px solid rgba(255, 255, 255, .34); border-top-color: white; border-radius: 50%; animation: button-spin 680ms linear infinite; }
@keyframes button-spin { to { transform: rotate(360deg); } }
</style>
