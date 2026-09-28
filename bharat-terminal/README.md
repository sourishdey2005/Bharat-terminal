# BHARAT TERMINAL v3

**Bloomberg power. Zero cost. Made in India.**

**Author:** Sourish Dey  
**Version:** 3.0.0  
**License:** MIT

---

## Overview

Bharat Terminal v3 is a free, open-source market data terminal for Indian and global markets. It provides 160+ real-time visualizations, live NSE/BSE data, F&O analytics, macro indicators, and a Bloomberg-style interface — all in a single Windows executable.

## Download

| File | Description | Size |
|------|-------------|------|
| [BharatTerminal-v3.0.0.exe](releases/BharatTerminal-v3.0.0.exe) | Desktop GUI (standalone) | ~8 MB |
| [BharatTerminal-v3.0.0-cli.exe](releases/BharatTerminal-v3.0.0-cli.exe) | Command-line tool | ~4 MB |

**System Requirements:** Windows 10/11 (64-bit), 4 GB RAM, 500 MB disk space

## Quick Start

1. Download `BharatTerminal-v3.0.0.exe` from the [releases](releases/) folder
2. Double-click to run — no installation required
3. The app creates `./data/` on first run for cache and preferences

## Features

### Market Data
- **NSE/BSE:** 78 companies (Reliance, TCS, Infosys, HDFC Bank, ICICI Bank, and more)
- **US Markets:** Apple, Microsoft, Google, Amazon, Tesla, NVIDIA, Meta, Netflix
- **Crypto:** Bitcoin, Ethereum, Solana, Cardano, Dogecoin (via Coinbase)
- **Indices:** Nifty 50, Sensex, S&P 500, NASDAQ, Dow Jones, VIX
- **F&O:** Options chain with IV, OI, Greeks
- **G-Sec:** Government securities yield curve
- **Money Market:** MIBOR, TREPS, CP/CD rates
- **Macro:** CPI, WPI, IIP, GDP, PMI, RBI policy rates
- **Commodities:** MCX, NCDEX
- **Mutual Funds:** NAV, AUM, category analytics

### Visualizations (160+)
- **Price Action:** Candlestick, Heikin-Ashi, Renko, Kagi, Point & Figure, Triangle-Stick (8 variants), Hollow Candles, Heatmaps
- **Order Flow:** Volume Profile, Footprint, Order Book Heatmap, Cumulative Delta, Market Profile
- **Indicators:** RSI, MACD, Stochastic, ATR, OBV, VWAP, Bollinger, ADX, CCI, Williams %R, ROC, CMF, Ichimoku, Keltner, Donchian
- **Risk:** Drawdown, Correlation, VaR, Monte Carlo, Rolling Sharpe/Sortino, Beta/Alpha
- **Volatility:** IV Surface, Term Structure, Greeks, VIX, Option Payoff, Gamma Exposure
- **3D Charts:** Candle Surface, Vol Surface, Correlation Sphere, Monte Carlo Cloud, Yield Terrain
- **India-Specific:** Nifty Treemap, Sensex Heatmap, FII/DII, Sector Performance, Yield Curve, USD/INR
- **Comparison:** Multi-Compare (2-5 stocks side-by-side), Normalized Price, Performance, Volatility

### Key Features
- **Live Mode:** Auto-refresh every 30 seconds
- **Multi-Compare:** Select 2-5 stocks and compare normalized price, returns, volatility, Sharpe, max drawdown
- **MarketWatch:** Bloomberg-style market overview table with 33 stocks
- **Company Dropdown:** Search and select from 78 companies
- **Time Ranges:** 1D, 1W, 1M, 3M, 6M, 1Y, 5Y
- **Theme Toggle:** Dark/Light mode
- **Preferences:** Auto-saved to `./data/prefs.json`
- **SQLite Cache:** 5-min TTL for intraday, 24h for daily data
- **Error Handling:** Graceful fallback to synthetic data — never crashes

## CLI Usage

```cmd
BharatTerminal-v3.0.0-cli.exe --list-companies
BharatTerminal-v3.0.0-cli.exe --symbol RELIANCE.NS --range 1y --live
BharatTerminal-v3.0.0-cli.exe --symbol AAPL --range 6mo --out-dir output_aapl
```

## Building from Source

```cmd
git clone https://github.com/sourishdey2005/Bharat-terminal-exe.git
cd Bharat-terminal-exe
cargo build --release --workspace
```

## Project Structure

```
BharatTerminal/
├── crates/
│   ├── bt-core/       # Shared types and errors
│   ├── bt-data/       # Market data providers (Yahoo, Coinbase, India)
│   ├── bt-analytics/  # Technical indicators and risk metrics
│   ├── bt-viz/        # 160+ visualization modules
│   ├── bt-cli/        # Command-line interface
│   └── bt-app/        # Desktop GUI (egui/eframe)
├── releases/          # Downloadable executables
├── docs/              # Documentation
├── scripts/           # Installer scripts
└── assets/            # Fonts and icons
```

## Data Sources

| Source | Coverage | Rate Limit |
|--------|----------|------------|
| Yahoo Finance | Stocks, ETFs, Indices, Crypto, FX | ~2000/hour |
| Coinbase | Crypto spot markets | Public endpoints |
| SQLite Cache | Local cache with TTL | None |

## Privacy

- No user data is collected or transmitted
- All data is fetched directly from public APIs
- Cache is stored locally in `./data/cache.db`
- Preferences are stored locally in `./data/prefs.json`
- No telemetry, no tracking, no analytics

## License

MIT License — Made by Sourish Dey

---

**Bharat Terminal v3 — Bloomberg power. Zero cost. Made in India.**