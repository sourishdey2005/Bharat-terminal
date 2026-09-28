// crates/bt-data/src/coinbase.rs
// Author: Sourish Dey

use crate::provider::Quote;
use bt_core::{BtError, Candle, OhlcvSeries, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration as StdDuration;
use tracing::instrument;

const USER_AGENT: &str = "Mozilla/5.0 (compatible; BharatTerminal/2.0)";
const BASE_URL: &str = "https://api.exchange.coinbase.com";

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CoinbaseTicker {
    price: String,
    #[serde(rename = "open_24h")]
    open_24h: String,
    #[serde(rename = "volume_24h")]
    volume_24h: String,
    #[serde(rename = "low_24h")]
    low_24h: String,
    #[serde(rename = "high_24h")]
    high_24h: String,
    #[serde(rename = "volume_30d")]
    volume_30d: String,
    #[serde(rename = "bestBid")]
    best_bid: String,
    #[serde(rename = "bestAsk")]
    best_ask: String,
}

pub struct CoinbaseProvider {
    client: Client,
}

impl CoinbaseProvider {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(StdDuration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");
        Self { client }
    }

    /// Granularity in seconds: 60 (1m), 300 (5m), 900 (15m), 3600 (1h), 21600 (6h), 86400 (1d)
    #[instrument(skip(self))]
    pub async fn fetch_candles(
        &self,
        pair: &str,
        granularity: u32,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> Result<OhlcvSeries> {
        let mut url = format!(
            "{}/products/{}/candles?granularity={}",
            BASE_URL, pair, granularity
        );

        if let Some(s) = start {
            url.push_str(&format!("&start={}", s.timestamp()));
        }
        if let Some(e) = end {
            url.push_str(&format!("&end={}", e.timestamp()));
        }

        let resp = self.fetch_with_retry(&url).await?;
        let data: Vec<Vec<f64>> = resp.json().await.map_err(|e| {
            BtError::InvalidInput(format!("JSON parse error: {}", e))
        })?;

        let mut candles: Vec<Candle> = data
            .into_iter()
            .map(|r| Candle {
                t: r[0],
                low: r[1],
                high: r[2],
                open: r[3],
                close: r[4],
                volume: r[5],
            })
            .collect();

        candles.reverse(); // Coinbase returns newest first

        if candles.is_empty() {
            return Err(BtError::EmptySeries(pair.to_string()));
        }

        Ok(OhlcvSeries::new(pair, candles))
    }

    #[instrument(skip(self))]
    pub async fn fetch_ticker(&self, pair: &str) -> Result<Quote> {
        let url = format!("{}/products/{}/ticker", BASE_URL, pair);
        let resp = self.fetch_with_retry(&url).await?;
        let data: CoinbaseTicker = resp.json().await.map_err(|e| {
            BtError::InvalidInput(format!("JSON parse error: {}", e))
        })?;

        let price = data.price.parse().unwrap_or(0.0);
        let open = data.open_24h.parse().unwrap_or(0.0);
        let volume = data.volume_24h.parse().unwrap_or(0.0);

        Ok(Quote {
            symbol: pair.to_string(),
            price,
            change: price - open,
            change_pct: if open > 0.0 { ((price - open) / open) * 100.0 } else { 0.0 },
            volume: volume as u64,
            timestamp: Utc::now(),
        })
    }

    async fn fetch_with_retry(&self, url: &str) -> Result<reqwest::Response> {
        let mut attempts = 0;
        const MAX_RETRIES: u32 = 3;

        loop {
            let resp = self.client.get(url).send().await.map_err(|e| {
                BtError::InvalidInput(format!("Network error: {}", e))
            })?;

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
}

impl Default for CoinbaseProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[tokio::test]
    async fn test_fetch_btc_usd() {
        let provider = CoinbaseProvider::new();
        let end = Utc::now();
        let start = end - Duration::days(30);
        let series = provider
            .fetch_candles("BTC-USD", 86400, Some(start), Some(end))
            .await;
        assert!(series.is_ok());
        let s = series.unwrap();
        assert!(!s.candles.is_empty());
        assert_eq!(s.symbol, "BTC-USD");
    }

    #[tokio::test]
    async fn test_fetch_eth_usd() {
        let provider = CoinbaseProvider::new();
        let series = provider.fetch_candles("ETH-USD", 86400, None, None).await;
        assert!(series.is_ok());
    }
}