export type ImportSource = "config";

export interface FundImportDraft {
  key: string;
  selected: boolean;
  code: string;
  name: string;
  units: string;
  totalCost: string;
  strategy: string;
  source: ImportSource;
  confidence?: number;
  warning?: string;
}

type UnknownRecord = Record<string, unknown>;

function record(value: unknown): UnknownRecord | undefined {
  return value !== null && typeof value === "object" && !Array.isArray(value)
    ? (value as UnknownRecord)
    : undefined;
}

function rows(value: unknown): UnknownRecord[] {
  return Array.isArray(value) ? value.map(record).filter(Boolean) as UnknownRecord[] : [];
}

function text(value: unknown): string {
  if (typeof value === "string") return value.trim();
  if (typeof value === "number" && Number.isFinite(value)) return String(value);
  return "";
}

function codeOf(item: UnknownRecord): string {
  const candidate = text(item.code ?? item.fundcode ?? item.fundCode ?? item.FCODE);
  return candidate.match(/\d{6}/)?.[0] ?? "";
}

function positiveNumber(value: unknown): number | undefined {
  const parsed = Number(text(value).replaceAll(",", ""));
  return Number.isFinite(parsed) && parsed > 0 ? parsed : undefined;
}

function decimal(value: number): string {
  return value.toFixed(4).replace(/\.?0+$/, "");
}

export function parseFundConfig(raw: string): FundImportDraft[] {
  let payload: unknown;
  try {
    payload = JSON.parse(raw);
  } catch {
    throw new Error("配置文件不是有效的 JSON");
  }
  const root = record(payload);
  if (!root) throw new Error("配置文件内容为空");

  const metadata = new Map<string, UnknownRecord>();
  for (const item of rows(root.dataList)) {
    const code = codeOf(item);
    if (code) metadata.set(code, item);
  }

  const drafts = new Map<string, FundImportDraft>();
  const add = (item: UnknownRecord, strategy: string) => {
    const code = codeOf(item);
    if (!code) return;
    const detail = metadata.get(code);
    const units = positiveNumber(item.num ?? item.units);
    const unitCost = positiveNumber(item.cost);
    const explicitTotalCost = positiveNumber(item.totalCost ?? item.total_cost);
    const totalCost = explicitTotalCost ?? (units && unitCost ? units * unitCost : 0);
    const name = text(item.name ?? detail?.name) || `基金 ${code}`;
    const warning = !units
      ? "未检测到持有份额；可勾选后作为 0 份额观察项"
      : !totalCost
        ? "未检测到成本价；总盈亏将显示为未录入成本"
        : undefined;
    drafts.set(code, {
      key: `config-${code}`,
      selected: Boolean(units),
      code,
      name,
      units: units ? decimal(units) : "0",
      totalCost: totalCost ? decimal(totalCost) : "0",
      strategy: strategy || "配置导入",
      source: "config",
      warning,
    });
  };

  for (const group of rows(root.fundListGroup)) {
    const strategy = text(group.name) || "默认分组";
    for (const item of rows(group.funds ?? group.list)) add(item, strategy);
  }
  for (const item of rows(root.fundListM)) add(item, "配置导入");
  for (const item of rows(root.fundList)) add(item, "配置导入");

  // dataList is the plug-in's transient quote cache, not its source of truth.
  // Only use it for legacy exports that contain no explicit holding collection.
  if (!drafts.size) {
    for (const item of rows(root.dataList)) add(item, "兼容导入");
  }

  if (!drafts.size) {
    throw new Error("没有识别到基金条目；目前支持自选基金助手 3.x 的 JSON 配置");
  }
  return [...drafts.values()];
}
