// crates/bt-data/src/provider.rs
// Author: Sourish Dey

use async_trait::async_trait;
use bt_core::{OhlcvSeries, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Market data interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Interval {
    /// 1 minute interval.
    Min1,
    /// 5 minute interval.
    Min5,
    /// 15 minute interval.
    Min15,
    /// 30 minute interval.
    Min30,
    /// 1 hour interval.
    Hour1,
    /// 1 day interval.
    Day1,
    /// 1 week interval.
    Week1,
    /// 1 month interval.
    Month1,
}

impl Interval {
    pub fn as_str(&self) -> &'static str {
        match self {
            Interval::Min1 => "1m",
            Interval::Min5 => "5m",
            Interval::Min15 => "15m",
            Interval::Min30 => "30m",
            Interval::Hour1 => "1h",
            Interval::Day1 => "1d",
            Interval::Week1 => "1wk",
            Interval::Month1 => "1mo",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "1m" | "1min" => Some(Interval::Min1),
            "5m" | "5min" => Some(Interval::Min5),
            "15m" | "15min" => Some(Interval::Min15),
            "30m" | "30min" => Some(Interval::Min30),
            "1h" | "1hour" => Some(Interval::Hour1),
            "1d" | "1day" | "daily" => Some(Interval::Day1),
            "1wk" | "1w" | "weekly" => Some(Interval::Week1),
            "1mo" | "1month" | "monthly" => Some(Interval::Month1),
            _ => None,
        }
    }
}

/// Real-time quote snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub symbol: String,
    pub price: f64,
    pub change: f64,
    pub change_pct: f64,
    pub volume: u64,
    pub timestamp: DateTime<Utc>,
}

/// Symbol search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInfo {
    pub symbol: String,
    pub name: String,
    pub exchange: String,
    pub asset_type: String,
}

/// Company fundamental profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyProfile {
    pub symbol: String,
    pub name: String,
    pub sector: String,
    pub industry: String,
    pub market_cap: f64,
    pub employees: u32,
}

/// Async trait for market data providers.
#[async_trait]
pub trait DataProvider: Send + Sync {
    /// Fetch OHLCV candles for a symbol in a date range.
    async fn fetch_ohlcv(
        &self,
        symbol: &str,
        interval: Interval,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<OhlcvSeries>;

    /// Fetch real-time quote.
    async fn fetch_quote(&self, symbol: &str) -> Result<Quote>;

    /// Search symbols by query.
    async fn search_symbols(&self, query: &str) -> Result<Vec<SymbolInfo>>;

    /// Fetch company profile (fundamentals).
    async fn fetch_company_profile(&self, symbol: &str) -> Result<CompanyProfile>;
}
