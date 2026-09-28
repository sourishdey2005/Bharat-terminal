// crates/bt-data/src/symbol.rs
// Author: Sourish Dey

use bt_core::{APP_NAME, AUTHOR, TAGLINE};

/// Company list with 50+ entries: (display_name, ticker, exchange)
/// Covers NSE India, BSE India, NYSE, NASDAQ, Crypto, and Indices.
pub const COMPANY_LIST: &[(&str, &str, &str)] = &[
    // Indian — NSE (Nifty 50 constituents + more)
    ("Reliance Industries", "RELIANCE.NS", "NSE"),
    ("Tata Consultancy Services", "TCS.NS", "NSE"),
    ("Infosys", "INFY.NS", "NSE"),
    ("HDFC Bank", "HDFCBANK.NS", "NSE"),
    ("ICICI Bank", "ICICIBANK.NS", "NSE"),
    ("State Bank of India", "SBIN.NS", "NSE"),
    ("Bharti Airtel", "BHARTIARTL.NS", "NSE"),
    ("ITC", "ITC.NS", "NSE"),
    ("Larsen & Toubro", "LT.NS", "NSE"),
    ("Axis Bank", "AXISBANK.NS", "NSE"),
    ("Kotak Mahindra Bank", "KOTAKBANK.NS", "NSE"),
    ("Maruti Suzuki", "MARUTI.NS", "NSE"),
    ("Asian Paints", "ASIANPAINT.NS", "NSE"),
    ("Bajaj Finance", "BAJFINANCE.NS", "NSE"),
    ("Sun Pharma", "SUNPHARMA.NS", "NSE"),
    ("Wipro", "WIPRO.NS", "NSE"),
    ("HCL Technologies", "HCLTECH.NS", "NSE"),
    ("Titan Company", "TITAN.NS", "NSE"),
    ("UltraTech Cement", "ULTRACEMCO.NS", "NSE"),
    ("Nestle India", "NESTLEIND.NS", "NSE"),
    ("Tata Motors", "TATAMOTORS.NS", "NSE"),
    ("Tata Steel", "TATASTEEL.NS", "NSE"),
    ("Adani Enterprises", "ADANIENT.NS", "NSE"),
    ("Adani Ports", "ADANIPORTS.NS", "NSE"),
    ("Bajaj Auto", "BAJAJ-AUTO.NS", "NSE"),
    ("Coal India", "COALINDIA.NS", "NSE"),
    ("Divi's Laboratories", "DIVISLAB.NS", "NSE"),
    ("Dr Reddy's Laboratories", "DRREDDY.NS", "NSE"),
    ("Eicher Motors", "EICHERMOT.NS", "NSE"),
    ("Grasim Industries", "GRASIM.NS", "NSE"),
    ("Hero MotoCorp", "HEROMOTOCO.NS", "NSE"),
    ("Hindalco", "HINDALCO.NS", "NSE"),
    ("Hindustan Unilever", "HINDUNILVR.NS", "NSE"),
    ("IndusInd Bank", "INDUSINDBK.NS", "NSE"),
    ("JSW Steel", "JSWSTEEL.NS", "NSE"),
    ("Mahindra & Mahindra", "M&M.NS", "NSE"),
    ("NTPC", "NTPC.NS", "NSE"),
    ("ONGC", "ONGC.NS", "NSE"),
    ("Power Grid", "POWERGRID.NS", "NSE"),
    ("Tech Mahindra", "TECHM.NS", "NSE"),
    ("Tata Consumer Products", "TATACONSUM.NS", "NSE"),
    ("UPL", "UPL.NS", "NSE"),

    // US — Mega Cap Tech
    ("Apple", "AAPL", "NASDAQ"),
    ("Microsoft", "MSFT", "NASDAQ"),
    ("Google (Alphabet)", "GOOGL", "NASDAQ"),
    ("Amazon", "AMZN", "NASDAQ"),
    ("Tesla", "TSLA", "NASDAQ"),
    ("NVIDIA", "NVDA", "NASDAQ"),
    ("Meta Platforms", "META", "NASDAQ"),
    ("Netflix", "NFLX", "NASDAQ"),

    // US — Blue Chip
    ("JPMorgan Chase", "JPM", "NYSE"),
    ("Berkshire Hathaway", "BRK-B", "NYSE"),
    ("Visa", "V", "NYSE"),
    ("Johnson & Johnson", "JNJ", "NYSE"),
    ("Walmart", "WMT", "NYSE"),
    ("Exxon Mobil", "XOM", "NYSE"),
    ("Procter & Gamble", "PG", "NYSE"),
    ("Mastercard", "MA", "NYSE"),
    ("UnitedHealth", "UNH", "NYSE"),
    ("Home Depot", "HD", "NYSE"),

    // Crypto (Coinbase / Yahoo)
    ("Bitcoin", "BTC-USD", "Crypto"),
    ("Ethereum", "ETH-USD", "Crypto"),
    ("Solana", "SOL-USD", "Crypto"),
    ("Cardano", "ADA-USD", "Crypto"),
    ("Dogecoin", "DOGE-USD", "Crypto"),
    ("Ripple (XRP)", "XRP-USD", "Crypto"),
    ("Polkadot", "DOT-USD", "Crypto"),

    // Major Indices
    ("Nifty 50", "^NSEI", "Index"),
    ("Sensex", "^BSESN", "Index"),
    ("S&P 500", "^GSPC", "Index"),
    ("NASDAQ Composite", "^IXIC", "Index"),
    ("Dow Jones Industrial Average", "^DJI", "Index"),
    ("Russell 2000", "^RUT", "Index"),
    ("VIX", "^VIX", "Index"),
    ("US 10Y Treasury", "^TNX", "Index"),
    ("US Dollar Index", "DX-Y.NYB", "Index"),
    ("Gold", "GC=F", "Commodity"),
    ("Crude Oil WTI", "CL=F", "Commodity"),
];

