use std::{collections::HashMap, str::FromStr, time::Duration};

use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};
use futures::future::join_all;
use market_glass_application::{FundMetadata, IndexMarketQuote, MarketDataProvider, ProviderError};
use market_glass_domain::{DataNature, Freshness, FundQuote};
use reqwest::{Client, header};
use rust_decimal::Decimal;
use serde_json::Value;

const MOBILE_USER_AGENT: &str =
    "Mozilla/5.0 (Linux; Android 13; Pixel 7) AppleWebKit/537.36 Chrome/124 Mobile Safari/537.36";

#[derive(Clone)]
pub struct HybridMarketDataProvider {
    client: Client,
}

#[derive(Debug, Clone)]
struct OfficialFund {
    code: String,
    name: String,
    nav: Decimal,
    nav_change_percent: Decimal,
    nav_date: Option<NaiveDate>,
    source_time: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct SinaEstimate {
    code: String,
    estimated_nav: Decimal,
    change_percent: Decimal,
    estimate_date: Option<NaiveDate>,
    source_time: DateTime<Utc>,
}

impl HybridMarketDataProvider {
    pub fn new() -> Result<Self, ProviderError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(12))
            .http1_only()
            .user_agent(MOBILE_USER_AGENT)
            .default_headers({
                let mut headers = header::HeaderMap::new();
                headers.insert(
                    header::ACCEPT,
                    header::HeaderValue::from_static("application/json,text/plain,*/*"),
                );
                headers
            })
            .build()
            .map_err(|error| ProviderError::Request(format!("{error:?}")))?;
        Ok(Self { client })
    }

    async fn official_funds(&self, codes: &[String]) -> Result<Vec<OfficialFund>, ProviderError> {
        if codes.is_empty() {
            return Ok(Vec::new());
        }
        let url = format!(
            "https://fundmobapi.eastmoney.com/FundMNewApi/FundMNFInfo?pageIndex=1&pageSize=200&plat=Android&appType=ttjj&product=EFund&Version=1&deviceid=market-glass&Fcodes={}",
            codes.join(",")
        );
        let payload = self.get_json(&url).await?;
        let values = payload
            .get("Datas")
            .and_then(Value::as_array)
            .ok_or_else(|| ProviderError::InvalidData("东方财富基金详情缺少 Datas".into()))?;

        Ok(values
            .iter()
            .filter_map(|value| {
                let nav_date = string_field(value, "PDATE").and_then(|date| parse_date(&date));
                Some(OfficialFund {
                    code: string_field(value, "FCODE")?,
                    name: string_field(value, "SHORTNAME").unwrap_or_else(|| "未命名基金".into()),
                    nav: decimal_field(value, "NAV")?,
                    nav_change_percent: decimal_field(value, "NAVCHGRT").unwrap_or(Decimal::ZERO),
                    nav_date,
                    source_time: nav_date
                        .and_then(|date| china_time(date, "15:00"))
                        .unwrap_or_else(Utc::now),
                })
            })
            .collect())
    }

    async fn fund_metadata(&self, code: &str) -> Result<Option<FundMetadata>, ProviderError> {
        let url = format!(
            "https://fundmobapi.eastmoney.com/FundMNewApi/FundMNNBasicInformation?version=6.2.4&plat=Android&appType=ttjj&FCODE={code}&onFundCache=3&keeeeeyparam=FCODE&deviceid=market-glass&igggggnoreburst=true&product=EFund&MobileKey=market-glass"
        );
        let payload = self.get_json(&url).await?;
        let value = match payload.get("Datas") {
            Some(Value::Object(_)) => payload.get("Datas"),
            Some(Value::Array(values)) => values.first(),
            _ => None,
        };
        Ok(value.and_then(parse_fund_metadata))
    }

    async fn sina_estimate(&self, code: &str) -> Option<SinaEstimate> {
        let url = format!(
            "https://stock.finance.sina.com.cn/fundInfo/api/openapi.php/FdFundService.getEstimateNetworthPic?symbol={code}"
        );
        let payload = self.get_json(&url).await.ok()?;
        let latest = payload
            .pointer("/result/data/networth")?
            .as_array()?
            .last()?;
        let ratio = decimal_field(latest, "growthrate2")?;
        let estimate_date = string_field(latest, "pre_date").and_then(|date| parse_date(&date));
        let estimate_time = string_field(latest, "min_time").unwrap_or_else(|| "15:00".into());
        Some(SinaEstimate {
            code: string_field(latest, "symbol").unwrap_or_else(|| code.to_owned()),
            estimated_nav: decimal_field(latest, "pre_nav2")?,
            change_percent: ratio * Decimal::new(100, 0),
            estimate_date,
            source_time: estimate_date
                .and_then(|date| china_time(date, &estimate_time))
                .unwrap_or_else(Utc::now),
        })
    }

    async fn holdings_estimate(&self, code: &str) -> Option<Decimal> {
        let mut payload = self.holdings_payload(code).await?;
        let mut stocks = payload
            .pointer("/Datas/fundStocks")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        if stocks.is_empty() {
            let etf_code = payload
                .pointer("/Datas/ETFCODE")
                .and_then(value_to_string)
                .filter(|value| !value.is_empty())?;
            payload = self.holdings_payload(&etf_code).await?;
            stocks = payload
                .pointer("/Datas/fundStocks")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
        }
        if stocks.is_empty() {
            return None;
        }

        let secids = stocks
            .iter()
            .filter_map(|stock| {
                let exchange = string_field(stock, "NEWTEXCH")?;
                let code = string_field(stock, "GPDM")?;
                Some(format!("{exchange}.{code}"))
            })
            .collect::<Vec<_>>();
        if secids.is_empty() {
            return None;
        }

        let url = format!(
            "https://push2.eastmoney.com/api/qt/ulist.np/get?fields=f1,f2,f3,f4,f12,f13,f14,f292&fltt=2&secids={}&deviceid=Wap&plat=Wap&product=EFund&version=2.0.0&Uid=",
            secids.join(",")
        );
        let quotes = self
            .get_json(&url)
            .await
            .ok()?
            .pointer("/data/diff")?
            .as_array()?
            .clone();

        let total_weight = stocks
            .iter()
            .filter_map(|stock| decimal_field(stock, "JZBL"))
            .filter(|weight| *weight > Decimal::ZERO)
            .sum::<Decimal>();
        if total_weight.is_zero() {
            return None;
        }

        let weighted_change = stocks
            .iter()
            .zip(quotes.iter())
            .filter_map(|(stock, quote)| {
                let weight = decimal_field(stock, "JZBL")?;
                let change = decimal_field(quote, "f3")?;
                (weight > Decimal::ZERO).then_some(weight / total_weight * change)
            })
            .sum::<Decimal>();
        Some(weighted_change.round_dp(2))
    }

    async fn holdings_payload(&self, code: &str) -> Option<Value> {
        let url = format!(
            "https://fundmobapi.eastmoney.com/FundMNewApi/FundMNInverstPosition?FCODE={code}&deviceid=Wap&plat=Wap&product=EFund&version=2.0.0&Uid="
        );
        let payload = self.get_json(&url).await.ok()?;
        payload.get("Datas")?;
        Some(payload)
    }

    async fn eastmoney_indices(
        &self,
        codes: &[String],
    ) -> Result<Vec<IndexMarketQuote>, ProviderError> {
        let url = format!(
            "https://push2.eastmoney.com/api/qt/ulist.np/get?fltt=2&fields=f2,f3,f4,f12,f13,f14&secids={}",
            codes.join(",")
        );
        let payload = self.get_json(&url).await?;
        let values = payload
            .pointer("/data/diff")
            .and_then(Value::as_array)
            .ok_or_else(|| ProviderError::InvalidData("东方财富指数行情缺少 data.diff".into()))?;
        let now = Utc::now();
        Ok(values
            .iter()
            .filter_map(|value| {
                Some(IndexMarketQuote {
                    code: string_field(value, "f12")?,
                    name: string_field(value, "f14")?,
                    value: decimal_field(value, "f2")?.to_string().parse().ok()?,
                    change: decimal_field(value, "f4")?.to_string().parse().ok()?,
                    change_percent: decimal_field(value, "f3")?.to_string().parse().ok()?,
                    source_time: now,
                })
            })
            .collect())
    }

    async fn sina_indices(&self, codes: &[String]) -> Result<Vec<IndexMarketQuote>, ProviderError> {
        let mappings = codes
            .iter()
            .filter_map(|code| index_mapping(code).map(|mapping| (code, mapping)))
            .collect::<Vec<_>>();
        if mappings.is_empty() {
            return Ok(Vec::new());
        }
        let symbols = mappings
            .iter()
            .map(|(_, (symbol, _, _))| *symbol)
            .collect::<Vec<_>>()
            .join(",");
        let url = format!("https://hq.sinajs.cn/list={symbols}");
        let bytes = self
            .client
            .get(url)
            .header(header::REFERER, "https://finance.sina.com.cn/")
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|error| ProviderError::Request(format!("{error:?}")))?
            .bytes()
            .await
            .map_err(|error| ProviderError::InvalidData(error.to_string()))?;
        let payload = String::from_utf8_lossy(&bytes);
        let now = Utc::now();

        Ok(payload
            .lines()
            .filter_map(|line| {
                let symbol = line.strip_prefix("var hq_str_")?.split('=').next()?.trim();
                let (_, (_, code, name)) = mappings
                    .iter()
                    .find(|(_, (candidate, _, _))| *candidate == symbol)?;
                let body = line.split('"').nth(1)?;
                let fields = body.split(',').collect::<Vec<_>>();
                Some(IndexMarketQuote {
                    code: (*code).into(),
                    name: (*name).into(),
                    value: fields.get(1)?.parse().ok()?,
                    change: fields.get(2)?.parse().ok()?,
                    change_percent: fields.get(3)?.parse().ok()?,
                    source_time: now,
                })
            })
            .collect())
    }

    async fn get_json(&self, url: &str) -> Result<Value, ProviderError> {
        self.client
            .get(url)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|error| ProviderError::Request(format!("{error:?}")))?
            .json::<Value>()
            .await
            .map_err(|error| ProviderError::InvalidData(error.to_string()))
    }
}

