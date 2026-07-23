import type { AssetSummary } from "@/types/contracts";

interface ExportFund {
  code: string;
  name: string;
  num: string;
  cost: string;
  totalCost: string;
}

function decimal(value: number, digits = 10) {
  if (!Number.isFinite(value)) return "0";
  return value.toFixed(digits).replace(/\.?0+$/, "");
}

export function createFundExport(assets: AssetSummary[]) {
  const grouped = new Map<string, ExportFund[]>();
  for (const asset of assets) {
    if (asset.kind !== "fund" || !asset.code) continue;
    const units = Number(asset.units);
    const totalCost = Number(asset.totalCost);
    const strategy = asset.strategy.trim() || "默认分组";
    const funds = grouped.get(strategy) ?? [];
    funds.push({
      code: asset.code,
      name: asset.name,
      num: decimal(units),
      cost: units > 0 && totalCost > 0 ? decimal(totalCost / units) : "0",
      totalCost: decimal(totalCost),
    });
    grouped.set(strategy, funds);
  }

  const exportedAt = new Date().toISOString();
  const payload = {
    version: "market-glass-1.0",
    exportedAt,
    description: "澄明行情基金持仓备份；可在“添加 / 导入”中重新导入",
    fundListGroup: [...grouped.entries()].map(([name, funds]) => ({ name, funds })),
  };
  const date = exportedAt.slice(0, 10);
  return {
    filename: `market-glass-持仓-${date}.json`,
    content: JSON.stringify(payload, null, 2),
    count: payload.fundListGroup.reduce((total, group) => total + group.funds.length, 0),
  };
}
