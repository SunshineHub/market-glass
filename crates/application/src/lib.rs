use std::{collections::HashMap, str::FromStr, sync::Arc};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use market_glass_domain::{AssetKind, FundQuote, Position, calculate_asset};
use rust_decimal::{Decimal, prelude::ToPrimitive};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("repository unavailable: {0}")]
    Unavailable(String),
    #[error("invalid stored data: {0}")]
    InvalidData(String),
}

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("provider request failed: {0}")]
    Request(String),
    #[error("provider returned invalid data: {0}")]
    InvalidData(String),
}

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error(transparent)]
    Provider(#[from] ProviderError),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

#[derive(Debug, Clone)]
pub struct IndexMarketQuote {
    pub code: String,
    pub name: String,
    pub value: f64,
    pub change: f64,
    pub change_percent: f64,
    pub source_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FundMetadata {
    pub code: String,
    pub name: String,
    pub fund_type: Option<String>,
    pub company: Option<String>,
    pub industry: Option<String>,
    pub index_name: Option<String>,
    pub latest_nav: Option<String>,
    pub nav_date: Option<String>,
    pub provider: String,
}

#[async_trait]
pub trait PortfolioRepository: Send + Sync {
    async fn list_positions(&self) -> Result<Vec<Position>, RepositoryError>;
    async fn add_position(&self, position: &Position) -> Result<(), RepositoryError>;
    async fn add_positions(&self, positions: &[Position]) -> Result<(), RepositoryError>;
    async fn delete_positions(&self, ids: &[Uuid]) -> Result<(), RepositoryError>;
    async fn upsert_position(&self, position: &Position) -> Result<(), RepositoryError>;
    async fn upsert_positions(&self, positions: &[Position]) -> Result<(), RepositoryError>;
    async fn get_string_setting(&self, key: &str, default: &str)
    -> Result<String, RepositoryError>;
    async fn set_string_setting(&self, key: &str, value: &str) -> Result<(), RepositoryError>;
    async fn get_bool_setting(&self, key: &str, default: bool) -> Result<bool, RepositoryError>;
    async fn set_bool_setting(&self, key: &str, value: bool) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait MarketDataProvider: Send + Sync {
    async fn fetch_indices(&self, codes: &[String])
    -> Result<Vec<IndexMarketQuote>, ProviderError>;
    async fn fetch_funds(&self, codes: &[String]) -> Result<Vec<FundQuote>, ProviderError>;
    async fn lookup_fund(&self, code: &str) -> Result<Option<FundMetadata>, ProviderError>;
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexQuoteDto {
    pub code: String,
    pub name: String,
    pub value: f64,
    pub change: f64,
    pub change_percent: f64,
    pub sparkline: Vec<f64>,
    pub freshness: &'static str,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetSummaryDto {
    pub id: String,
    pub kind: &'static str,
    pub code: Option<String>,
    pub name: String,
    pub units: String,
    pub total_cost: String,
    pub strategy: String,
    pub provider: String,
    pub data_nature: &'static str,
    pub freshness: &'static str,
    pub current_nav: Option<f64>,
    pub current_value: f64,
    pub day_profit: f64,
    pub day_profit_percent: f64,
    pub total_profit: f64,
    pub total_profit_percent: f64,
    pub cost_known: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationSliceDto {
    pub key: String,
    pub label: String,
    pub value: f64,
    pub color: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexOptionDto {
    pub code: &'static str,
    pub name: &'static str,
    pub region: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverviewSnapshotDto {
    pub total_assets: f64,
    pub day_profit: f64,
    pub day_profit_percent: f64,
    pub total_profit: f64,
    pub total_profit_percent: f64,
    pub indices: Vec<IndexQuoteDto>,
    pub assets: Vec<AssetSummaryDto>,
    pub allocation: Vec<AllocationSliceDto>,
    pub asset_trend: Vec<f64>,
    pub calculated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatusDto {
    pub phase: &'static str,
    pub message: &'static str,
    pub last_success_at: Option<String>,
    pub next_refresh_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapPayloadDto {
    pub overview: OverviewSnapshotDto,
    pub privacy_mode: bool,
    pub selected_index_codes: Vec<String>,
    pub market_index_codes: Vec<String>,
    pub index_options: Vec<IndexOptionDto>,
    pub sync: SyncStatusDto,
    pub demo_mode: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionInputDto {
    pub id: Option<String>,
    pub kind: String,
    pub code: Option<String>,
    pub name: String,
    pub units: Option<String>,
    #[serde(default)]
    pub total_cost: String,
    pub manual_value: Option<String>,
    pub manual_day_percent: Option<String>,
    pub provider: Option<String>,
    pub strategy: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionUpdateFailureDto {
    pub id: String,
    pub name: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionBatchUpdateResultDto {
    pub snapshot: OverviewSnapshotDto,
    pub succeeded_ids: Vec<String>,
    pub failures: Vec<PositionUpdateFailureDto>,
}

pub struct OverviewService {
    repository: Arc<dyn PortfolioRepository>,
    market: Arc<dyn MarketDataProvider>,
    market_indices: Vec<MarketIndexDefinition>,
}

#[derive(Debug, Clone)]
struct MarketIndexDefinition {
    secid: &'static str,
    code: &'static str,
    name: &'static str,
    region: &'static str,
}

impl OverviewService {
    pub fn new(
        repository: Arc<dyn PortfolioRepository>,
        market: Arc<dyn MarketDataProvider>,
    ) -> Self {
        Self {
            repository,
            market,
            market_indices: market_index_definitions(),
        }
    }

    pub async fn bootstrap(&self) -> Result<BootstrapPayloadDto, ApplicationError> {
        let privacy_mode = self
            .repository
            .get_bool_setting("privacy_mode", false)
            .await?;
        let selected_index_codes = self.selected_index_codes().await?;
        let market_index_codes = self.selected_market_index_codes().await?;
        let (overview, degraded) = self.overview().await?;
        let calculated_at = overview.calculated_at.clone();
        Ok(BootstrapPayloadDto {
            overview,
            privacy_mode,
            selected_index_codes,
            market_index_codes,
            index_options: self.index_options(),
            sync: SyncStatusDto {
                phase: if degraded { "degraded" } else { "idle" },
                message: if degraded {
                    "部分数据源异常，已保留可用数据"
                } else {
                    "数据已同步"
                },
                last_success_at: Some(calculated_at),
                next_refresh_at: None,
            },
            demo_mode: false,
        })
    }

    pub async fn lookup_fund(
        &self,
        code: String,
    ) -> Result<Option<FundMetadata>, ApplicationError> {
        let code = code.trim();
        if code.len() != 6 || !code.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(ApplicationError::InvalidInput(
                "fund code must contain six digits".into(),
            ));
        }
        self.market.lookup_fund(code).await.map_err(Into::into)
    }

    pub async fn overview(&self) -> Result<(OverviewSnapshotDto, bool), ApplicationError> {
        // Advisory platforms do not expose a stable, authorized constituent feed.
        // Keep legacy rows in local storage for compatibility, but do not present
        // partial manual totals as a supported portfolio feature.
        let mut positions = self
            .repository
            .list_positions()
            .await?
            .into_iter()
            .filter(|position| position.kind != AssetKind::Advisory)
            .collect::<Vec<_>>();
        let has_explicit_plugin_import = positions
            .iter()
            .any(|position| position.provider == "插件配置导入" && position.strategy != "历史持仓");
        if has_explicit_plugin_import {
            // Older builds incorrectly imported dataList (the extension's quote
            // cache) as extra holdings. Preserve those rows in SQLite for recovery,
            // but quarantine them from portfolio totals and display.
            positions.retain(|position| {
                !(position.provider == "插件配置导入" && position.strategy == "历史持仓")
            });
        }
        let fund_codes = positions
            .iter()
            .filter(|position| position.kind == AssetKind::Fund)
            .filter_map(|position| position.code.clone())
            .collect::<Vec<_>>();

        let market_codes = self
            .market_indices
            .iter()
            .map(|index| index.secid.to_owned())
            .collect::<Vec<_>>();
        let (indices_result, fund_result) = tokio::join!(
            self.market.fetch_indices(&market_codes),
            self.market.fetch_funds(&fund_codes),
        );

        let degraded = indices_result.is_err() || fund_result.is_err();
        let indices = indices_result.unwrap_or_default();
        let fund_quotes = fund_result.unwrap_or_default();
        let quote_map = fund_quotes
            .iter()
            .map(|quote| (quote.code.as_str(), quote))
            .collect::<HashMap<_, _>>();
        let performances = positions
            .iter()
            .map(|position| {
                let quote = position
                    .code
                    .as_deref()
                    .and_then(|code| quote_map.get(code).copied());
                calculate_asset(position, quote)
            })
            .collect::<Vec<_>>();

        let total_assets = performances
            .iter()
            .map(|item| item.current_value)
            .sum::<Decimal>();
        let day_profit = performances
            .iter()
            .map(|item| item.day_profit)
            .sum::<Decimal>();
        let total_profit = performances
            .iter()
            .map(|item| item.total_profit)
            .sum::<Decimal>();
        let previous_assets = total_assets - day_profit;
        let total_cost = positions
            .iter()
            .filter(|position| position.total_cost > Decimal::ZERO)
            .map(|position| position.total_cost)
            .sum::<Decimal>();
        let day_profit_percent = percent(day_profit, previous_assets);
        let total_profit_percent = percent(total_profit, total_cost);

        let assets = performances
            .iter()
            .zip(positions.iter())
            .map(|(item, position)| AssetSummaryDto {
                id: item.id.to_string(),
                kind: asset_kind(item.kind),
                code: item.code.clone(),
                name: item.name.clone(),
                units: position.units.normalize().to_string(),
                total_cost: position.total_cost.normalize().to_string(),
                strategy: item.strategy.clone(),
                provider: item.provider.clone(),
                data_nature: data_nature(item.nature),
                freshness: freshness(item.freshness),
                current_nav: item
                    .code
                    .as_deref()
                    .and_then(|code| quote_map.get(code).copied())
                    .map(|quote| decimal_to_f64(quote.current_nav)),
                current_value: decimal_to_f64(item.current_value),
                day_profit: decimal_to_f64(item.day_profit),
                day_profit_percent: decimal_to_f64(item.day_profit_percent),
                total_profit: decimal_to_f64(item.total_profit),
                total_profit_percent: decimal_to_f64(item.total_profit_percent),
                cost_known: item.cost_known,
                updated_at: item.source_time.to_rfc3339(),
            })
            .collect::<Vec<_>>();

        let allocation = allocation(&performances);
        let calculated_at = Utc::now().to_rfc3339();
        Ok((
            OverviewSnapshotDto {
                total_assets: decimal_to_f64(total_assets),
                day_profit: decimal_to_f64(day_profit),
                day_profit_percent: decimal_to_f64(day_profit_percent),
                total_profit: decimal_to_f64(total_profit),
                total_profit_percent: decimal_to_f64(total_profit_percent),
                indices: indices
                    .into_iter()
                    .map(|quote| IndexQuoteDto {
                        name: self
                            .market_indices
                            .iter()
                            .find(|item| item.code == quote.code)
                            .map(|item| item.name.to_owned())
                            .unwrap_or(quote.name),
                        code: quote.code,
                        value: quote.value,
                        change: quote.change,
                        change_percent: quote.change_percent,
                        sparkline: vec![quote.value - quote.change, quote.value],
                        freshness: "fresh",
                        updated_at: quote.source_time.to_rfc3339(),
                    })
                    .collect(),
                assets,
                allocation,
                asset_trend: Vec::new(),
                calculated_at,
            },
            degraded,
        ))
    }

    pub async fn set_privacy_mode(&self, enabled: bool) -> Result<(), ApplicationError> {
        self.repository
            .set_bool_setting("privacy_mode", enabled)
            .await?;
        Ok(())
    }

    pub async fn set_selected_indices(
        &self,
        codes: Vec<String>,
    ) -> Result<Vec<String>, ApplicationError> {
        let normalized = codes
            .into_iter()
            .filter(|code| self.market_indices.iter().any(|item| item.code == code))
            .fold(Vec::<String>::new(), |mut result, code| {
                if !result.contains(&code) && result.len() < 4 {
                    result.push(code);
                }
                result
            });
        if normalized.is_empty() {
            return Err(ApplicationError::InvalidInput(
                "select at least one market index".into(),
            ));
        }
        self.repository
            .set_string_setting("overview_index_codes", &normalized.join(","))
            .await?;
        Ok(normalized)
    }

    pub async fn set_market_indices(
        &self,
        codes: Vec<String>,
    ) -> Result<Vec<String>, ApplicationError> {
        let normalized = codes
            .into_iter()
            .filter(|code| self.market_indices.iter().any(|item| item.code == code))
            .fold(Vec::<String>::new(), |mut result, code| {
                if !result.contains(&code) {
                    result.push(code);
                }
                result
            });
        if normalized.is_empty() {
            return Err(ApplicationError::InvalidInput(
                "select at least one market index".into(),
            ));
        }
        self.repository
            .set_string_setting("market_index_codes", &normalized.join(","))
            .await?;
        Ok(normalized)
    }

    fn index_options(&self) -> Vec<IndexOptionDto> {
        self.market_indices
            .iter()
            .map(|item| IndexOptionDto {
                code: item.code,
                name: item.name,
                region: item.region,
            })
            .collect()
    }

    async fn selected_index_codes(&self) -> Result<Vec<String>, ApplicationError> {
        let stored = self
            .repository
            .get_string_setting("overview_index_codes", "000001,399001,000300,399006")
            .await?;
        let selected = stored
            .split(',')
            .map(str::trim)
            .filter(|code| self.market_indices.iter().any(|item| item.code == *code))
            .take(4)
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if selected.is_empty() {
            Ok(vec![
                "000001".into(),
                "399001".into(),
                "000300".into(),
                "399006".into(),
            ])
        } else {
            Ok(selected)
        }
    }

    async fn selected_market_index_codes(&self) -> Result<Vec<String>, ApplicationError> {
        let default_codes = self
            .market_indices
            .iter()
            .map(|item| item.code)
            .collect::<Vec<_>>()
            .join(",");
        let stored = self
            .repository
            .get_string_setting("market_index_codes", &default_codes)
            .await?;
        let selected = stored
            .split(',')
            .map(str::trim)
            .filter(|code| self.market_indices.iter().any(|item| item.code == *code))
            .fold(Vec::<String>::new(), |mut result, code| {
                if !result.iter().any(|item| item == code) {
                    result.push(code.to_owned());
                }
                result
            });
        if selected.is_empty() {
            Ok(self
                .market_indices
                .iter()
                .map(|item| item.code.to_owned())
                .collect())
        } else {
            Ok(selected)
        }
    }

    pub async fn upsert_position(
        &self,
        input: PositionInputDto,
    ) -> Result<OverviewSnapshotDto, ApplicationError> {
        let is_existing_edit = input.id.is_some();
        let position = parse_position(input)?;
        if is_existing_edit {
            self.repository.upsert_position(&position).await?;
        } else {
            self.repository.add_position(&position).await?;
        }
        self.overview().await.map(|result| result.0)
    }

    pub async fn import_positions(
        &self,
        inputs: Vec<PositionInputDto>,
    ) -> Result<OverviewSnapshotDto, ApplicationError> {
        if inputs.is_empty() {
            return Err(ApplicationError::InvalidInput(
                "at least one position is required".into(),
            ));
        }
        if inputs.len() > 500 {
            return Err(ApplicationError::InvalidInput(
                "a single import is limited to 500 positions".into(),
            ));
        }
        let positions = inputs
            .into_iter()
            .map(parse_position)
            .collect::<Result<Vec<_>, _>>()?;
        self.repository.add_positions(&positions).await?;
        self.overview().await.map(|result| result.0)
    }

    pub async fn update_positions_partial(
        &self,
        inputs: Vec<PositionInputDto>,
    ) -> Result<PositionBatchUpdateResultDto, ApplicationError> {
        if inputs.is_empty() {
            return Err(ApplicationError::InvalidInput(
                "at least one position is required".into(),
            ));
        }
        if inputs.len() > 500 {
            return Err(ApplicationError::InvalidInput(
                "a single update is limited to 500 positions".into(),
            ));
        }

        let mut succeeded_ids = Vec::new();
        let mut failures = Vec::new();
        for input in inputs {
            let requested_id = input.id.clone().unwrap_or_default();
            let requested_name = input.name.clone();
            let position = match parse_position(input) {
                Ok(position) if !requested_id.is_empty() => position,
                Ok(_) => {
                    failures.push(PositionUpdateFailureDto {
                        id: requested_id,
                        name: requested_name,
                        message: "缺少资产标识，请重新打开批量修改".into(),
                    });
                    continue;
                }
                Err(error) => {
                    failures.push(PositionUpdateFailureDto {
                        id: requested_id,
                        name: requested_name,
                        message: error.to_string(),
                    });
                    continue;
                }
            };

            match self.repository.upsert_position(&position).await {
                Ok(()) => succeeded_ids.push(position.id.to_string()),
                Err(error) => failures.push(PositionUpdateFailureDto {
                    id: requested_id,
                    name: requested_name,
                    message: error.to_string(),
                }),
            }
        }

        let snapshot = self.overview().await?.0;
        Ok(PositionBatchUpdateResultDto {
            snapshot,
            succeeded_ids,
            failures,
        })
    }

    pub async fn delete_positions(
        &self,
        ids: Vec<String>,
    ) -> Result<OverviewSnapshotDto, ApplicationError> {
        if ids.is_empty() {
            return Err(ApplicationError::InvalidInput(
                "at least one position id is required".into(),
            ));
        }
        if ids.len() > 2_000 {
            return Err(ApplicationError::InvalidInput(
                "a single delete is limited to 2000 positions".into(),
            ));
        }
        let ids = ids
            .into_iter()
            .map(|id| {
                Uuid::parse_str(&id)
                    .map_err(|error| ApplicationError::InvalidInput(error.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.repository.delete_positions(&ids).await?;
        self.overview().await.map(|result| result.0)
    }
}

fn parse_position(input: PositionInputDto) -> Result<Position, ApplicationError> {
    let kind = match input.kind.as_str() {
        "fund" => AssetKind::Fund,
        "advisory" => AssetKind::Advisory,
        "cash" => AssetKind::Cash,
        other => {
            return Err(ApplicationError::InvalidInput(format!(
                "unknown asset kind: {other}"
            )));
        }
    };
    if input.name.trim().is_empty() {
        return Err(ApplicationError::InvalidInput("name is required".into()));
    }
    let fund_code = input.code.as_deref().unwrap_or_default().trim();
    if kind == AssetKind::Fund
        && (fund_code.len() != 6 || !fund_code.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return Err(ApplicationError::InvalidInput(
            "fund code must contain six digits".into(),
        ));
    }

    Ok(Position {
        id: input
            .id
            .as_deref()
            .map(Uuid::parse_str)
            .transpose()
            .map_err(|error| ApplicationError::InvalidInput(error.to_string()))?
            .unwrap_or_else(Uuid::new_v4),
        kind,
        code: input
            .code
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty()),
        name: input.name.trim().to_owned(),
        units: parse_non_negative_or_zero(input.units.as_deref(), "units")?,
        total_cost: parse_non_negative_or_zero(Some(&input.total_cost), "totalCost")?,
        manual_value: parse_optional_non_negative_decimal(
            input.manual_value.as_deref(),
            "manualValue",
        )?,
        manual_day_percent: parse_optional_decimal(
            input.manual_day_percent.as_deref(),
            "manualDayPercent",
        )?,
        provider: input.provider.unwrap_or_else(|| "manual".into()),
        strategy: input.strategy.unwrap_or_else(|| "未分类".into()),
    })
}

fn parse_decimal(value: &str, field: &str) -> Result<Decimal, ApplicationError> {
    Decimal::from_str(value.trim())
        .map_err(|_| ApplicationError::InvalidInput(format!("{field} is not a decimal")))
}

fn parse_non_negative_or_zero(
    value: Option<&str>,
    field: &str,
) -> Result<Decimal, ApplicationError> {
    let value = value
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("0");
    let parsed = parse_decimal(value, field)?;
    if parsed < Decimal::ZERO {
        return Err(ApplicationError::InvalidInput(format!(
            "{field} cannot be negative"
        )));
    }
    Ok(parsed)
}

fn parse_optional_non_negative_decimal(
    value: Option<&str>,
    field: &str,
) -> Result<Option<Decimal>, ApplicationError> {
    let parsed = parse_optional_decimal(value, field)?;
    if parsed.is_some_and(|value| value < Decimal::ZERO) {
        return Err(ApplicationError::InvalidInput(format!(
            "{field} cannot be negative"
        )));
    }
    Ok(parsed)
}

fn parse_optional_decimal(
    value: Option<&str>,
    field: &str,
) -> Result<Option<Decimal>, ApplicationError> {
    value
        .filter(|value| !value.trim().is_empty())
        .map(|value| parse_decimal(value, field))
        .transpose()
}

fn allocation(performances: &[market_glass_domain::AssetPerformance]) -> Vec<AllocationSliceDto> {
    let mut industries = HashMap::<String, Decimal>::new();
    for item in performances
        .iter()
        .filter(|item| item.current_value > Decimal::ZERO)
    {
        *industries.entry(industry_label(item)).or_default() += item.current_value;
    }
    let allocated_total = industries.values().copied().sum::<Decimal>();
    if allocated_total <= Decimal::ZERO {
        return Vec::new();
    }
    let mut ranked = industries.into_iter().collect::<Vec<_>>();
    ranked.sort_by(|left, right| right.1.cmp(&left.1));

    if ranked.len() > 6 {
        let other = ranked.drain(5..).map(|(_, amount)| amount).sum::<Decimal>();
        ranked.push(("其他".into(), other));
    }

    const COLORS: [&str; 6] = [
        "#ff7468", "#687ff7", "#42b69d", "#e7a23a", "#a36ee8", "#4da5e8",
    ];
    ranked
        .into_iter()
        .enumerate()
        .map(|(index, (label, amount))| AllocationSliceDto {
            key: format!("industry-{index}"),
            label,
            value: decimal_to_f64(percent(amount, allocated_total)),
            color: COLORS[index.min(COLORS.len() - 1)],
        })
        .collect()
}

fn industry_label(item: &market_glass_domain::AssetPerformance) -> String {
    if item.kind == AssetKind::Cash {
        return "现金管理".into();
    }
    let strategy = item.strategy.trim();
    let generic = [
        "",
        "未分类",
        "默认分组",
        "配置导入",
        "历史持仓",
        "公募基金",
        "投顾组合",
    ];
    if !generic.contains(&strategy) {
        return strategy.to_owned();
    }
    if item.kind == AssetKind::Advisory {
        return "投顾组合".into();
    }

    let name = item.name.as_str();
    let groups: [(&str, &[&str]); 10] = [
        (
            "科技",
            &[
                "科技",
                "电子",
                "芯片",
                "半导体",
                "计算机",
                "通信",
                "人工智能",
                "数字",
            ],
        ),
        ("医药健康", &["医药", "医疗", "健康", "生物", "创新药"]),
        ("大消费", &["消费", "食品", "白酒", "农业", "家电", "旅游"]),
        (
            "新能源",
            &["新能源", "光伏", "电池", "汽车", "风电", "储能"],
        ),
        ("金融", &["金融", "银行", "证券", "保险"]),
        (
            "周期资源",
            &["有色", "煤炭", "钢铁", "资源", "化工", "能源"],
        ),
        ("国防军工", &["军工", "国防", "航空", "航天"]),
        (
            "海外市场",
            &["纳斯达克", "标普", "恒生", "海外", "全球", "QDII"],
        ),
        ("红利低波", &["红利", "低波", "价值"]),
        ("固收", &["债", "货币", "理财", "同业存单"]),
    ];
    groups
        .iter()
        .find(|(_, keywords)| keywords.iter().any(|keyword| name.contains(keyword)))
        .map(|(label, _)| (*label).to_owned())
        .unwrap_or_else(|| "综合配置".into())
}

fn market_index_definitions() -> Vec<MarketIndexDefinition> {
    vec![
        MarketIndexDefinition {
            secid: "1.000001",
            code: "000001",
            name: "上证指数",
            region: "中国内地",
        },
        MarketIndexDefinition {
            secid: "0.399001",
            code: "399001",
            name: "深证成指",
            region: "中国内地",
        },
        MarketIndexDefinition {
            secid: "1.000300",
            code: "000300",
            name: "沪深 300",
            region: "中国内地",
        },
        MarketIndexDefinition {
            secid: "0.399006",
            code: "399006",
            name: "创业板指",
            region: "中国内地",
        },
        MarketIndexDefinition {
            secid: "100.DJIA",
            code: "DJIA",
            name: "道琼斯",
            region: "美国",
        },
        MarketIndexDefinition {
            secid: "100.NDX",
            code: "NDX",
            name: "纳斯达克",
            region: "美国",
        },
        MarketIndexDefinition {
            secid: "100.SPX",
            code: "SPX",
            name: "标普 500",
            region: "美国",
        },
        MarketIndexDefinition {
            secid: "100.HSI",
            code: "HSI",
            name: "恒生指数",
            region: "中国香港",
        },
        MarketIndexDefinition {
            secid: "100.N225",
            code: "N225",
            name: "日经 225",
            region: "亚太",
        },
        MarketIndexDefinition {
            secid: "100.KOSPI",
            code: "KOSPI",
            name: "韩国 KOSPI",
            region: "亚太",
        },
        MarketIndexDefinition {
            secid: "100.TWII",
            code: "TWII",
            name: "台湾加权",
            region: "亚太",
        },
        MarketIndexDefinition {
            secid: "100.AS51",
            code: "AS51",
            name: "澳洲标普 200",
            region: "亚太",
        },
        MarketIndexDefinition {
            secid: "100.SENSEX",
            code: "SENSEX",
            name: "印度孟买 30",
            region: "亚太",
        },
        MarketIndexDefinition {
            secid: "100.FTSE",
            code: "FTSE",
            name: "英国富时 100",
            region: "欧洲",
        },
        MarketIndexDefinition {
            secid: "100.GDAXI",
            code: "GDAXI",
            name: "德国 DAX",
            region: "欧洲",
        },
        MarketIndexDefinition {
            secid: "100.FCHI",
            code: "FCHI",
            name: "法国 CAC 40",
            region: "欧洲",
        },
    ]
}

fn percent(numerator: Decimal, denominator: Decimal) -> Decimal {
    if denominator.is_zero() {
        Decimal::ZERO
    } else {
        numerator / denominator * Decimal::new(100, 0)
    }
}

fn decimal_to_f64(value: Decimal) -> f64 {
    value.round_dp(2).to_f64().unwrap_or_default()
}

fn asset_kind(value: AssetKind) -> &'static str {
    match value {
        AssetKind::Fund => "fund",
        AssetKind::Advisory => "advisory",
        AssetKind::Cash => "cash",
    }
}

fn data_nature(value: market_glass_domain::DataNature) -> &'static str {
    match value {
        market_glass_domain::DataNature::Realtime => "realtime",
        market_glass_domain::DataNature::Estimated => "estimated",
        market_glass_domain::DataNature::Confirmed => "confirmed",
        market_glass_domain::DataNature::Manual => "manual",
    }
}

fn freshness(value: market_glass_domain::Freshness) -> &'static str {
    match value {
        market_glass_domain::Freshness::Fresh => "fresh",
        market_glass_domain::Freshness::Delayed => "delayed",
        market_glass_domain::Freshness::Stale => "stale",
        market_glass_domain::Freshness::Offline => "offline",
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    #[derive(Default)]
    struct TestRepository {
        positions: Mutex<Vec<Position>>,
    }

    #[async_trait]
    impl PortfolioRepository for TestRepository {
        async fn list_positions(&self) -> Result<Vec<Position>, RepositoryError> {
            Ok(self.positions.lock().unwrap().clone())
        }

        async fn upsert_position(&self, position: &Position) -> Result<(), RepositoryError> {
            let mut positions = self.positions.lock().unwrap();
            if let Some(existing) = positions.iter_mut().find(|item| item.id == position.id) {
                *existing = position.clone();
            } else {
                positions.push(position.clone());
            }
            Ok(())
        }

        async fn add_position(&self, _: &Position) -> Result<(), RepositoryError> {
            unimplemented!()
        }
        async fn add_positions(&self, _: &[Position]) -> Result<(), RepositoryError> {
            unimplemented!()
        }
        async fn delete_positions(&self, _: &[Uuid]) -> Result<(), RepositoryError> {
            unimplemented!()
        }
        async fn upsert_positions(&self, _: &[Position]) -> Result<(), RepositoryError> {
            unimplemented!()
        }
        async fn get_string_setting(&self, _: &str, _: &str) -> Result<String, RepositoryError> {
            unimplemented!()
        }
        async fn set_string_setting(&self, _: &str, _: &str) -> Result<(), RepositoryError> {
            unimplemented!()
        }
        async fn get_bool_setting(&self, _: &str, _: bool) -> Result<bool, RepositoryError> {
            unimplemented!()
        }
        async fn set_bool_setting(&self, _: &str, _: bool) -> Result<(), RepositoryError> {
            unimplemented!()
        }
    }

    struct TestMarket;

    #[async_trait]
    impl MarketDataProvider for TestMarket {
        async fn fetch_indices(
            &self,
            _: &[String],
        ) -> Result<Vec<IndexMarketQuote>, ProviderError> {
            Ok(Vec::new())
        }

        async fn fetch_funds(&self, _: &[String]) -> Result<Vec<FundQuote>, ProviderError> {
            Ok(Vec::new())
        }

        async fn lookup_fund(&self, _: &str) -> Result<Option<FundMetadata>, ProviderError> {
            Ok(None)
        }
    }

    fn fund_input(id: Uuid, name: &str) -> PositionInputDto {
        PositionInputDto {
            id: Some(id.to_string()),
            kind: "fund".into(),
            code: Some("005827".into()),
            name: name.into(),
            units: Some("100".into()),
            total_cost: "200".into(),
            manual_value: None,
            manual_day_percent: None,
            provider: Some("test".into()),
            strategy: Some("科技".into()),
        }
    }

    #[tokio::test]
    async fn partial_batch_update_keeps_valid_items_when_another_is_invalid() {
        let repository = Arc::new(TestRepository::default());
        let service = OverviewService::new(repository.clone(), Arc::new(TestMarket));
        let valid_id = Uuid::new_v4();
        let invalid_id = Uuid::new_v4();

        let result = service
            .update_positions_partial(vec![
                fund_input(valid_id, "有效基金"),
                fund_input(invalid_id, ""),
            ])
            .await
            .unwrap();

        assert_eq!(result.succeeded_ids, vec![valid_id.to_string()]);
        assert_eq!(result.failures.len(), 1);
        assert_eq!(result.failures[0].id, invalid_id.to_string());
        let saved = repository.list_positions().await.unwrap();
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].id, valid_id);
    }
}