#[async_trait]
impl MarketDataProvider for HybridMarketDataProvider {
    async fn fetch_indices(
        &self,
        codes: &[String],
    ) -> Result<Vec<IndexMarketQuote>, ProviderError> {
        if codes.is_empty() {
            return Ok(Vec::new());
        }
        let eastmoney = self.eastmoney_indices(codes).await.unwrap_or_default();
        let mut merged = eastmoney
            .into_iter()
            .map(|quote| (quote.code.clone(), quote))
            .collect::<HashMap<_, _>>();
        let missing = codes
            .iter()
            .filter(|secid| {
                index_mapping(secid)
                    .map(|(_, code, _)| !merged.contains_key(code))
                    .unwrap_or(false)
            })
            .cloned()
            .collect::<Vec<_>>();
        for quote in self.sina_indices(&missing).await.unwrap_or_default() {
            merged.insert(quote.code.clone(), quote);
        }

        let ordered = codes
            .iter()
            .filter_map(|secid| {
                let (_, code, _) = index_mapping(secid)?;
                merged.remove(code)
            })
            .collect::<Vec<_>>();
        if ordered.is_empty() {
            Err(ProviderError::Request(
                "all configured market index providers failed".into(),
            ))
        } else {
            Ok(ordered)
        }
    }

    async fn fetch_funds(&self, codes: &[String]) -> Result<Vec<FundQuote>, ProviderError> {
        if codes.is_empty() {
            return Ok(Vec::new());
        }
        let official = self.official_funds(codes).await?;
        let official_by_code = official
            .into_iter()
            .map(|fund| (fund.code.clone(), fund))
            .collect::<HashMap<_, _>>();

        let estimates = join_all(codes.iter().map(|code| self.sina_estimate(code))).await;
        let sina_by_code = estimates
            .into_iter()
            .flatten()
            .map(|estimate| (estimate.code.clone(), estimate))
            .collect::<HashMap<_, _>>();
        let missing_codes = codes
            .iter()
            .filter(|code| {
                match (
                    sina_by_code.get(code.as_str()),
                    official_by_code.get(code.as_str()),
                ) {
                    (Some(estimate), Some(official)) => !estimate_is_current(estimate, official),
                    _ => true,
                }
            })
            .cloned()
            .collect::<Vec<_>>();
        let holdings = join_all(
            missing_codes
                .iter()
                .map(|code| async move { (code.clone(), self.holdings_estimate(code).await) }),
        )
        .await
        .into_iter()
        .filter_map(|(code, estimate)| estimate.map(|value| (code, value)))
        .collect::<HashMap<_, _>>();

        let now = Utc::now();
        Ok(codes
            .iter()
            .filter_map(|code| {
                let fund = official_by_code.get(code)?;
                if let Some(estimate) = sina_by_code
                    .get(code)
                    .filter(|estimate| estimate_is_current(estimate, fund))
                {
                    let divisor = Decimal::ONE + estimate.change_percent / Decimal::new(100, 0);
                    let previous_nav = if divisor.is_zero() {
                        fund.nav
                    } else {
                        estimate.estimated_nav / divisor
                    };
                    return Some(FundQuote {
                        code: code.clone(),
                        name: fund.name.clone(),
                        current_nav: estimate.estimated_nav,
                        previous_nav,
                        change_percent: estimate.change_percent,
                        nature: DataNature::Estimated,
                        freshness: if estimate.estimate_date == Some(Local::now().date_naive()) {
                            Freshness::Fresh
                        } else {
                            Freshness::Delayed
                        },
                        provider: "新浪盘中估值".into(),
                        source_time: estimate.source_time,
                    });
                }

                if let Some(change_percent) = holdings.get(code) {
                    return Some(FundQuote {
                        code: code.clone(),
                        name: fund.name.clone(),
                        current_nav: fund.nav
                            * (Decimal::ONE + *change_percent / Decimal::new(100, 0)),
                        previous_nav: fund.nav,
                        change_percent: *change_percent,
                        nature: DataNature::Estimated,
                        freshness: Freshness::Delayed,
                        provider: "披露持仓加权估算".into(),
                        source_time: now,
                    });
                }

                let divisor = Decimal::ONE + fund.nav_change_percent / Decimal::new(100, 0);
                Some(FundQuote {
                    code: code.clone(),
                    name: fund.name.clone(),
                    current_nav: fund.nav,
                    previous_nav: if divisor.is_zero() {
                        fund.nav
                    } else {
                        fund.nav / divisor
                    },
                    change_percent: fund.nav_change_percent,
                    nature: DataNature::Confirmed,
                    freshness: Freshness::Stale,
                    provider: "东方财富确认净值".into(),
                    source_time: fund.source_time,
                })
            })
            .collect())
    }

