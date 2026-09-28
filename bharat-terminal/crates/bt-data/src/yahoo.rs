// crates/bt-data/src/yahoo.rs
// Author: Sourish Dey

//! Yahoo Finance direct HTTP provider (no external crate required).
//! Fetches real OHLCV data, quotes, search results, and company profiles
//! from Yahoo Finance's public endpoints.

use crate::provider::{CompanyProfile, Interval, Quote, SymbolInfo, DataProvider};
use bt_core::{BtError, Candle, OhlcvSeries, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration as StdDuration;
use tracing::instrument;

const USER_AGENT: &str = "Mozilla/5.0 (compatible; BharatTerminal/2.0)";
const BASE_CHART: &str = "https://query1.finance.yahoo.com/v8/finance/chart";
const BASE_QUOTE: &str = "https://query1.finance.yahoo.com/v7/finance/quote";
const BASE_SEARCH: &str = "https://query2.finance.yahoo.com/v1/finance/search";
const BASE_PROFILE: &str = "https://query1.finance.yahoo.com/v10/finance/quoteSummary";

#[derive(Debug, Deserialize)]
struct YahooChartResponse {
    chart: ChartData,
}

#[derive(Debug, Deserialize)]
struct ChartData {
    result: Option<Vec<ChartResult>>,
    error: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ChartResult {
    timestamp: Option<Vec<i64>>,
    indicators: Indicators,
    meta: ChartMeta,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ChartMeta {
    symbol: String,
    #[serde(rename = "regularMarketPrice")]
    regular_market_price: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct Indicators {
    quote: Vec<QuoteIndicators>,
}

#[derive(Debug, Deserialize)]
struct QuoteIndicators {
    open: Vec<Option<f64>>,
    high: Vec<Option<f64>>,
    low: Vec<Option<f64>>,
    close: Vec<Option<f64>>,
    volume: Vec<Option<f64>>,
}

#[derive(Debug, Deserialize)]
struct YahooQuoteResponse {
    #[serde(rename = "quoteResponse")]
    quote_response: QuoteResponse,
}

#[derive(Debug, Deserialize)]
struct QuoteResponse {
    result: Vec<QuoteResult>,
}

#[derive(Debug, Deserialize)]
struct QuoteResult {
    symbol: String,
    #[serde(rename = "regularMarketPrice")]
    regular_market_price: Option<f64>,
    #[serde(rename = "regularMarketChange")]
    regular_market_change: Option<f64>,
    #[serde(rename = "regularMarketChangePercent")]
    regular_market_change_percent: Option<f64>,
    #[serde(rename = "regularMarketVolume")]
    regular_market_volume: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct YahooSearchResponse {
    quotes: Vec<SearchQuote>,
}

#[derive(Debug, Deserialize)]
struct SearchQuote {
    symbol: String,
    #[serde(rename = "shortname")]
    short_name: Option<String>,
    #[serde(rename = "longname")]
    long_name: Option<String>,
    exchange: Option<String>,
    #[serde(rename = "quoteType")]
    quote_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct YahooProfileResponse {
    #[serde(rename = "quoteSummary")]
    quote_summary: QuoteSummary,
}

#[derive(Debug, Deserialize)]
struct QuoteSummary {
    result: Vec<ProfileResult>,
}

#[derive(Debug, Deserialize)]
struct ProfileResult {
    #[serde(rename = "longName")]
    long_name: Option<String>,
    sector: Option<String>,
    industry: Option<String>,
    #[serde(rename = "marketCap")]
    market_cap: Option<MarketCap>,
    #[serde(rename = "fullTimeEmployees")]
    full_time_employees: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct MarketCap {
    raw: u64,
}

/// Yahoo Finance provider using direct HTTP calls.
pub struct YahooProvider {
    client: Client,
}

impl YahooProvider {
    /// Create a new Yahoo Finance provider.
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(StdDuration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");
        Self { client }
    }

    fn build_chart_url(&self, symbol: &str, interval: Interval, range: &str) -> String {
        format!(
            "{}/{}?interval={}&range={}",
            BASE_CHART,
            symbol,
            interval.as_str(),
            range
        )
    }

    async fn fetch_with_retry(&self, url: &str) -> Result<reqwest::Response> {
        let mut attempts = 0;
        const MAX_RETRIES: u32 = 3;

        loop {
            let resp = self
                .client
                .get(url)
                .send()
                .await
                .map_err(|e| BtError::InvalidInput(format!("Network error: {}", e)))?;

            if resp.status() == 429 {
                attempts += 1;
                if attempts >= MAX_RETRIES {
                    return Err(BtError::InvalidInput(
                        "Rate limited (429) after 3 retries".to_string(),
                    ));
                }
                let delay = StdDuration::from_millis(500 * 2_u64.pow(attempts - 1));
                tokio::time::sleep(delay).await;
                continue;
            }

            if !resp.status().is_success() {
                return Err(BtError::InvalidInput(format!(
                    "HTTP {}: {}",
                    resp.status(),
                    resp.text().await.unwrap_or_default()
                )));
            }

            return Ok(resp);
        }
    }

    /// Fetch OHLCV history for a symbol.
    pub async fn fetch_ohlcv_range(
        &self,
        symbol: &str,
        interval: Interval,
        range: &str,
    ) -> Result<OhlcvSeries> {
        let url = self.build_chart_url(symbol, interval, range);
        let resp = self.fetch_with_retry(&url).await?;
        let data: YahooChartResponse = resp
            .json()
            .await
            .map_err(|e| BtError::InvalidInput(format!("JSON parse error: {}", e)))?;

        let result = data
            .chart
            .result
            .and_then(|r| r.into_iter().next())
            .ok_or_else(|| BtError::EmptySeries(symbol.to_string()))?;

        if let Some(error) = data.chart.error {
            return Err(BtError::InvalidInput(format!("Yahoo error: {}", error)));
        }

        let timestamps = result.timestamp.unwrap_or_default();
        let quote = result
            .indicators
            .quote
            .into_iter()
            .next()
            .ok_or_else(|| BtError::EmptySeries(symbol.to_string()))?;

        let mut candles = Vec::with_capacity(timestamps.len());

        for (i, &ts) in timestamps.iter().enumerate() {
            let open = quote.open.get(i).and_then(|v| *v);
            let high = quote.high.get(i).and_then(|v| *v);
            let low = quote.low.get(i).and_then(|v| *v);
            let close = quote.close.get(i).and_then(|v| *v);
            let volume = quote.volume.get(i).and_then(|v| *v).unwrap_or(0.0);

            if let (Some(o), Some(h), Some(l), Some(c)) = (open, high, low, close) {
                candles.push(Candle::new(ts as f64, o, h, l, c, volume));
            }
        }

        if candles.is_empty() {
            return Err(BtError::EmptySeries(symbol.to_string()));
        }

        Ok(OhlcvSeries::new(symbol, candles))
    }
}

#[async_trait::async_trait]
impl DataProvider for YahooProvider {
    #[instrument(skip(self))]
    async fn fetch_ohlcv(
        &self,
        symbol: &str,
        interval: Interval,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<OhlcvSeries> {
        let days = (end - start).num_days();
        let range = if days <= 1 {
            "1d".to_string()
        } else if days <= 5 {
            "5d".to_string()
        } else if days <= 30 {
            "1mo".to_string()
        } else if days <= 90 {
            "3mo".to_string()
        } else if days <= 180 {
            "6mo".to_string()
        } else if days <= 365 {
            "1y".to_string()
        } else if days <= 730 {
            "2y".to_string()
        } else {
            "5y".to_string()
        };

        self.fetch_ohlcv_range(symbol, interval, &range).await
    }

    #[instrument(skip(self))]
    async fn fetch_quote(&self, symbol: &str) -> Result<Quote> {
        let url = format!("{}?symbols={}", BASE_QUOTE, symbol);
        let resp = self.fetch_with_retry(&url).await?;
        let data: YahooQuoteResponse = resp
            .json()
            .await
            .map_err(|e| BtError::InvalidInput(format!("JSON parse error: {}", e)))?;

        let result = data
            .quote_response
            .result
            .into_iter()
            .next()
            .ok_or_else(|| BtError::EmptySeries(symbol.to_string()))?;

        Ok(Quote {
            symbol: result.symbol,
            price: result.regular_market_price.unwrap_or(0.0),
            change: result.regular_market_change.unwrap_or(0.0),
            change_pct: result.regular_market_change_percent.unwrap_or(0.0),
            volume: result.regular_market_volume.unwrap_or(0),
            timestamp: Utc::now(),
        })
    }

    #[instrument(skip(self))]
    async fn search_symbols(&self, query: &str) -> Result<Vec<SymbolInfo>> {
        let url = format!("{}?q={}&quotesCount=20", BASE_SEARCH, query);
        let resp = self.fetch_with_retry(&url).await?;
        let data: YahooSearchResponse = resp
            .json()
            .await
            .map_err(|e| BtError::InvalidInput(format!("JSON parse error: {}", e)))?;

        let results = data
            .quotes
            .into_iter()
            .map(|q| SymbolInfo {
                symbol: q.symbol,
                name: q.long_name.or(q.short_name).unwrap_or_default(),
                exchange: q.exchange.unwrap_or_default(),
                asset_type: q.quote_type.unwrap_or_default(),
            })
            .collect();

        Ok(results)
    }

    #[instrument(skip(self))]
    async fn fetch_company_profile(&self, symbol: &str) -> Result<CompanyProfile> {
        let url = format!(
            "{}/{}?modules=assetProfile,summaryDetail",
            BASE_PROFILE, symbol
        );
        let resp = self.fetch_with_retry(&url).await?;
        let data: YahooProfileResponse = resp
            .json()
            .await
            .map_err(|e| BtError::InvalidInput(format!("JSON parse error: {}", e)))?;

        let result = data
            .quote_summary
            .result
            .into_iter()
            .next()
            .ok_or_else(|| BtError::EmptySeries(symbol.to_string()))?;

        Ok(CompanyProfile {
            symbol: symbol.to_string(),
            name: result.long_name.unwrap_or_default(),
            sector: result.sector.unwrap_or_default(),
            industry: result.industry.unwrap_or_default(),
            market_cap: result.market_cap.map(|m| m.raw as f64).unwrap_or(0.0),
            employees: result.full_time_employees.unwrap_or(0),
        })
    }
}

impl Default for YahooProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resolve_indian_symbol() {
        let provider = YahooProvider::new();
        let series = provider
            .fetch_ohlcv_range("RELIANCE.NS", Interval::Day1, "1mo")
            .await;
        assert!(series.is_ok());
        let s = series.unwrap();
        assert!(!s.candles.is_empty());
        assert_eq!(s.symbol, "RELIANCE.NS");
    }

    #[tokio::test]
    async fn test_resolve_us_symbol() {
        let provider = YahooProvider::new();
        let series = provider
            .fetch_ohlcv_range("AAPL", Interval::Day1, "1mo")
            .await;
        assert!(series.is_ok());
        let s = series.unwrap();
        assert!(!s.candles.is_empty());
        assert_eq!(s.symbol, "AAPL");
    }

    #[tokio::test]
    async fn test_search() {
        let provider = YahooProvider::new();
        let results = provider.search_symbols("RELIANCE").await;
        assert!(results.is_ok());
        let r = results.unwrap();
        assert!(!r.is_empty());
    }
}