// crates/bt-data/src/bhavcopy.rs
// Author: Sourish Dey

//! NSE/BSE official daily settlement files ("bhavcopy").
//!
//! Exchanges publish the complete end-of-day tape for every listed scrip as a
//! static ZIP containing a CSV. Unlike the browser-facing JSON endpoints these
//! files need no session, no cookie and no API key, and they are the
//! authoritative settlement record rather than a derived feed.
//!
//! NSE layout:
//! `https://archives.nseindia.com/content/historical/EQUITIES/{YYYY}/{MMM}/cm{DD}{MMM}{YYYY}bhav.csv.zip`
//!
//! CSV columns (NSE equity bhavcopy):
//! `SYMBOL, SERIES, OPEN, HIGH, LOW, CLOSE, LAST, PREVCLOSE, TOTTRDQTY,
//!  TOTTRDVAL, TIMESTAMP, TOTALTRADES, ISIN`
//!
//! BSE layout (one file per day, all scrips):
//! `https://www.bseindia.com/BSEDATA/Gross/Public/EQ_ISIN_DDMMYY.zip`

use std::io::Read;

use bt_core::{BtError, Candle, OhlcvSeries, Result};
use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};
use serde::Deserialize;

const NSE_ARCHIVE: &str = "https://archives.nseindia.com/content/historical/EQUITIES";
const BSE_GROSS: &str = "https://www.bseindia.com/BSEDATA/Gross/Public";
const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36";

/// A single row of an NSE equity bhavcopy CSV.
#[derive(Debug, Deserialize)]
struct NseRow {
    #[serde(rename = "SYMBOL")]
    symbol: String,
    #[serde(rename = "OPEN", default)]
    open: String,
    #[serde(rename = "HIGH", default)]
    high: String,
    #[serde(rename = "LOW", default)]
    low: String,
    #[serde(rename = "CLOSE", default)]
    close: String,
    #[serde(rename = "TOTTRDQTY", default)]
    qty: String,
    #[serde(rename = "TIMESTAMP", default)]
    timestamp: String,
}

/// A single row of a BSE equity bhavcopy CSV.
#[derive(Debug, Deserialize)]
struct BseRow {
    #[serde(rename = "Scrip_Code", default)]
    scrip_code: String,
    #[serde(rename = "Scrip_Name", default)]
    scrip_name: String,
    #[serde(rename = "Open", default)]
    open: String,
    #[serde(rename = "High", default)]
    high: String,
    #[serde(rename = "Low", default)]
    low: String,
    #[serde(rename = "Close", default)]
    close: String,
    #[serde(rename = "Date", default)]
    date: String,
}

/// Normalises a Yahoo-style symbol to the exchange's own scrip name.
///
/// `RELIANCE.NS` -> `RELIANCE`, `^NSEI` -> `NIFTY 50` is intentionally not
/// supported because the bhavcopy only covers listed equities.
fn scrip_from_symbol(symbol: &str) -> String {
    symbol
        .split('.')
        .next()
        .unwrap_or(symbol)
        .trim()
        .to_ascii_uppercase()
}

/// Parses a Stooq/NSE style `DD-MMM-YYYY` or `YYYY-MM-DD` date.
fn parse_row_date(raw: &str) -> Option<NaiveDate> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    for fmt in ["%d-%b-%Y", "%Y-%m-%d", "%d-%m-%Y"] {
        if let Ok(d) = NaiveDate::parse_from_str(raw, fmt) {
            return Some(d);
        }
    }
    None
}

/// Parses a numeric CSV cell, tolerating `-`, blanks and stray whitespace.
fn parse_num(raw: &str) -> Option<f64> {
    let t = raw.trim();
    if t.is_empty() || t == "-" || t == "NA" || t == "N/A" {
        return None;
    }
    t.parse::<f64>().ok().filter(|v| v.is_finite())
}

/// Unix timestamp at market close for the given trading date.
fn close_timestamp(date: NaiveDate) -> i64 {
    Utc.from_utc_datetime(&date.and_hms_opt(15, 30, 0).unwrap_or_default())
        .timestamp()
}