    async fn lookup_fund(&self, code: &str) -> Result<Option<FundMetadata>, ProviderError> {
        self.fund_metadata(code).await
    }
}

fn estimate_is_current(estimate: &SinaEstimate, official: &OfficialFund) -> bool {
    match (estimate.estimate_date, official.nav_date) {
        (Some(estimate_date), Some(nav_date)) => estimate_date >= nav_date,
        _ => true,
    }
}

fn index_mapping(secid: &str) -> Option<(&'static str, &'static str, &'static str)> {
    match secid {
        "1.000001" => Some(("s_sh000001", "000001", "上证指数")),
        "0.399001" => Some(("s_sz399001", "399001", "深证成指")),
        "1.000300" => Some(("s_sh000300", "000300", "沪深 300")),
        "0.399006" => Some(("s_sz399006", "399006", "创业板指")),
        "100.DJIA" => Some(("int_dji", "DJIA", "道琼斯")),
        "100.NDX" => Some(("int_nasdaq", "NDX", "纳斯达克")),
        "100.SPX" => Some(("int_sp500", "SPX", "标普 500")),
        "100.HSI" => Some(("b_HSI", "HSI", "恒生指数")),
        "100.N225" => Some(("b_NKY", "N225", "日经 225")),
        "100.KOSPI" => Some(("b_KOSPI", "KOSPI", "韩国 KOSPI")),
        "100.TWII" => Some(("b_TWSE", "TWII", "台湾加权")),
        "100.AS51" => Some(("b_AS51", "AS51", "澳洲标普 200")),
        "100.SENSEX" => Some(("b_SENSEX", "SENSEX", "印度孟买 30")),
        "100.FTSE" => Some(("b_FTSE", "FTSE", "英国富时 100")),
        "100.GDAXI" => Some(("b_DAX", "GDAXI", "德国 DAX")),
        "100.FCHI" => Some(("b_CAC", "FCHI", "法国 CAC 40")),
        _ => None,
    }
}

