import { computed, ref } from "vue";
import { defineStore } from "pinia";
import {
  getBootstrap,
  deletePositions,
  importPositions,
  refreshOverview,
  setMarketIndices,
  setSelectedIndices,
  setPrivacyMode,
  subscribeOverview,
  subscribeMarketIndices,
  subscribePrivacy,
  subscribeSelectedIndices,
  updatePositionsPartial,
  upsertPosition,
} from "@/ipc/client";
import { mockBootstrap } from "@/ipc/mock";
import type {
  BootstrapPayload,
  OverviewSnapshot,
  PositionBatchUpdateResult,
  PositionInput,
} from "@/types/contracts";

export const usePortfolioStore = defineStore("portfolio", () => {
  const payload = ref<BootstrapPayload>(structuredClone(mockBootstrap));
  const initialized = ref(false);
  const refreshing = ref(false);
  const errorMessage = ref("");
  let cleanup: Array<() => void> = [];

  const overview = computed(() => payload.value.overview);
  const privacyMode = computed(() => payload.value.privacyMode);
  const selectedIndices = computed(() =>
    payload.value.selectedIndexCodes
      .map((code) => payload.value.overview.indices.find((index) => index.code === code))
      .filter((index): index is NonNullable<typeof index> => Boolean(index)),
  );

  async function initialize() {
    if (initialized.value) return;
    errorMessage.value = "";
    try {
      payload.value = await getBootstrap();
      const [stopOverview, stopPrivacy, stopIndices, stopMarketIndices] = await Promise.all([
        subscribeOverview((snapshot) => {
          payload.value.overview = snapshot;
          payload.value.sync.phase = "idle";
          payload.value.sync.message = "数据已同步";
          payload.value.sync.lastSuccessAt = snapshot.calculatedAt;
        }),
        subscribePrivacy((enabled) => {
          payload.value.privacyMode = enabled;
        }),
        subscribeSelectedIndices((codes) => {
          payload.value.selectedIndexCodes = codes;
        }),
        subscribeMarketIndices((codes) => {
          payload.value.marketIndexCodes = codes;
        }),
      ]);
      cleanup = [stopOverview, stopPrivacy, stopIndices, stopMarketIndices];
      initialized.value = true;
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : "初始化失败";
      payload.value.sync.phase = "offline";
      payload.value.sync.message = "暂时无法连接本地服务";
    }
  }

  async function refresh() {
    if (refreshing.value) return;
    refreshing.value = true;
    errorMessage.value = "";
    payload.value.sync.phase = "refreshing";
    payload.value.sync.message = "正在合并更新…";
    try {
      const next = await refreshOverview();
      applySnapshot(next);
      payload.value.sync.phase = "idle";
      payload.value.sync.message = "数据已同步";
      payload.value.sync.lastSuccessAt = next.calculatedAt;
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : "刷新失败";
      payload.value.sync.phase = "degraded";
      payload.value.sync.message = "更新失败，已保留上次数据";
    } finally {
      refreshing.value = false;
    }
  }

  async function togglePrivacy() {
    const next = !payload.value.privacyMode;
    payload.value.privacyMode = next;
    try {
      await setPrivacyMode(next);
    } catch (error) {
      payload.value.privacyMode = !next;
      errorMessage.value = error instanceof Error ? error.message : "隐私设置保存失败";
    }
  }

  async function updateSelectedIndices(codes: string[]) {
    const previous = [...payload.value.selectedIndexCodes];
    payload.value.selectedIndexCodes = codes.slice(0, 4);
    try {
      payload.value.selectedIndexCodes = await setSelectedIndices(codes);
    } catch (error) {
      payload.value.selectedIndexCodes = previous;
      errorMessage.value = error instanceof Error ? error.message : "自选指数保存失败";
      throw error;
    }
  }

  async function updateMarketIndices(codes: string[]) {
    const previous = [...payload.value.marketIndexCodes];
    payload.value.marketIndexCodes = [...new Set(codes)];
    try {
      payload.value.marketIndexCodes = await setMarketIndices(codes);
    } catch (error) {
      payload.value.marketIndexCodes = previous;
      errorMessage.value = error instanceof Error ? error.message : "大盘指数保存失败";
      throw error;
    }
  }

  async function savePosition(input: PositionInput) {
    errorMessage.value = "";
    try {
      const snapshot = await upsertPosition(input);
      applySnapshot(snapshot);
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : "资产保存失败";
      throw error;
    }
  }

  async function savePositions(inputs: PositionInput[]) {
    errorMessage.value = "";
    try {
      const snapshot = await importPositions(inputs);
      applySnapshot(snapshot);
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : "批量导入失败";
      throw error;
    }
  }

  async function updatePositions(inputs: PositionInput[]): Promise<PositionBatchUpdateResult> {
    errorMessage.value = "";
    try {
      const result = await updatePositionsPartial(inputs);
      applySnapshot(result.snapshot);
      return result;
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : "批量修改失败";
      throw error;
    }
  }

  async function removePositions(ids: string[]) {
    errorMessage.value = "";
    try {
      const snapshot = await deletePositions(ids);
      applySnapshot(snapshot);
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : "资产删除失败";
      throw error;
    }
  }

  function applySnapshot(snapshot: OverviewSnapshot) {
    payload.value.overview = snapshot;
  }

  function dispose() {
    cleanup.forEach((stop) => stop());
    cleanup = [];
    initialized.value = false;
  }

  return {
    payload,
    overview,
    privacyMode,
    selectedIndices,
    initialized,
    refreshing,
    errorMessage,
    initialize,
    refresh,
    togglePrivacy,
    updateSelectedIndices,
    updateMarketIndices,
    savePosition,
    savePositions,
    updatePositions,
    removePositions,
    applySnapshot,
    dispose,
  };
});