/// Downloads and parses the official daily settlement file provider.
#[derive(Debug, Clone)]
pub struct BhavcopyProvider {
    client: reqwest::Client,
}

impl BhavcopyProvider {
    /// Build a provider with browser-like headers, which the archive host
    /// requires before it will serve the file.
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(UA)
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| BtError::DataFetch(e.to_string()))?;
        Ok(Self { client })
    }

    /// URL of the NSE equity bhavcopy for a trading date.
    pub fn nse_url(date: NaiveDate) -> String {
        let month = month_abbr(date.month());
        format!(
            "{}/{}/{}/cm{:02}{}{}bhav.csv.zip",
            NSE_ARCHIVE,
            date.year(),
            month,
            date.day(),
            month,
            date.year()
        )
    }

    /// URL of the BSE equity bhavcopy for a trading date.
    pub fn bse_url(date: NaiveDate) -> String {
        format!(
            "{}/EQ_ISIN_{:02}{:02}{:02}.zip",
            BSE_GROSS,
            date.day(),
            date.month(),
            date.year() % 100
        )
    }

    async fn download(&self, url: &str) -> Result<Vec<u8>> {
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| BtError::DataFetch(format!("bhavcopy request failed: {e}")))?;
        if !resp.status().is_success() {
            return Err(BtError::DataFetch(format!(
                "bhavcopy HTTP {} for {url}",
                resp.status()
            )));
        }
        resp.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| BtError::DataFetch(e.to_string()))
    }

    /// Reads the first CSV entry out of a ZIP archive in memory.
    fn first_csv_from_zip(bytes: &[u8]) -> Result<String> {
        let cursor = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| BtError::InvalidInput(format!("invalid bhavcopy zip: {e}")))?;
        // Prefer a `.csv` entry; fall back to the first entry.
        let mut target = 0usize;
        for i in 0..archive.len() {
            let name = archive
                .by_index(i)
                .map(|f| f.name().to_ascii_lowercase())
                .unwrap_or_default();
            if name.ends_with(".csv") {
                target = i;
                break;
            }
        }
        let mut file = archive
            .by_index(target)
            .map_err(|e| BtError::InvalidInput(e.to_string()))?;
        let mut text = String::new();
        file.read_to_string(&mut text)
            .map_err(|e| BtError::InvalidInput(format!("bhavcopy is not utf-8: {e}")))?;
        Ok(text)
    }

    /// Parses NSE equity bhavcopy CSV text into candles.
    pub fn parse_nse_csv(text: &str, fallback_date: NaiveDate) -> Vec<Candle> {
        let mut reader = csv::Reader::from_reader(text.as_bytes());
        let mut out = Vec::new();
        for record in reader.deserialize::<NseRow>() {
            let Ok(row) = record else { continue };
            let (Some(o), Some(h), Some(l), Some(c)) = (
                parse_num(&row.open),
                parse_num(&row.high),
                parse_num(&row.low),
                parse_num(&row.close),
            ) else {
                continue;
            };
            if o <= 0.0 || h <= 0.0 || l <= 0.0 || c <= 0.0 {
                continue;
            }
            // Enforce OHLC consistency: a malformed row would otherwise draw
            // an inverted wick that no real feed can produce.
            let hi = h.max(o).max(c);
            let lo = l.min(o).min(c);
            if hi < lo {
                continue;
            }
            let date = parse_row_date(&row.timestamp).unwrap_or(fallback_date);
            out.push(Candle::new(
                close_timestamp(date) as f64,
                o,
                hi,
                lo,
                c,
                parse_num(&row.qty).unwrap_or(0.0),
            ));
        }
        out
    }

    /// Parses BSE equity bhavcopy CSV text into candles.
    ///
    /// BSE identifies scrips by numeric code, so rows are returned keyed by
    /// name via [`BhavcopyProvider::find_bse_by_name`].
    pub fn parse_bse_csv(text: &str, fallback_date: NaiveDate) -> Vec<(String, Candle)> {
        let mut reader = csv::Reader::from_reader(text.as_bytes());
        let mut out = Vec::new();
        for record in reader.deserialize::<BseRow>() {
            let Ok(row) = record else { continue };
            let (Some(o), Some(h), Some(l), Some(c)) = (
                parse_num(&row.open),
                parse_num(&row.high),
                parse_num(&row.low),
                parse_num(&row.close),
            ) else {
                continue;
            };
            if o <= 0.0 || h <= 0.0 || l <= 0.0 || c <= 0.0 {
                continue;
            }
            let hi = h.max(o).max(c);
            let lo = l.min(o).min(c);
            if hi < lo {
                continue;
            }
            let date = parse_row_date(&row.date).unwrap_or(fallback_date);
            out.push((
                format!("{}|{}", row.scrip_code.trim(), row.scrip_name.trim()),
                Candle::new(close_timestamp(date) as f64, o, hi, lo, c, 0.0),
            ));
        }
        out
    }

    /// Downloads the NSE settlement file for `date` and returns every scrip
    /// keyed by its exchange symbol.
    pub async fn fetch_nse_day(&self, date: NaiveDate) -> Result<Vec<(String, Candle)>> {
        let url = Self::nse_url(date);
        let bytes = self.download(&url).await?;
        let text = Self::first_csv_from_zip(&bytes)?;
        Ok(Self::parse_nse_csv_keyed(&text, date))
    }

    /// Parses NSE equity bhavcopy CSV text, keeping each scrip's symbol.
    pub fn parse_nse_csv_keyed(text: &str, fallback_date: NaiveDate) -> Vec<(String, Candle)> {
        let mut reader = csv::Reader::from_reader(text.as_bytes());
        let mut out = Vec::new();
        for record in reader.deserialize::<NseRow>() {
            let Ok(row) = record else { continue };
            let (Some(o), Some(h), Some(l), Some(c)) = (
                parse_num(&row.open),
                parse_num(&row.high),
                parse_num(&row.low),
                parse_num(&row.close),
            ) else {
                continue;
            };
            if o <= 0.0 || h <= 0.0 || l <= 0.0 || c <= 0.0 {
                continue;
            }
            let hi = h.max(o).max(c);
            let lo = l.min(o).min(c);
            if hi < lo {
                continue;
            }
            let date = parse_row_date(&row.timestamp).unwrap_or(fallback_date);
            let sym = row.symbol.trim().to_ascii_uppercase();
            if sym.is_empty() {
                continue;
            }
            out.push((
                sym,
                Candle::new(
                    close_timestamp(date) as f64,
                    o,
                    hi,
                    lo,
                    c,
                    parse_num(&row.qty).unwrap_or(0.0),
                ),
            ));
        }
        out
    }

    /// Downloads the NSE settlement file and returns the candle for one symbol.
    pub async fn fetch_nse_symbol(&self, symbol: &str, date: NaiveDate) -> Result<OhlcvSeries> {
        let want = scrip_from_symbol(symbol);
        let rows = self.fetch_nse_day(date).await?;
        let candles: Vec<Candle> = rows
            .into_iter()
            .filter(|(sym, _)| *sym == want)
            .map(|(_, c)| c)
            .collect();
        if candles.is_empty() {
            return Err(BtError::EmptySeries(symbol.to_string()));
        }
        Ok(OhlcvSeries::new(symbol, candles))
    }

    /// Downloads the BSE settlement file for `date`.
    pub async fn fetch_bse_day(&self, date: NaiveDate) -> Result<Vec<(String, Candle)>> {
        let url = Self::bse_url(date);
        let bytes = self.download(&url).await?;
        let text = Self::first_csv_from_zip(&bytes)?;
        Ok(Self::parse_bse_csv(&text, date))
    }
}

