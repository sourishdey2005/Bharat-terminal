# Bharat Terminal

**Bloomberg power. Zero cost. Made in India.**

A free, open-source Bloomberg Terminal alternative built in Rust. 140+ financial visualizations, real market data from Yahoo Finance + Coinbase, native Windows desktop application. MIT licensed.

---

## Overview

Bharat Terminal democratizes institutional-grade financial analysis. Built from the ground up in Rust, it delivers 140+ professional visualizations — candlesticks, Bollinger bands, options Greeks, risk analytics, order flow, India-specific charts — all powered by real market data without API keys.

| Feature | Bharat Terminal | Bloomberg | TradingView | Zerodha Kite |
|---------|----------------|-----------|-------------|--------------|
| **Price** | **Free** | $24,000/yr | $15/mo | Free |
| **Visualizations** | 140+ | 200+ | 100+ | 20+ |
| **Open Source** | ✅ | ❌ | ❌ | ❌ |
| **API Keys Required** | ❌ | ✅ | ✅ | ✅ |
| **Desktop App** | ✅ | ✅ | ❌ | ❌ |
| **CLI** | ✅ | ❌ | ❌ | ❌ |
| **NSE/BSE Data** | ✅ | ✅ | ✅ | ✅ |

---

## Quick Start

### Windows (Recommended)

```powershell
# Portable — no installation
# 1. Download latest release
# 2. Extract to C:\BharatTerminal
# 3. Run bt-app.exe
```

### From Source

```bash
git clone https://github.com/sourishdey/bharat-terminal.git
cd bharat-terminal
cargo build --release --workspace
# Binary at target/release/bt-app.exe
```

---

## Capabilities

### Market Data (No API Keys)
- **Yahoo Finance** — Global equities, ETFs, indices, futures, FX, crypto
- **Coinbase** — Spot crypto markets
- **78+ Symbols** — NSE/BSE (RELIANCE, TCS, INFY, HDFCBANK…), US (AAPL, MSFT, NVDA), indices (NIFTY 50, S&P 500), crypto (BTC, ETH), commodities (Gold, Oil), FX (USD/INR)

### Visualization Families (140+)
| Family | Count | Examples |
|--------|-------|----------|
| Candlestick | 32 | Volume, MA, Bollinger, Heikin-Ashi, Renko, Pattern detection |
| Bollinger | 15 | Bands, %B, Squeeze, Keltner, Donchian, STARC |
| Indicators | 25 | RSI, MACD, Stochastic, ADX, Ichimoku, Supertrend, VWAP |
| Risk & Portfolio | 15 | Drawdown, Efficient Frontier, Correlation, VaR, Rolling Sharpe |
| Options & Volatility | 15 | Vol Smile/Surface, Greeks, Monte Carlo, Strategy Lab |
| Statistical | 10 | ACF/PACF, QQ Plot, Hurst, GARCH, Regime HMM |
| Order Flow | 10 | Cumulative Delta, Footprint, Volume Profile, Market Depth |
| India-Specific | 10 | NSE Heatmap, Sector Treemap, FII/DII Flow, Circuit Map |
| Bloomberg-Style | 8 | Yield Curve, FX Board, Commodity Pit, Watchlist Pro |

### Performance
- **60ms** for 10 charts (SIMD + Rayon parallelism)
- **842ms** for all 140 visualizations end-to-end
- SQLite caching — works offline on cached data
- 30-second live auto-refresh

### Distribution
| Platform | Format |
|----------|--------|
| Windows 10+ | `.exe` (Setup), `.zip` (Portable), `.msi` |
| macOS 11+ | `.dmg`, Homebrew `brew install --cask bharat-terminal` |
| Linux | `.AppImage`, `.deb`, `.rpm`, AUR, Snap |
| Source | `cargo build --release` |

---

## Architecture

```
bharat-terminal/
├── crates/
│   ├── bt-core       # Core types: Candle, OhlcvSeries, Range
│   ├── bt-data       # Data providers (Yahoo, Coinbase) + SQLite cache
│   ├── bt-analytics  # 140+ analytics engines (indicators, risk, options)
│   ├── bt-viz        # Rendering: PNG output, chart primitives
│   ├── bt-cli        # CLI: --symbol, --live, --list-companies
│   └── bt-app        # eGUI desktop application
├── data/             # Company lists, defaults
└── scripts/          # Installer builders (MSI, Inno Setup, NSIS)
```

**Tech Stack**: Rust 2021, Tokio, Rayon, eGUI, SQLite (rusqlite), reqwest, serde, plotters

---

## CLI Usage

```bash
# Live chart with 30s auto-refresh
bt-cli --symbol RELIANCE.NS --live

# Yearly history
bt-cli --symbol TCS.NS --range 1Y

# List all 78 supported symbols
bt-cli --list-companies

# Light theme output
bt-cli --symbol BTC-USD --theme light
```

---

## Project Structure (Website)

This repository also contains the official marketing & documentation site at `/website`:

- **Framework**: Next.js 14 (App Router) + TypeScript
- **Styling**: Tailwind CSS + CSS Variables (cinematic design system)
- **Motion**: Framer Motion (11 animation types integrated)
- **Charts**: TradingView Lightweight Charts (live demo)
- **Deploy**: Vercel (free tier)

```bash
cd website
npm install
npm run dev     # http://localhost:3000
npm run build   # Production build
```

---

## Design System

- **Typography**: Manrope (variable 200–800), JetBrains Mono
- **Colors**: Void `#050505`, Amber `#e8b43c`, Silver `#b6b5b5`, White pills
- **Motion**: 11 animation types (microinteractions, hero, loading, page transitions, scroll, character, 3D, morphing, icons, ambient, stop-motion)
- **Unit System**: Height-locked `--u` (1/1058 viewport height) with responsive `--h` clamp
- **Accessibility**: WCAG 2.1 AA, `prefers-reduced-motion` respected

---

## License

MIT License — Copyright (c) 2026 Sourish Dey

Free forever. No subscriptions. No tiers. No limits.

---

## Community

- **GitHub**: [github.com/sourishdey/bharat-terminal](https://github.com/sourishdey/bharat-terminal)
- **Issues**: Bug reports, feature requests, contributions welcome
- **Author**: Sourish Dey — Built in India 🇮🇳

---

*Made by Sourish Dey. Bloomberg power. Zero cost. Made in India.*