/// Resolve a friendly name or ticker to a Yahoo Finance symbol.
/// Falls back to returning the input as-is if not found.
pub fn resolve(name_or_ticker: &str) -> String {
    for (name, ticker, _) in COMPANY_LIST {
        if name.eq_ignore_ascii_case(name_or_ticker)
            || ticker.eq_ignore_ascii_case(name_or_ticker)
        {
            return ticker.to_string();
        }
    }
    name_or_ticker.to_string()
}

/// Get display name for a ticker (reverse lookup).
pub fn display_name(ticker: &str) -> String {
    for (name, t, _) in COMPANY_LIST {
        if t.eq_ignore_ascii_case(ticker) {
            return name.to_string();
        }
    }
    ticker.to_string()
}

/// Get exchange for a ticker.
pub fn exchange(ticker: &str) -> &str {
    for (_, t, ex) in COMPANY_LIST {
        if t.eq_ignore_ascii_case(ticker) {
            return ex;
        }
    }
    "Unknown"
}

/// Get all unique exchanges.
pub fn all_exchanges() -> Vec<&'static str> {
    let mut exchanges: Vec<&'static str> = COMPANY_LIST
        .iter()
        .map(|(_, _, ex)| *ex)
        .collect();
    exchanges.sort();
    exchanges.dedup();
    exchanges
}

/// Get companies filtered by exchange.
pub fn companies_by_exchange(exchange: &str) -> Vec<(&'static str, &'static str, &'static str)> {
    COMPANY_LIST
        .iter()
        .filter(|(_, _, ex)| ex.eq_ignore_ascii_case(exchange))
        .copied()
        .collect()
}

/// Search companies by name or ticker (case-insensitive substring).
pub fn search_companies(query: &str) -> Vec<(&'static str, &'static str, &'static str)> {
    let q = query.to_lowercase();
    COMPANY_LIST
        .iter()
        .filter(|(name, ticker, _)| {
            name.to_lowercase().contains(&q) || ticker.to_lowercase().contains(&q)
        })
        .copied()
        .collect()
}

/// Default company for initial selection.
pub const DEFAULT_COMPANY: (&str, &str, &str) = ("Reliance Industries", "RELIANCE.NS", "NSE");

/// Application metadata constants.
pub fn app_info() -> String {
    format!("{} v{} — Made by {}. {}", APP_NAME, env!("CARGO_PKG_VERSION"), AUTHOR, TAGLINE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_known_names() {
        assert_eq!(resolve("Reliance Industries"), "RELIANCE.NS");
        assert_eq!(resolve("Apple"), "AAPL");
        assert_eq!(resolve("Bitcoin"), "BTC-USD");
        assert_eq!(resolve("Nifty 50"), "^NSEI");
    }

    #[test]
    fn test_resolve_tickers() {
        assert_eq!(resolve("RELIANCE.NS"), "RELIANCE.NS");
        assert_eq!(resolve("AAPL"), "AAPL");
        assert_eq!(resolve("BTC-USD"), "BTC-USD");
    }

    #[test]
    fn test_resolve_unknown_fallback() {
        assert_eq!(resolve("UNKNOWN"), "UNKNOWN");
        assert_eq!(resolve("RANDOM.TICKER"), "RANDOM.TICKER");
    }

    #[test]
    fn test_display_name() {
        assert_eq!(display_name("RELIANCE.NS"), "Reliance Industries");
        assert_eq!(display_name("AAPL"), "Apple");
        assert_eq!(display_name("BTC-USD"), "Bitcoin");
    }

    #[test]
    fn test_exchange_lookup() {
        assert_eq!(exchange("RELIANCE.NS"), "NSE");
        assert_eq!(exchange("AAPL"), "NASDAQ");
        assert_eq!(exchange("BTC-USD"), "Crypto");
    }

    #[test]
    fn test_search() {
        let results = search_companies("reliance");
        assert!(!results.is_empty());
        assert_eq!(results[0].0, "Reliance Industries");

        let results = search_companies("bank");
        assert!(results.len() >= 4); // HDFC, ICICI, Axis, Kotak, SBI, IndusInd
    }

    #[test]
    fn test_companies_by_exchange() {
        let nse = companies_by_exchange("NSE");
        assert!(nse.len() >= 40);

        let nasdaq = companies_by_exchange("NASDAQ");
        assert!(nasdaq.len() >= 8);

        let crypto = companies_by_exchange("Crypto");
        assert!(crypto.len() >= 7);
    }

    #[test]
    fn test_default_company() {
        assert_eq!(DEFAULT_COMPANY.1, "RELIANCE.NS");
        assert!(resolve(&DEFAULT_COMPANY.1) == "RELIANCE.NS");
    }

    #[test]
fn test_app_info_contains_author() {
    let info = app_info();
    assert!(info.contains("Sourish Dey"));
    assert!(info.contains("Bloomberg power"));
    assert!(info.contains("Made in India"));
}
}