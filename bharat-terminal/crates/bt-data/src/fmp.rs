// crates/bt-data/src/fmp.rs
// Author: Sourish Dey

//! Financial Modeling Prep (FMP) provider.
//!
//! Free tier: 250 requests/day.
//! Base URL: https://financialmodelingprep.com/api/v3/
//!
//! Provides company fundamentals: income statement, balance sheet,
//! cash flow, quotes, and profiles.

use bt_core::{BtError, Result};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration as StdDuration;
use tracing::instrument;

const USER_AGENT: &str = "Mozilla/5.0 (compatible; BharatTerminal/2.0)";
const BASE_URL: &str = "https://financialmodelingprep.com/api/v3";

#[derive(Debug, Clone, Deserialize)]
pub struct FMPQuote {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change: f64,
    pub change_pct: f64,
    pub market_cap: u64,
    pub volume: u64,
    pub pe_ratio: f64,
    pub eps: f64,
    pub dividend_yield: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FMPProfile {
    pub symbol: String,
    pub name: String,
    pub sector: String,
    pub industry: String,
    pub market_cap: u64,
    pub description: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FMPIncomeStatement {
    pub symbol: String,
    pub date: String,
    pub revenue: f64,
    pub net_income: f64,
    pub gross_profit: f64,
    pub operating_income: f64,
    pub eps: f64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FMPBalanceSheet {
    pub symbol: String,
    pub date: String,
    pub total_assets: f64,
    pub total_liabilities: f64,
    pub total_equity: f64,
    pub cash_and_equivalents: f64,
    pub long_term_debt: f64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FMPCashFlow {
    pub symbol: String,
    pub date: String,
    pub operating_cash_flow: f64,
    pub investing_cash_flow: f64,
    pub financing_cash_flow: f64,
    pub free_cash_flow: f64,
    pub capital_expenditure: f64,
}

pub struct FMPProvider {
    client: Client,
    api_key: String,
}

impl FMPProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(StdDuration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");
        Self {
            client,
            api_key: api_key.into(),
        }
    }

    #[instrument(skip(self))]
    pub async fn fetch_quote(&self, symbol: &str) -> Result<FMPQuote> {
        let url = format!(
            "{}/quote/{}?apikey={}",
            BASE_URL, symbol, self.api_key
        );
        let resp = self.fetch_with_retry(&url).await?;
        let data: Vec<FMPQuote> = resp.json().await.map_err(|e| {
            BtError::InvalidInput(format!("JSON parse error: {}", e))
        })?;
        data.into_iter().next().ok_or_else(|| {
            BtError::EmptySeries(format!("No quote for {}", symbol))
        })
    }

    #[instrument(skip(self))]
    pub async fn fetch_profile(&self, symbol: &str) -> Result<FMPProfile> {
        let url = format!(
            "{}/profile/{}?apikey={}",
            BASE_URL, symbol, self.api_key
        );
        let resp = self.fetch_with_retry(&url).await?;
        let data: Vec<FMPProfile> = resp.json().await.map_err(|e| {
            BtError::InvalidInput(format!("JSON parse error: {}", e))
        })?;
        data.into_iter().next().ok_or_else(|| {
            BtError::EmptySeries(format!("No profile for {}", symbol))
        })
    }

    #[instrument(skip(self))]
    pub async fn fetch_income_statement(
        &self,
        symbol: &str,
        period: &str,
        limit: u32,
    ) -> Result<Vec<FMPIncomeStatement>> {
        let url = format!(
            "{}/income-statement/{}?period={}&limit={}&apikey={}",
            BASE_URL, symbol, period, limit, self.api_key
        );
        let resp = self.fetch_with_retry(&url).await?;
        let data: Vec<FMPIncomeStatement> = resp.json().await.map_err(|e| {
            BtError::InvalidInput(format!("JSON parse error: {}", e))
        })?;
        Ok(data)
    }

    #[instrument(skip(self))]
    pub async fn fetch_balance_sheet(
        &self,
        symbol: &str,
        period: &str,
        limit: u32,
    ) -> Result<Vec<FMPBalanceSheet>> {
        let url = format!(
            "{}/balance-sheet-statement/{}?period={}&limit={}&apikey={}",
            BASE_URL, symbol, period, limit, self.api_key
        );
        let resp = self.fetch_with_retry(&url).await?;
        let data: Vec<FMPBalanceSheet> = resp.json().await.map_err(|e| {
            BtError::InvalidInput(format!("JSON parse error: {}", e))
        })?;
        Ok(data)
    }

    #[instrument(skip(self))]
    pub async fn fetch_cash_flow(
        &self,
        symbol: &str,
        period: &str,
        limit: u32,
    ) -> Result<Vec<FMPCashFlow>> {
        let url = format!(
            "{}/cash-flow-statement/{}?period={}&limit={}&apikey={}",
            BASE_URL, symbol, period, limit, self.api_key
        );
        let resp = self.fetch_with_retry(&url).await?;
        let data: Vec<FMPCashFlow> = resp.json().await.map_err(|e| {
            BtError::InvalidInput(format!("JSON parse error: {}", e))
        })?;
        Ok(data)
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

impl Default for FMPProvider {
    fn default() -> Self {
        Self::new("demo")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Live FMP quote test. Requires network access and a valid API key.
    /// Run with: cargo test -p bt-data -- --ignored
    #[tokio::test]
    #[ignore = "requires network access and a valid FMP API key"]
    async fn test_fetch_quote() {
        let provider = FMPProvider::new(std::env::var("FMP_API_KEY").unwrap_or_else(|_| "demo".into()));
        let quote = provider.fetch_quote("AAPL").await;
        assert!(quote.is_ok());
        let q = quote.unwrap();
        assert_eq!(q.symbol, "AAPL");
    }

    /// Live FMP profile test. Run with: cargo test -p bt-data -- --ignored
    #[tokio::test]
    #[ignore = "requires network access and a valid FMP API key"]
    async fn test_fetch_profile() {
        let provider = FMPProvider::new(std::env::var("FMP_API_KEY").unwrap_or_else(|_| "demo".into()));
        let profile = provider.fetch_profile("AAPL").await;
        assert!(profile.is_ok());
    }

    /// Live FMP income statement test. Run with: cargo test -p bt-data -- --ignored
    #[tokio::test]
    #[ignore = "requires network access and a valid FMP API key"]
    async fn test_fetch_income_statement() {
        let provider = FMPProvider::new(std::env::var("FMP_API_KEY").unwrap_or_else(|_| "demo".into()));
        let statements = provider.fetch_income_statement("AAPL", "annual", 5).await;
        assert!(statements.is_ok());
    }
}
