import { getVersion } from "@tauri-apps/api/app";
import { relaunch } from "@tauri-apps/plugin-process";
import {
  check,
  type DownloadEvent,
  type Update,
} from "@tauri-apps/plugin-updater";

export interface AppUpdateInfo {
  currentVersion: string;
  version: string;
  date?: string;
  notes?: string;
}

export interface AppUpdateProgress {
  downloaded: number;
  total?: number;
  percent?: number;
}

let pendingUpdate: Update | null = null;
let previewUpdate = false;

function isTauriRuntime() {
  return Boolean(window.__TAURI_INTERNALS__);
}

function previewUpdateEnabled() {
  return import.meta.env.DEV
    && new URLSearchParams(window.location.search).get("previewUpdate") === "1";
}

function toProgress(downloaded: number, total?: number): AppUpdateProgress {
  const percent = total && total > 0
    ? Math.min(100, Math.round((downloaded / total) * 100))
    : undefined;
  return { downloaded, total, percent };
}

export async function getCurrentAppVersion(): Promise<string> {
  if (!isTauriRuntime()) return "0.1.3";
  return getVersion();
}

export async function checkForAppUpdate(): Promise<AppUpdateInfo | null> {
  if (!isTauriRuntime()) {
    await new Promise((resolve) => window.setTimeout(resolve, 520));
    previewUpdate = previewUpdateEnabled();
    return previewUpdate
      ? {
          currentVersion: "0.1.2",
          version: "0.1.3",
          date: new Date().toISOString(),
          notes: "新增应用内自动更新、签名校验和下载进度展示。\n优化跨平台安装与重启流程。",
        }
      : null;
  }

  if (pendingUpdate) {
    await pendingUpdate.close();
    pendingUpdate = null;
  }

  pendingUpdate = await check({ timeout: 20_000 });
  if (!pendingUpdate) return null;
  return {
    currentVersion: pendingUpdate.currentVersion,
    version: pendingUpdate.version,
    date: pendingUpdate.date,
    notes: pendingUpdate.body,
  };
}

export async function installAppUpdate(
  onProgress: (progress: AppUpdateProgress) => void,
): Promise<void> {
  if (!isTauriRuntime()) {
    if (!previewUpdate) throw new Error("没有可安装的更新");
    const total = 12_000_000;
    for (let downloaded = 0; downloaded <= total; downloaded += 1_500_000) {
      onProgress(toProgress(Math.min(downloaded, total), total));
      await new Promise((resolve) => window.setTimeout(resolve, 90));
    }
    return;
  }

  if (!pendingUpdate) throw new Error("更新信息已失效，请重新检查");

  let downloaded = 0;
  let total: number | undefined;
  const handleDownloadEvent = (event: DownloadEvent) => {
    if (event.event === "Started") {
      total = event.data.contentLength;
      onProgress(toProgress(downloaded, total));
      return;
    }
    if (event.event === "Progress") {
      downloaded += event.data.chunkLength;
      onProgress(toProgress(downloaded, total));
      return;
    }
    if (event.event === "Finished") {
      onProgress(toProgress(total ?? downloaded, total ?? downloaded));
    }
  };

  await pendingUpdate.downloadAndInstall(handleDownloadEvent, { timeout: 10 * 60_000 });
  pendingUpdate = null;
  await relaunch();
}