impl Default for BhavcopyProvider {
    fn default() -> Self {
        Self::new().expect("bhavcopy http client")
    }
}

/// Three-letter month abbreviation used by the NSE archive paths.
fn month_abbr(month: u32) -> &'static str {
    match month {
        1 => "JAN",
        2 => "FEB",
        3 => "MAR",
        4 => "APR",
        5 => "MAY",
        6 => "JUN",
        7 => "JUL",
        8 => "AUG",
        9 => "SEP",
        10 => "OCT",
        11 => "NOV",
        12 => "DEC",
        _ => "JAN",
    }
}

/// Most recent weekday on or before `date`, which is where a settlement file
/// for "today" will actually be published after the close.
pub fn latest_settlement_date(now: DateTime<Utc>) -> NaiveDate {
    let mut d = now.date_naive();
    while d.weekday().num_days_from_monday() >= 5 {
        d -= chrono::Duration::days(1);
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "SYMBOL,SERIES,OPEN,HIGH,LOW,CLOSE,LAST,PREVCLOSE,TOTTRDQTY,TOTTRDVAL,TIMESTAMP,TOTALTRADES,ISIN\n\
REL,EQ,100.0,105.5,99.5,104.0,104.0,103.0,15000,1560000,15-JAN-2024,42,INE000000001\n\
BAD,EQ,0,0,0,0,0,0,0,0,15-JAN-2024,0,\n\
NEG,,-,-,-,-,-,-,-,-,-,-\n";

    #[test]
    fn test_nse_url_matches_official_layout() {
        let d = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        assert_eq!(
            BhavcopyProvider::nse_url(d),
            "https://archives.nseindia.com/content/historical/EQUITIES/2024/JAN/cm15JAN2024bhav.csv.zip"
        );
    }

    #[test]
    fn test_bse_url_matches_official_layout() {
        let d = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        assert_eq!(
            BhavcopyProvider::bse_url(d),
            "https://www.bseindia.com/BSEDATA/Gross/Public/EQ_ISIN_150124.zip"
        );
    }

    #[test]
    fn test_nse_url_pads_day_and_month() {
        let d = NaiveDate::from_ymd_opt(2024, 12, 5).unwrap();
        assert!(BhavcopyProvider::nse_url(d).ends_with("cm05DEC2024bhav.csv.zip"));
    }

    #[test]
    fn test_parse_nse_csv_reads_a_valid_row() {
        let d = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let candles = BhavcopyProvider::parse_nse_csv(SAMPLE, d);
        assert_eq!(candles.len(), 1, "zero and missing rows are dropped");
        let c = &candles[0];
        assert!((c.open - 100.0).abs() < 1e-9);
        assert!((c.high - 105.5).abs() < 1e-9);
        assert!((c.low - 99.5).abs() < 1e-9);
        assert!((c.close - 104.0).abs() < 1e-9);
        assert!((c.volume - 15_000.0).abs() < 1e-9);
        assert_eq!(c.t as i64, close_timestamp(d));
    }

    #[test]
    fn test_parse_nse_csv_drops_zero_and_blank_rows() {
        let d = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let candles = BhavcopyProvider::parse_nse_csv(SAMPLE, d);
        assert!(candles.iter().all(|c| c.close > 0.0));
    }

    #[test]
    fn test_parse_nse_csv_normalises_inconsistent_ohlc() {
        // A row whose HIGH is below CLOSE must be widened, never drawn inverted.
        let csv = "SYMBOL,SERIES,OPEN,HIGH,LOW,CLOSE,TOTTRDQTY,TIMESTAMP\nX,EQ,10,11,9,12,100,15-JAN-2024\n";
        let d = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let candles = BhavcopyProvider::parse_nse_csv(csv, d);
        assert_eq!(candles.len(), 1);
        let c = &candles[0];
        assert!(c.high >= c.open.max(c.close), "high must bound the body");
        assert!(c.low <= c.open.min(c.close), "low must bound the body");
    }

    #[test]
    fn test_scrip_from_symbol_strips_exchange_suffix() {
        assert_eq!(scrip_from_symbol("RELIANCE.NS"), "RELIANCE");
        assert_eq!(scrip_from_symbol("tcs.ns"), "TCS");
        assert_eq!(scrip_from_symbol("AAPL"), "AAPL");
    }

    #[test]
    fn test_parse_row_date_accepts_common_formats() {
        let d = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        assert_eq!(parse_row_date("15-JAN-2024"), Some(d));
        assert_eq!(parse_row_date("2024-01-15"), Some(d));
        assert_eq!(parse_row_date(""), None);
        assert_eq!(parse_row_date("nonsense"), None);
    }

    #[test]
    fn test_parse_num_tolerates_placeholders() {
        assert_eq!(parse_num("100.5"), Some(100.5));
        assert_eq!(parse_num(" - "), None);
        assert_eq!(parse_num("NA"), None);
        assert_eq!(parse_num(""), None);
        assert_eq!(parse_num("NaN"), None);
    }

    #[test]
    fn test_month_abbr_covers_every_month() {
        let expect = [
            "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
        ];
        for (i, m) in expect.iter().enumerate() {
            assert_eq!(month_abbr(i as u32 + 1), *m);
        }
        // Out-of-range falls back rather than panicking.
        assert_eq!(month_abbr(0), "JAN");
        assert_eq!(month_abbr(13), "JAN");
    }

    #[test]
    fn test_latest_settlement_date_skips_the_weekend() {
        // 2024-01-13 is a Saturday, so the previous weekday is Friday the 12th.
        let sat = DateTime::parse_from_rfc3339("2024-01-13T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(
            latest_settlement_date(sat),
            NaiveDate::from_ymd_opt(2024, 1, 12).unwrap()
        );
        // A weekday is returned unchanged.
        let wed = DateTime::parse_from_rfc3339("2024-01-17T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(
            latest_settlement_date(wed),
            NaiveDate::from_ymd_opt(2024, 1, 17).unwrap()
        );
    }

    #[test]
    fn test_bse_csv_parsing_reads_keyed_rows() {
        let csv = "Scrip_Code,Scrip_Name,Open,High,Low,Close,Date\n500325,RELIANCE,100,105,99,104,15-Jan-2024\n";
        let d = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let rows = BhavcopyProvider::parse_bse_csv(csv, d);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "500325|RELIANCE");
        assert!((rows[0].1.close - 104.0).abs() < 1e-9);
    }

    #[test]
    fn test_empty_csv_yields_no_candles() {
        let d = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        assert!(BhavcopyProvider::parse_nse_csv("", d).is_empty());
        assert!(BhavcopyProvider::parse_bse_csv("", d).is_empty());
    }

    #[test]
    fn test_keyed_parse_keeps_each_scrip_symbol() {
        let csv = "SYMBOL,SERIES,OPEN,HIGH,LOW,CLOSE,TOTTRDQTY,TIMESTAMP\n\
RELIANCE,EQ,100,105,99,104,500,15-JAN-2024\n\
TCS,EQ,200,210,195,205,700,15-JAN-2024\n\
,EQ,1,1,1,1,1,15-JAN-2024\n";
        let d = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let rows = BhavcopyProvider::parse_nse_csv_keyed(csv, d);
        assert_eq!(rows.len(), 2, "blank symbol rows are dropped");
        assert_eq!(rows[0].0, "RELIANCE");
        assert_eq!(rows[1].0, "TCS");
        // Each scrip keeps its own price, so symbols cannot be mixed up.
        assert!((rows[1].1.close - 205.0).abs() < 1e-9);
    }

    #[test]
    fn test_symbol_lookup_selects_only_the_requested_scrip() {
        let csv = "SYMBOL,SERIES,OPEN,HIGH,LOW,CLOSE,TOTTRDQTY,TIMESTAMP\n\
RELIANCE,EQ,100,105,99,104,500,15-JAN-2024\n\
TCS,EQ,200,210,195,205,700,15-JAN-2024\n";
        let d = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let rows = BhavcopyProvider::parse_nse_csv_keyed(csv, d);
        let want = scrip_from_symbol("TCS.NS");
        let picked: Vec<&bt_core::Candle> = rows
            .iter()
            .filter(|(s, _)| *s == want)
            .map(|(_, c)| c)
            .collect();
        assert_eq!(picked.len(), 1);
        assert!((picked[0].close - 205.0).abs() < 1e-9);
    }
}
