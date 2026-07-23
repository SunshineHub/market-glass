use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AssetKind {
    Fund,
    Advisory,
    Cash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DataNature {
    Realtime,
    Estimated,
    Confirmed,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Freshness {
    Fresh,
    Delayed,
    Stale,
    Offline,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub id: Uuid,
    pub kind: AssetKind,
    pub code: Option<String>,
    pub name: String,
    pub units: Decimal,
    pub total_cost: Decimal,
    pub manual_value: Option<Decimal>,
    pub manual_day_percent: Option<Decimal>,
    pub provider: String,
    pub strategy: String,
}

#[derive(Debug, Clone)]
pub struct FundQuote {
    pub code: String,
    pub name: String,
    pub current_nav: Decimal,
    pub previous_nav: Decimal,
    pub change_percent: Decimal,
    pub nature: DataNature,
    pub freshness: Freshness,
    pub provider: String,
    pub source_time: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AssetPerformance {
    pub id: Uuid,
    pub kind: AssetKind,
    pub code: Option<String>,
    pub name: String,
    pub strategy: String,
    pub provider: String,
    pub nature: DataNature,
    pub freshness: Freshness,
    pub current_value: Decimal,
    pub day_profit: Decimal,
    pub day_profit_percent: Decimal,
    pub total_profit: Decimal,
    pub total_profit_percent: Decimal,
    pub cost_known: bool,
    pub source_time: DateTime<Utc>,
}

pub fn calculate_asset(position: &Position, quote: Option<&FundQuote>) -> AssetPerformance {
    let now = Utc::now();
    let (current_value, previous_value, day_percent, nature, freshness, provider, source_time) =
        match position.kind {
            AssetKind::Fund => {
                if let Some(quote) = quote {
                    (
                        quote.current_nav * position.units,
                        quote.previous_nav * position.units,
                        quote.change_percent,
                        quote.nature,
                        quote.freshness,
                        quote.provider.clone(),
                        quote.source_time,
                    )
                } else {
                    let current = position.manual_value.unwrap_or(Decimal::ZERO);
                    (
                        current,
                        current,
                        Decimal::ZERO,
                        DataNature::Manual,
                        Freshness::Offline,
                        position.provider.clone(),
                        now,
                    )
                }
            }
            AssetKind::Advisory | AssetKind::Cash => {
                let current = position.manual_value.unwrap_or(Decimal::ZERO);
                let rate = position.manual_day_percent.unwrap_or(Decimal::ZERO);
                let denominator = Decimal::ONE + rate / Decimal::new(100, 0);
                let previous = if denominator > Decimal::ZERO {
                    current / denominator
                } else {
                    current
                };
                (
                    current,
                    previous,
                    rate,
                    DataNature::Manual,
                    Freshness::Delayed,
                    position.provider.clone(),
                    now,
                )
            }
        };

    let day_profit = current_value - previous_value;
    let cost_known = position.total_cost > Decimal::ZERO;
    let total_profit = if cost_known {
        current_value - position.total_cost
    } else {
        Decimal::ZERO
    };
    let total_profit_percent = if cost_known {
        total_profit / position.total_cost * Decimal::new(100, 0)
    } else {
        Decimal::ZERO
    };

    AssetPerformance {
        id: position.id,
        kind: position.kind,
        code: position.code.clone(),
        name: quote
            .map(|quote| quote.name.clone())
            .unwrap_or_else(|| position.name.clone()),
        strategy: position.strategy.clone(),
        provider,
        nature,
        freshness,
        current_value,
        day_profit,
        day_profit_percent: day_percent,
        total_profit,
        total_profit_percent,
        cost_known,
        source_time,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn calculates_fund_day_and_total_profit_without_binary_float() {
        let position = Position {
            id: Uuid::new_v4(),
            kind: AssetKind::Fund,
            code: Some("001618".into()),
            name: "测试基金".into(),
            units: Decimal::new(100_000, 3),
            total_cost: Decimal::new(20_000, 2),
            manual_value: None,
            manual_day_percent: None,
            provider: "manual".into(),
            strategy: "equity".into(),
        };
        let quote = FundQuote {
            code: "001618".into(),
            name: "测试基金".into(),
            current_nav: Decimal::new(250, 2),
            previous_nav: Decimal::new(240, 2),
            change_percent: Decimal::new(417, 2),
            nature: DataNature::Estimated,
            freshness: Freshness::Fresh,
            provider: "sina".into(),
            source_time: Utc::now(),
        };

        let result = calculate_asset(&position, Some(&quote));
        assert_eq!(result.current_value, Decimal::new(25_000, 2));
        assert_eq!(result.day_profit, Decimal::new(1_000, 2));
        assert_eq!(result.total_profit, Decimal::new(5_000, 2));
        assert_eq!(result.total_profit_percent, Decimal::new(25, 0));
    }

    #[test]
    fn derives_manual_advisory_day_profit_from_percent() {
        let position = Position {
            id: Uuid::new_v4(),
            kind: AssetKind::Advisory,
            code: None,
            name: "投顾".into(),
            units: Decimal::ZERO,
            total_cost: Decimal::new(10_000, 0),
            manual_value: Some(Decimal::new(10_100, 0)),
            manual_day_percent: Some(Decimal::ONE),
            provider: "manual".into(),
            strategy: "balanced".into(),
        };

        let result = calculate_asset(&position, None);
        assert_eq!(result.day_profit, Decimal::new(100, 0));
        assert_eq!(result.day_profit_percent, Decimal::ONE);
    }

    #[test]
    fn does_not_treat_unknown_cost_as_full_profit() {
        let position = Position {
            id: Uuid::new_v4(),
            kind: AssetKind::Fund,
            code: Some("001618".into()),
            name: "未知成本基金".into(),
            units: Decimal::new(100, 0),
            total_cost: Decimal::ZERO,
            manual_value: None,
            manual_day_percent: None,
            provider: "import".into(),
            strategy: "fund".into(),
        };
        let quote = FundQuote {
            code: "001618".into(),
            name: "未知成本基金".into(),
            current_nav: Decimal::new(250, 2),
            previous_nav: Decimal::new(240, 2),
            change_percent: Decimal::new(417, 2),
            nature: DataNature::Estimated,
            freshness: Freshness::Fresh,
            provider: "sina".into(),
            source_time: Utc::now(),
        };

        let result = calculate_asset(&position, Some(&quote));
        assert_eq!(result.current_value, Decimal::new(25_000, 2));
        assert_eq!(result.total_profit, Decimal::ZERO);
        assert_eq!(result.total_profit_percent, Decimal::ZERO);
        assert!(!result.cost_known);
    }
}