fn parse_date(value: &str) -> Option<NaiveDate> {
    ["%Y-%m-%d", "%Y%m%d"]
        .iter()
        .find_map(|format| NaiveDate::parse_from_str(value.get(..10).unwrap_or(value), format).ok())
}

fn china_time(date: NaiveDate, time: &str) -> Option<DateTime<Utc>> {
    let raw = format!(
        "{} {}",
        date.format("%Y-%m-%d"),
        time.get(..8).unwrap_or(time)
    );
    let naive = ["%Y-%m-%d %H:%M:%S", "%Y-%m-%d %H:%M"]
        .iter()
        .find_map(|format| NaiveDateTime::parse_from_str(&raw, format).ok())?;
    FixedOffset::east_opt(8 * 60 * 60)?
        .from_local_datetime(&naive)
        .single()
        .map(|value| value.to_utc())
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(value_to_string)
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

fn decimal_field(value: &Value, key: &str) -> Option<Decimal> {
    value
        .get(key)
        .and_then(value_to_string)
        .and_then(|value| Decimal::from_str(&value).ok())
}

fn meaningful_string_field(value: &Value, key: &str) -> Option<String> {
    string_field(value, key).filter(|value| {
        let value = value.trim();
        !value.is_empty() && value != "--"
    })
}

fn parse_fund_metadata(value: &Value) -> Option<FundMetadata> {
    Some(FundMetadata {
        code: meaningful_string_field(value, "FCODE")?,
        name: meaningful_string_field(value, "SHORTNAME")?,
        fund_type: meaningful_string_field(value, "FTYPE"),
        company: meaningful_string_field(value, "JJGS"),
        industry: meaningful_string_field(value, "TTYPENAME")
            .or_else(|| meaningful_string_field(value, "FBKINDEXNAME")),
        index_name: meaningful_string_field(value, "INDEXNAME"),
        latest_nav: meaningful_string_field(value, "DWJZ"),
        nav_date: meaningful_string_field(value, "FSRQ"),
        provider: "东方财富基金资料".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_api_numbers_from_strings_and_json_numbers() {
        let value = serde_json::json!({"a": "1.2345", "b": -0.42});
        assert_eq!(decimal_field(&value, "a"), Some(Decimal::new(12_345, 4)));
        assert_eq!(decimal_field(&value, "b"), Some(Decimal::new(-42, 2)));
    }

    #[test]
    fn rejects_estimate_older_than_confirmed_nav() {
        let estimate = SinaEstimate {
            code: "001618".into(),
            estimated_nav: Decimal::ONE,
            change_percent: Decimal::ZERO,
            estimate_date: parse_date("2026-07-20"),
            source_time: Utc::now(),
        };
        let official = OfficialFund {
            code: "001618".into(),
            name: "test".into(),
            nav: Decimal::ONE,
            nav_change_percent: Decimal::ZERO,
            nav_date: parse_date("2026-07-21"),
            source_time: Utc::now(),
        };
        assert!(!estimate_is_current(&estimate, &official));
    }

    #[test]
    fn parses_fund_metadata_and_prefers_standard_industry() {
        let value = serde_json::json!({
            "FCODE": "161725",
            "SHORTNAME": "招商中证白酒指数(LOF)A",
            "FTYPE": "指数型-股票",
            "JJGS": "招商基金",
            "TTYPENAME": "食品饮料",
            "FBKINDEXNAME": "白酒",
            "INDEXNAME": "中证白酒指数",
            "DWJZ": "0.5581",
            "FSRQ": "2026-07-22"
        });
        let metadata = parse_fund_metadata(&value).unwrap();
        assert_eq!(metadata.code, "161725");
        assert_eq!(metadata.name, "招商中证白酒指数(LOF)A");
        assert_eq!(metadata.industry.as_deref(), Some("食品饮料"));
        assert_eq!(metadata.fund_type.as_deref(), Some("指数型-股票"));
        assert_eq!(metadata.company.as_deref(), Some("招商基金"));
    }

    #[tokio::test]
    #[ignore = "requires public market data APIs"]
    async fn live_provider_returns_global_indices_and_a_fund_snapshot() {
        let provider = HybridMarketDataProvider::new().unwrap();
        let funds = provider.fetch_funds(&["005827".into()]).await.unwrap();
        let metadata = provider.lookup_fund("161725").await.unwrap().unwrap();
        let indices = provider
            .fetch_indices(&[
                "1.000001".into(),
                "100.DJIA".into(),
                "100.HSI".into(),
                "100.N225".into(),
                "100.FTSE".into(),
            ])
            .await
            .unwrap();

        let codes = indices
            .iter()
            .map(|quote| quote.code.as_str())
            .collect::<Vec<_>>();
        assert_eq!(codes, ["000001", "DJIA", "HSI", "N225", "FTSE"]);
        assert_eq!(
            funds.first().map(|quote| quote.code.as_str()),
            Some("005827")
        );
        assert_eq!(metadata.name, "招商中证白酒指数(LOF)A");
        assert_eq!(metadata.industry.as_deref(), Some("食品饮料"));
    }
}
