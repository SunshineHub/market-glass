<script setup lang="ts">
import type { AppUpdateInfo } from "@/features/update/appUpdater";

export type UpdateDialogState =
  | "checking"
  | "available"
  | "downloading"
  | "installing"
  | "current"
  | "error";

const props = defineProps<{
  state: UpdateDialogState;
  currentVersion: string;
  update?: AppUpdateInfo;
  progress?: number;
  error?: string;
}>();

defineEmits<{
  close: [];
  check: [];
  install: [];
}>();

function formatDate(value?: string) {
  if (!value) return "";
  const date = new Date(value);
  return Number.isNaN(date.getTime())
    ? ""
    : date.toLocaleDateString("zh-CN", {
        year: "numeric",
        month: "long",
        day: "numeric",
      });
}

function canClose() {
  return props.state !== "downloading" && props.state !== "installing";
}
</script>

<template>
  <div class="update-backdrop no-drag" @mousedown.self="canClose() && $emit('close')">
    <section
      class="update-dialog material-panel"
      role="dialog"
      aria-modal="true"
      aria-labelledby="update-title"
    >
      <header>
        <div class="update-symbol" :class="state">
          <svg v-if="state === 'current'" viewBox="0 0 24 24"><path d="m5 12 4 4 10-10" /></svg>
          <svg v-else-if="state === 'error'" viewBox="0 0 24 24"><path d="M12 8v5M12 17h.01" /><circle cx="12" cy="12" r="9" /></svg>
          <svg v-else viewBox="0 0 24 24" :class="{ spinning: state === 'checking' }">
            <path d="M12 3v12" /><path d="m7 11 5 5 5-5" /><path d="M5 20h14" />
          </svg>
        </div>
        <div>
          <span>MARKET GLASS UPDATE</span>
          <h2 id="update-title">
            {{
              state === "checking"
                ? "正在检查更新"
                : state === "current"
                  ? "已经是最新版本"
                  : state === "error"
                    ? "暂时无法检查更新"
                    : state === "installing"
                      ? "正在完成安装"
                      : state === "downloading"
                        ? "正在下载安全更新"
                        : "发现新版本"
            }}
          </h2>
        </div>
        <button
          v-if="canClose()"
          type="button"
          class="close-button"
          aria-label="关闭更新窗口"
          @click="$emit('close')"
        >
          <svg viewBox="0 0 24 24"><path d="m7 7 10 10M17 7 7 17" /></svg>
        </button>
      </header>

      <div v-if="state === 'available' || state === 'downloading' || state === 'installing'" class="version-flow">
        <div><span>当前版本</span><strong>v{{ update?.currentVersion ?? currentVersion }}</strong></div>
        <svg viewBox="0 0 24 24"><path d="M5 12h14M15 8l4 4-4 4" /></svg>
        <div class="latest"><span>可用版本</span><strong>v{{ update?.version }}</strong></div>
      </div>

      <div v-if="state === 'available' && update?.notes" class="release-notes">
        <div><strong>本次更新</strong><span>{{ formatDate(update.date) }}</span></div>
        <p>{{ update.notes }}</p>
      </div>

      <div v-if="state === 'downloading' || state === 'installing'" class="progress-block">
        <div>
          <span>{{ state === "installing" ? "正在校验并安装" : "下载进度" }}</span>
          <strong>{{ progress ?? 0 }}%</strong>
        </div>
        <div class="progress-track"><i :style="{ width: `${progress ?? 0}%` }" /></div>
        <p>{{ state === "installing" ? "安装完成后应用会自动重新打开。" : "更新包会先完成签名校验，再进入安装。" }}</p>
      </div>

      <p v-else-if="state === 'checking'" class="state-copy">
        正在连接 GitHub Release 并验证可用版本，请稍候。
      </p>
      <p v-else-if="state === 'current'" class="state-copy">
        当前版本为 v{{ currentVersion }}，无需进行任何操作。
      </p>
      <p v-else-if="state === 'error'" class="state-copy error-copy">
        {{ error || "网络连接失败，请稍后重试。" }}
      </p>

      <footer>
        <template v-if="state === 'available'">
          <button type="button" class="secondary" @click="$emit('close')">稍后提醒</button>
          <button type="button" class="primary" data-liquid-glass @click="$emit('install')">
            立即更新
          </button>
        </template>
        <template v-else-if="state === 'current'">
          <button type="button" class="secondary wide" @click="$emit('close')">完成</button>
        </template>
        <template v-else-if="state === 'error'">
          <button type="button" class="secondary" @click="$emit('close')">关闭</button>
          <button type="button" class="primary" data-liquid-glass @click="$emit('check')">重新检查</button>
        </template>
        <button v-else type="button" class="secondary wide" disabled>
          {{ state === "checking" ? "检查中…" : state === "installing" ? "正在安装…" : "请保持应用开启" }}
        </button>
      </footer>
    </section>
  </div>
</template>

<style scoped>
.update-backdrop {
  position: fixed;
  z-index: 90;
  display: grid;
  padding: 22px;
  background: rgba(12, 17, 29, .3);
  inset: 0;
  place-items: center;
  backdrop-filter: blur(18px) saturate(120%);
  -webkit-backdrop-filter: blur(18px) saturate(120%);
}

