import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { mockBootstrap } from "@/ipc/mock";
import type {
  BootstrapPayload,
  FundMetadata,
  OverviewSnapshot,
  PositionBatchUpdateResult,
  PositionInput,
} from "@/types/contracts";

export const isTauri = () => Boolean(window.__TAURI_INTERNALS__);

export async function getBootstrap(): Promise<BootstrapPayload> {
  if (!isTauri()) return structuredClone(mockBootstrap);
  return invoke<BootstrapPayload>("get_bootstrap");
}

export async function refreshOverview(): Promise<OverviewSnapshot> {
  if (!isTauri()) {
    await new Promise((resolve) => window.setTimeout(resolve, 420));
    const snapshot = structuredClone(mockBootstrap.overview);
    snapshot.calculatedAt = new Date().toISOString();
    return snapshot;
  }
  return invoke<OverviewSnapshot>("refresh_overview");
}

export async function lookupFund(code: string): Promise<FundMetadata | null> {
  if (!isTauri()) {
    await new Promise((resolve) => window.setTimeout(resolve, 280));
    const asset = mockBootstrap.overview.assets.find((item) => item.code === code);
    if (asset) {
      return {
        code,
        name: asset.name,
        industry: asset.strategy,
        fundType: "公募基金",
        provider: "本地预览数据",
      };
    }
    if (code === "161725") {
      return {
        code,
        name: "招商中证白酒指数(LOF)A",
        fundType: "指数型-股票",
        company: "招商基金",
        industry: "食品饮料",
        indexName: "中证白酒指数",
        latestNav: "0.5581",
        navDate: "2026-07-22",
        provider: "本地预览数据",
      };
    }
    return null;
  }
  return invoke<FundMetadata | null>("lookup_fund", { code });
}

export async function setPrivacyMode(enabled: boolean): Promise<void> {
  if (!isTauri()) return;
  await invoke("set_privacy_mode", { enabled });
}

export async function setSelectedIndices(codes: string[]): Promise<string[]> {
  if (!isTauri()) return codes.slice(0, 4);
  return invoke<string[]>("set_selected_indices", { codes });
}

export async function setMarketIndices(codes: string[]): Promise<string[]> {
  if (!isTauri()) return [...new Set(codes)];
  return invoke<string[]>("set_market_indices", { codes });
}

export async function upsertPosition(input: PositionInput): Promise<OverviewSnapshot> {
  if (!isTauri()) return refreshOverview();
  return invoke<OverviewSnapshot>("upsert_position", { input });
}

export async function importPositions(inputs: PositionInput[]): Promise<OverviewSnapshot> {
  if (!isTauri()) return refreshOverview();
  return invoke<OverviewSnapshot>("import_positions", { inputs });
}

export async function updatePositionsPartial(
  inputs: PositionInput[],
): Promise<PositionBatchUpdateResult> {
  if (!isTauri()) {
    return {
      snapshot: await refreshOverview(),
      succeededIds: inputs.flatMap((input) => input.id ? [input.id] : []),
      failures: [],
    };
  }
  return invoke<PositionBatchUpdateResult>("update_positions_partial", { inputs });
}

export async function deletePositions(ids: string[]): Promise<OverviewSnapshot> {
  if (!isTauri()) return refreshOverview();
  return invoke<OverviewSnapshot>("delete_positions", { ids });
}

export async function saveExportFile(filename: string, content: string): Promise<string> {
  if (isTauri()) return invoke<string>("export_json", { filename, content });
  const blob = new Blob([content], { type: "application/json;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  anchor.click();
  window.setTimeout(() => URL.revokeObjectURL(url), 0);
  return filename;
}

export async function showWindow(kind: "main" | "mini"): Promise<void> {
  if (!isTauri()) return;
  await invoke("show_window", { kind });
}

export async function minimizeCurrentWindow(): Promise<void> {
  if (!isTauri()) return;
  await invoke("minimize_current_window");
}

export async function hideCurrentWindow(): Promise<void> {
  if (!isTauri()) return;
  await invoke("hide_current_window");
}

export async function startDraggingCurrentWindow(): Promise<void> {
  if (!isTauri()) return;
  await getCurrentWindow().startDragging();
}

export async function toggleAlwaysOnTop(enabled: boolean): Promise<void> {
  if (!isTauri()) return;
  await getCurrentWindow().setAlwaysOnTop(enabled);
}

export async function subscribeOverview(
  handler: (snapshot: OverviewSnapshot) => void,
): Promise<UnlistenFn> {
  if (!isTauri()) return () => undefined;
  const currentWindow = getCurrentWindow();
  const eventName = currentWindow.label === "mini"
    ? "portfolio://mini-snapshot-updated"
    : "portfolio://main-snapshot-updated";
  return currentWindow.listen<OverviewSnapshot>(eventName, (event) => {
    handler(event.payload);
  });
}

export async function subscribePrivacy(handler: (enabled: boolean) => void): Promise<UnlistenFn> {
  if (!isTauri()) return () => undefined;
  return listen<boolean>("settings://privacy-changed", (event) => {
    handler(event.payload);
  });
}

export async function subscribeSelectedIndices(
  handler: (codes: string[]) => void,
): Promise<UnlistenFn> {
  if (!isTauri()) return () => undefined;
  return listen<string[]>("settings://indices-changed", (event) => {
    handler(event.payload);
  });
}

export async function subscribeMarketIndices(
  handler: (codes: string[]) => void,
): Promise<UnlistenFn> {
  if (!isTauri()) return () => undefined;
  return listen<string[]>("settings://market-indices-changed", (event) => {
    handler(event.payload);
  });
}
