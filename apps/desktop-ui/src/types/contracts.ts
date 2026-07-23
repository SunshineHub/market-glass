export type AssetKind = "fund" | "advisory" | "cash";
export type DataNature = "realtime" | "estimated" | "confirmed" | "manual";
export type Freshness = "fresh" | "delayed" | "stale" | "offline";
export type SyncPhase = "idle" | "refreshing" | "degraded" | "offline";

export interface IndexQuote {
  code: string;
  name: string;
  value: number;
  change: number;
  changePercent: number;
  sparkline: number[];
  freshness: Freshness;
  updatedAt: string;
}

export interface IndexOption {
  code: string;
  name: string;
  region: string;
}

export interface FundMetadata {
  code: string;
  name: string;
  fundType?: string;
  company?: string;
  industry?: string;
  indexName?: string;
  latestNav?: string;
  navDate?: string;
  provider: string;
}

export interface AssetSummary {
  id: string;
  kind: AssetKind;
  code?: string;
  name: string;
  units: string;
  totalCost: string;
  strategy: string;
  provider: string;
  dataNature: DataNature;
  freshness: Freshness;
  currentNav?: number | null;
  currentValue: number;
  dayProfit: number;
  dayProfitPercent: number;
  totalProfit: number;
  totalProfitPercent: number;
  costKnown: boolean;
  updatedAt: string;
}

export interface AllocationSlice {
  key: string;
  label: string;
  value: number;
  color: string;
}

export interface OverviewSnapshot {
  totalAssets: number;
  dayProfit: number;
  dayProfitPercent: number;
  totalProfit: number;
  totalProfitPercent: number;
  indices: IndexQuote[];
  assets: AssetSummary[];
  allocation: AllocationSlice[];
  assetTrend: number[];
  calculatedAt: string;
}

export interface SyncStatus {
  phase: SyncPhase;
  message: string;
  lastSuccessAt?: string;
  nextRefreshAt?: string;
}

export interface BootstrapPayload {
  overview: OverviewSnapshot;
  privacyMode: boolean;
  selectedIndexCodes: string[];
  marketIndexCodes: string[];
  indexOptions: IndexOption[];
  sync: SyncStatus;
  demoMode: boolean;
}

export interface PositionInput {
  id?: string;
  kind: AssetKind;
  code?: string;
  name: string;
  units?: string;
  totalCost: string;
  manualValue?: string;
  manualDayPercent?: string;
  provider?: string;
  strategy?: string;
}

export interface PositionUpdateFailure {
  id: string;
  name: string;
  message: string;
}

export interface PositionBatchUpdateResult {
  snapshot: OverviewSnapshot;
  succeededIds: string[];
  failures: PositionUpdateFailure[];
}