.update-dialog {
  width: min(500px, calc(100vw - 44px));
  padding: 24px;
  border-radius: 24px;
}

.update-dialog > header {
  display: grid;
  grid-template-columns: 50px minmax(0, 1fr) 34px;
  gap: 13px;
  align-items: center;
}

.update-dialog header > div:nth-child(2) {
  min-width: 0;
}

.update-dialog header span {
  font-size: var(--font-xs);
  font-weight: 720;
  color: var(--accent);
  letter-spacing: .1em;
}

.update-dialog h2 {
  margin: 3px 0 0;
  font-size: var(--font-lg);
  color: var(--text-strong);
  letter-spacing: -.025em;
}

.update-symbol {
  display: grid;
  width: 50px;
  height: 50px;
  color: white;
  background: linear-gradient(145deg, #6f85ff, #5069e9);
  border: 1px solid rgba(255, 255, 255, .36);
  border-radius: 16px;
  box-shadow: 0 9px 22px color-mix(in srgb, var(--accent) 28%, transparent);
  place-items: center;
}

.update-symbol.current {
  background: linear-gradient(145deg, #3eb89d, #229578);
}

.update-symbol.error {
  background: linear-gradient(145deg, #f4a657, #dd7d32);
}

.update-symbol svg,
.close-button svg,
.version-flow > svg {
  fill: none;
  stroke: currentColor;
  stroke-width: 1.9;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.update-symbol svg {
  width: 24px;
  height: 24px;
}

.close-button {
  display: grid;
  width: 34px;
  height: 34px;
  padding: 0;
  color: var(--text-muted);
  cursor: pointer;
  background: var(--glass-subtle);
  border: 1px solid var(--hairline);
  border-radius: 11px;
  place-items: center;
}

.close-button svg {
  width: 17px;
  height: 17px;
}

.version-flow {
  display: grid;
  grid-template-columns: 1fr 30px 1fr;
  gap: 8px;
  align-items: center;
  padding: 14px 15px;
  margin-top: 20px;
  background: var(--glass-subtle);
  border: 1px solid var(--hairline);
  border-radius: 15px;
}

.version-flow > div {
  display: flex;
  flex-direction: column;
}

.version-flow .latest {
  text-align: right;
}

.version-flow span {
  font-size: var(--font-xs);
  color: var(--text-muted);
}

.version-flow strong {
  margin-top: 4px;
  font-size: var(--font-md);
  color: var(--text-strong);
}

.version-flow .latest strong {
  color: var(--accent);
}

.version-flow > svg {
  width: 18px;
  height: 18px;
  color: var(--text-muted);
  justify-self: center;
}

.release-notes {
  padding: 15px;
  margin-top: 11px;
  background: color-mix(in srgb, var(--material-content) 76%, transparent);
  border: 1px solid var(--hairline);
  border-radius: 15px;
}

.release-notes > div,
.progress-block > div:first-child {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.release-notes strong,
.progress-block strong {
  font-size: var(--font-sm);
  color: var(--text-strong);
}

.release-notes span,
.progress-block span {
  font-size: var(--font-xs);
  color: var(--text-muted);
}

.release-notes p {
  margin: 8px 0 0;
  max-height: 150px;
  overflow: auto;
  font-size: var(--font-sm);
  line-height: 1.65;
  color: var(--text);
  white-space: pre-wrap;
}

.state-copy {
  margin: 20px 2px 5px;
  font-size: var(--font-sm);
  line-height: 1.65;
  color: var(--text-muted);
  text-align: center;
}

.error-copy {
  color: var(--warning);
}

.progress-block {
  margin-top: 20px;
}

.progress-track {
  height: 8px;
  margin-top: 10px;
  overflow: hidden;
  background: var(--glass-subtle);
  border: 1px solid var(--hairline);
  border-radius: 999px;
}

.progress-track i {
  display: block;
  height: 100%;
  background: linear-gradient(90deg, #647cf4, #91a3ff);
  border-radius: inherit;
  transition: width 180ms ease;
}

.progress-block p {
  margin: 9px 0 0;
  font-size: var(--font-xs);
  color: var(--text-muted);
}

.update-dialog footer {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 9px;
  margin-top: 20px;
}

.update-dialog footer button {
  min-height: 42px;
  padding: 9px 14px;
  font-size: var(--font-sm);
  font-weight: 650;
  cursor: pointer;
  border: 1px solid var(--hairline);
  border-radius: 12px;
}

.update-dialog footer button:disabled {
  cursor: default;
  opacity: .62;
}

.update-dialog footer .wide {
  grid-column: 1 / -1;
}

.secondary {
  color: var(--text);
  background: var(--glass-subtle);
}

.primary {
  color: white;
  background: linear-gradient(145deg, #6e84fb, #566eef);
  border-color: transparent !important;
  box-shadow: 0 7px 18px color-mix(in srgb, var(--accent) 25%, transparent);
}

.spinning {
  animation: update-pulse 900ms ease-in-out infinite alternate;
}

@keyframes update-pulse {
  from { opacity: .5; transform: translateY(-1px); }
  to { opacity: 1; transform: translateY(1px); }
}

:global(html[data-platform="windows"] .update-backdrop) {
  background: rgba(19, 27, 40, .38);
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
}

:global(html[data-platform="windows"] .update-dialog) {
  background: var(--material-elevated);
}
</style>
