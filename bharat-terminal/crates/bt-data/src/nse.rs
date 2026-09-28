// crates/bt-data/src/nse.rs
// Author: Sourish Dey

//! NSE India provider for F&O chains, Bhavcopy, and indices.
//!
//! Uses a cookie jar: first hits https://www.nseindia.com to obtain
//! session cookies, then queries API endpoints.

use bt_core::{BtError, Result};
use reqwest::cookie::Jar;
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tracing::instrument;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
const BASE_URL: &str = "https://www.nseindia.com";

#[derive(Debug, Clone, Deserialize)]
pub struct NSEOptionChain {
    pub strike: f64,
    pub call_oi: u64,
    pub call_ltp: f64,
    pub call_iv: f64,
    pub put_oi: u64,
    pub put_ltp: f64,
    pub put_iv: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NSEIndex {
    pub name: String,
    pub last: f64,
    pub change: f64,
    pub change_pct: f64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NSEOptionChainRecord {
    #[serde(rename = "strikePrice")]
    strike_price: f64,
    #[serde(rename = "CE")]
    ce: Option<NSEOptionData>,
    #[serde(rename = "PE")]
    pe: Option<NSEOptionData>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NSEOptionData {
    #[serde(rename = "openInterest")]
    open_interest: u64,
    #[serde(rename = "lastPrice")]
    last_price: f64,
    #[serde(rename = "impliedVolatility")]
    implied_volatility: f64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NSEOptionChainResponse {
    records: NSEOptionChainRecords,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NSEOptionChainRecords {
    data: Vec<NSEOptionChainRecord>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NSEIndexItem {
    #[serde(rename = "symbol")]
    symbol: String,
    #[serde(rename = "lastPrice")]
    last_price: f64,
    #[serde(rename = "change")]
    change: f64,
    #[serde(rename = "pChange")]
    p_change: f64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NSEIndexResponse {
    data: Vec<NSEIndexItem>,
}

pub struct NSEProvider {
    client: Client,
}

impl NSEProvider {
    pub fn new() -> Self {
        let jar = Jar::default();
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .cookie_provider(Arc::new(jar))
            .timeout(StdDuration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");
        Self { client }
    }

    /// Establish a session by hitting the NSE homepage to get cookies.
    #[instrument(skip(self))]
    pub async fn init_session(&self) -> Result<()> {
        let resp = self
            .client
            .get(BASE_URL)
            .send()
            .await
            .map_err(|e| BtError::DataFetch(format!("NSE session init failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(BtError::DataFetch(format!(
                "NSE session init HTTP {}",
                resp.status()
            )));
        }
        Ok(())
    }

    /// Fetch option chain for an index (e.g., "NIFTY", "BANKNIFTY").
    #[instrument(skip(self))]
    pub async fn fetch_option_chain(&self, symbol: &str) -> Result<Vec<NSEOptionChain>> {
        let url = format!(
            "{}/api/option-chain-indices?symbol={}",
            BASE_URL,
            urlencoding::encode(symbol)
        );
        let resp = self.fetch_with_retry(&url).await?;
        let data: NSEOptionChainResponse = resp.json().await.map_err(|e| {
            BtError::InvalidInput(format!("JSON parse error: {}", e))
        })?;

        let chains: Vec<NSEOptionChain> = data
            .records
            .data
            .into_iter()
            .map(|r| NSEOptionChain {
                strike: r.strike_price,
                call_oi: r.ce.as_ref().map(|c| c.open_interest).unwrap_or(0),
                call_ltp: r.ce.as_ref().map(|c| c.last_price).unwrap_or(0.0),
                call_iv: r.ce.as_ref().map(|c| c.implied_volatility).unwrap_or(0.0),
                put_oi: r.pe.as_ref().map(|p| p.open_interest).unwrap_or(0),
                put_ltp: r.pe.as_ref().map(|p| p.last_price).unwrap_or(0.0),
                put_iv: r.pe.as_ref().map(|p| p.implied_volatility).unwrap_or(0.0),
            })
            .collect();

        Ok(chains)
    }

    /// Fetch index constituents (e.g., "NIFTY 50").
    #[instrument(skip(self))]
    pub async fn fetch_index(&self, index: &str) -> Result<Vec<NSEIndex>> {
        let url = format!(
            "{}/api/equity-stockIndices?index={}",
            BASE_URL,
            urlencoding::encode(index)
        );
        let resp = self.fetch_with_retry(&url).await?;
        let data: NSEIndexResponse = resp.json().await.map_err(|e| {
            BtError::InvalidInput(format!("JSON parse error: {}", e))
        })?;

        let indices: Vec<NSEIndex> = data
            .data
            .into_iter()
            .map(|item| NSEIndex {
                name: item.symbol,
                last: item.last_price,
                change: item.change,
                change_pct: item.p_change,
            })
            .collect();

        Ok(indices)
    }

    async fn fetch_with_retry(&self, url: &str) -> Result<reqwest::Response> {
        let mut attempts = 0;
        const MAX_RETRIES: u32 = 3;

        loop {
            let resp = self.client.get(url).send().await.map_err(|e| {
                BtError::DataFetch(format!("Network error: {}", e))
            })?;

            if resp.status() == 429 || resp.status() == 401 {
                attempts += 1;
                if attempts >= MAX_RETRIES {
                    return Err(BtError::DataFetch(format!(
                        "Rate limited ({}) after {} retries",
                        resp.status(),
                        MAX_RETRIES
                    )));
                }
                let delay = StdDuration::from_millis(500 * 2_u64.pow(attempts - 1));
                tokio::time::sleep(delay).await;
                continue;
            }

            if !resp.status().is_success() {
                return Err(BtError::DataFetch(format!(
                    "HTTP {}: {}",
                    resp.status(),
                    resp.text().await.unwrap_or_default()
                )));
            }

            return Ok(resp);
        }
    }
}

impl Default for NSEProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Live NSE session test. Requires network access to nseindia.com.
    /// Run with: cargo test -p bt-data -- --ignored
    #[tokio::test]
    #[ignore = "requires live network access to nseindia.com"]
    async fn test_init_session() {
        let provider = NSEProvider::new();
        let result = provider.init_session().await;
        assert!(result.is_ok());
    }

    /// Live NSE option-chain test. Run with: cargo test -p bt-data -- --ignored
    #[tokio::test]
    #[ignore = "requires live network access to nseindia.com"]
    async fn test_fetch_option_chain() {
        let provider = NSEProvider::new();
        provider.init_session().await.unwrap();
        let chain = provider.fetch_option_chain("NIFTY").await;
        assert!(chain.is_ok());
        assert!(!chain.unwrap().is_empty());
    }

    /// Live NSE index quote test. Run with: cargo test -p bt-data -- --ignored
    #[tokio::test]
    #[ignore = "requires live network access to nseindia.com"]
    async fn test_fetch_index() {
        let provider = NSEProvider::new();
        provider.init_session().await.unwrap();
        let index = provider.fetch_index("NIFTY 50").await;
        assert!(index.is_ok());
        assert!(!index.unwrap().is_empty());
    }
}
