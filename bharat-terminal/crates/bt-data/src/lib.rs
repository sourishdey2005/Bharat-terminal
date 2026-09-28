// crates/bt-data/src/lib.rs
// Author: Sourish Dey

//! bt-data: Real market data providers for Bharat Terminal.
//!
//! Provides access to free public market data APIs:
//! - Yahoo Finance (stocks, ETFs, indices, crypto, FX)
//! - Coinbase (crypto spot markets)
//! - SQLite caching with TTL-based expiration
//! - Symbol resolution for 50+ major companies

pub mod cache;
pub mod coinbase;
pub mod fmp;
pub mod india;
pub mod nse;
pub mod provider;
pub mod symbol;
pub mod yahoo;

pub use provider::{CompanyProfile, DataProvider, Interval, Quote, SymbolInfo};
pub use symbol::{COMPANY_LIST, DEFAULT_COMPANY};

use crate::cache::Cache;
use crate::coinbase::CoinbaseProvider;
use crate::yahoo::YahooProvider;
use bt_core::{OhlcvSeries, Result};
use chrono::{DateTime, Utc};
use std::path::Path;

/// High-level data service combining providers with caching.
pub struct DataService {
    yahoo: YahooProvider,
    coinbase: CoinbaseProvider,
    cache: Cache,
}

impl DataService {
    /// Create a new data service with default cache path.
    pub fn new() -> Result<Self> {
        std::fs::create_dir_all("./data")?;
        Ok(Self {
            yahoo: YahooProvider::new(),
            coinbase: CoinbaseProvider::new(),
            cache: Cache::new("./data/cache.db")?,
        })
    }

    /// Create with custom cache path.
    pub fn with_cache_path<P: AsRef<std::path::Path>>(cache_path: P) -> Result<Self> {
        Ok(Self {
            yahoo: YahooProvider::new(),
            coinbase: CoinbaseProvider::new(),
            cache: Cache::new(cache_path)?,
        })
    }

    /// Fetch OHLCV data with caching.
    /// Tries cache first, falls back to provider, updates cache on success.
    pub async fn fetch_ohlcv(
        &self,
        symbol: &str,
        interval: Interval,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<OhlcvSeries> {
        let interval_str = interval.as_str();

        // Try cache first
        if let Ok(Some(candles)) = self.cache.get_ohlcv(symbol, interval_str) {
            let series = OhlcvSeries::new(symbol, candles);
            if series.validate().is_ok() {
                return Ok(series);
            }
        }

        // Determine provider based on symbol
        let series = if symbol.ends_with("-USD") || symbol.contains("BTC") || symbol.contains("ETH")
        {
            // Use Coinbase for crypto
            let granularity = self.interval_to_granularity(interval);
            let start_ts = Some(start);
            let end_ts = Some(end);
            self.coinbase
                .fetch_candles(symbol, granularity, start_ts, end_ts)
                .await?
        } else {
            // Use Yahoo for everything else
            self.yahoo.fetch_ohlcv(symbol, interval, start, end).await?
        };

        // Update cache
        let _ = self.cache.put_ohlcv(symbol, interval_str, &series.candles);

        Ok(series)
    }

    /// Fetch real-time quote.
    pub async fn fetch_quote(&self, symbol: &str) -> Result<Quote> {
        if symbol.ends_with("-USD") {
            self.coinbase.fetch_ticker(symbol).await
        } else {
            self.yahoo.fetch_quote(symbol).await
        }
    }

    /// Search symbols.
    pub async fn search_symbols(&self, query: &str) -> Result<Vec<SymbolInfo>> {
        self.yahoo.search_symbols(query).await
    }

    /// Fetch company profile.
    pub async fn fetch_company_profile(&self, symbol: &str) -> Result<CompanyProfile> {
        self.yahoo.fetch_company_profile(symbol).await
    }

    /// Get cache statistics.
    pub fn cache_stats(&self) -> Result<crate::cache::CacheStats> {
        self.cache.stats()
    }

    /// Cleanup expired cache entries.
    pub fn cleanup_cache(&self) -> Result<usize> {
        self.cache.cleanup()
    }

    fn interval_to_granularity(&self, interval: Interval) -> u32 {
        match interval {
            Interval::Min1 => 60,
            Interval::Min5 => 300,
            Interval::Min15 => 900,
            Interval::Min30 => 1800,
            Interval::Hour1 => 3600,
            Interval::Day1 => 86400,
            Interval::Week1 => 604800,
            Interval::Month1 => 2592000,
        }
    }
}

impl Default for DataService {
    fn default() -> Self {
        Self::new().expect("Failed to create DataService")
    }
}

// Re-exports for convenience
pub use crate::coinbase::CoinbaseProvider as Coinbase;
pub use crate::provider::DataProvider as Provider;
pub use crate::yahoo::YahooProvider as Yahoo;

// India market data re-exports
pub use crate::india::{
    BankingIndicator, CommodityData, CorporateAction, CreditRating, FPIFlow, GSecData, IPOData,
    MacroIndicator, MoneyMarketRate, MutualFundData, OptionsChainEntry, RBIPolicy, USDRate,
    YieldCurvePoint,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_constants() {
        assert!(!COMPANY_LIST.is_empty());
        assert!(COMPANY_LIST.len() >= 50);
    }

    #[test]
    fn test_data_service_creation() {
        let service = DataService::new();
        assert!(service.is_ok());
    }
}
