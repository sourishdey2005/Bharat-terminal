#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// crates/bt-app/src/main.rs
// Author: Sourish Dey

//! bt-app: BHARAT TERMINAL v3 desktop app (egui/eframe). Made by Sourish Dey.
//!
//! Features real market data from Yahoo Finance and Coinbase, with synthetic
//! fallback. Includes company dropdown, time range selector, live refresh,
//! 163 interactive tabs with category grouping, auto-focus on chart canvas,
//! and prefs persistence via serde_json.

use std::fs;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};

use chrono::{Duration as ChronoDuration, Utc};
use eframe::egui;
use egui::{Color32, Id, RichText, Stroke, Vec2};
use egui_plot::{Bar, BarChart, Legend, Line, MarkerShape, Plot, PlotPoints, Points};
use serde::{Deserialize, Serialize};

use bt_core::{synthetic_correlated_returns, synthetic_ohlcv, APP_NAME, AUTHOR, Candle, OhlcvSeries, TAGLINE};
use bt_data::india;
use bt_data::{symbol::COMPANY_LIST, DataService, Interval};
use bt_viz::palette::Theme;

const AMBER: Color32 = Color32::from_rgb(0xFF, 0xB0, 0x00);
const PROFIT: Color32 = Color32::from_rgb(0x00, 0xFF, 0x88);
const LOSS: Color32 = Color32::from_rgb(0xFF, 0x3B, 0x3B);
const INFO: Color32 = Color32::from_rgb(0x00, 0xBF, 0xFF);
const PURPLE: Color32 = Color32::from_rgb(0xBF, 0x5A, 0xFF);

const DAY_SECS: f64 = 86400.0;
const BAR_WIDTH: f64 = 0.7 * DAY_SECS;
const BAR_HALF: f64 = 0.35 * DAY_SECS;

fn format_ts(ts: f64) -> String {
    let ts = ts as i64;
    let dt = chrono::DateTime::from_timestamp(ts, 0).unwrap_or_default();
    dt.format("%b %Y").to_string()
}

/// Smallest visible body height, as a fraction of the series high/low range.
/// Keeps doji candles (open == close) from collapsing into an invisible line.
const MIN_BODY_FRAC: f64 = 0.004;

/// Vertical size of the trend arrow, as a fraction of the series high/low range.
const ARROW_FRAC: f64 = 0.022;

/// Returns `(min_low, range)` for the series, used to scale candle geometry.
fn price_scale(candles: &[Candle]) -> (f64, f64) {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for c in candles {
        lo = lo.min(c.low);
        hi = hi.max(c.high);
    }
    if !lo.is_finite() || !hi.is_finite() {
        return (0.0, 1.0);
    }
    let range = hi - lo;
    if range <= 0.0 {
        (lo, 1.0)
    } else {
        (lo, range)
    }
}

/// Returns the `(top, bottom)` of the candle body, guaranteeing a visible body
/// for doji candles by expanding around the midpoint when open == close.
fn candle_body(c: &Candle, min_body: f64) -> (f64, f64) {
    let (mut top, mut bottom) = if c.is_bullish() {
        (c.close, c.open)
    } else {
        (c.open, c.close)
    };
    if (top - bottom).abs() < min_body {
        let mid = 0.5 * (top + bottom);
        top = mid + 0.5 * min_body;
        bottom = mid - 0.5 * min_body;
    }
    (top, bottom)
}

/// Draws a professional candlestick: thin high/low wick behind a filled
/// open/close body, coloured green when bullish and red when bearish.
fn draw_candle(plot_ui: &mut egui_plot::PlotUi, c: &Candle, half: f64, min_body: f64) {
    let color = if c.is_bullish() { PROFIT } else { LOSS };
    plot_ui.line(
        Line::new(PlotPoints::from(vec![[c.t, c.low], [c.t, c.high]]))
            .color(color)
            .width(1.0_f32),
    );
    let (top, bottom) = candle_body(c, min_body);
    plot_ui.polygon(
        egui_plot::Polygon::new(PlotPoints::from(vec![
            [c.t - half, bottom],
            [c.t + half, bottom],
            [c.t + half, top],
            [c.t - half, top],
        ]))
        .fill_color(color)
        .stroke(Stroke::new(1.0_f32, color)),
    );
}

/// Converts a series into a true Heikin-Ashi series.
///
/// `ha_close = (o + h + l + c) / 4`, `ha_open[i] = (ha_open[i-1] + ha_close[i-1]) / 2`
/// (seeded with `(o[0] + c[0]) / 2`), and the high/low are widened to contain
/// the synthetic open and close.
fn heikin_ashi(candles: &[Candle]) -> Vec<Candle> {
    let mut out: Vec<Candle> = Vec::with_capacity(candles.len());
    let mut prev: Option<(f64, f64)> = None;
    for c in candles {
        let ha_close = (c.open + c.high + c.low + c.close) / 4.0;
        let ha_open = match prev {
            Some((po, pc)) => 0.5 * (po + pc),
            None => 0.5 * (c.open + c.close),
        };
        out.push(Candle::new(
            c.t,
            ha_open,
            c.high.max(ha_open).max(ha_close),
            c.low.min(ha_open).min(ha_close),
            ha_close,
            c.volume,
        ));
        prev = Some((ha_open, ha_close));
    }
    out
}

/// Draws a green up-arrow below a bullish candle and a red down-arrow above a
/// bearish candle, so the direction of every bar is readable at a glance.
fn draw_trend_arrow(plot_ui: &mut egui_plot::PlotUi, c: &Candle, half: f64, size: f64) {
    let color = if c.is_bullish() { PROFIT } else { LOSS };
    let w = half.max(size * 0.9);
    let pts = if c.is_bullish() {
        let tip = c.low - 1.2 * size;
        vec![
            [c.t, tip + size],
            [c.t - w, tip],
            [c.t + w, tip],
        ]
    } else {
        let tip = c.high + 1.2 * size;
        vec![
            [c.t, tip - size],
            [c.t - w, tip],
            [c.t + w, tip],
        ]
    };
    plot_ui.polygon(
        egui_plot::Polygon::new(PlotPoints::from(pts))
            .fill_color(color.gamma_multiply(0.9))
            .stroke(Stroke::new(0.5_f32, color)),
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Tab {
    Candlestick, HeikinAshi, Renko, Kagi, PointFigure,
    Candlestick3D, CandlestickMA, CandlestickBollinger, CandlestickRSI, CandlestickMACD,
    VolumeProfile, Footprint, OrderBookHeatmap, CumulativeDelta,
    MarketProfile, VolumeClock, TickTape, DeltaDivergence,
    RSI, MACD, Stochastic, ATR, OBV, VWAP,
    Bollinger, BBWidth, ADX, CCI, WilliamsR, ROC,
    CMF, Ichimoku, Keltner, Donchian,
    Drawdown, Correlation, VolSmile, EffFrontier, RollingSharpe,
    RollingSortino, BetaAlpha, RollingMaxDD, VaRBacktest, MonteCarlo,
    VolSmileOpt, IVSurface, TermStructure, GreeksHeatmap, VIXTerm,
    VolCone, OptionPayoff, SkewEvolution, GammaExposure, PutCallRatio,
    IVRank, SharpeSurface,
    AcfPacf, Hurst, Wavelet, Kalman, MarkovRegime,
    Copula3D, QQPlot, ReturnDist, RollingMoments,
    NiftyTreemap, SensexHeatmap, FII_DIIFlow, SectorPerf,
    YieldCurve, USDINR, MonsoonAgri, Seasonality,
    WorldIndices, TickerTape, CurrencyMatrix, SectorWheel,
    EarningsCalendar, EconCalendar, CorrelationNetwork, ReturnHeatmap,
    MultiIndicator, MultiTimeframe, MACDDivergence, BollingerBreakout,
    VolumeWeightedScatter, PriceMomentum, DrawdownRecovery, RollingCorrelation,
    TickTapeAdv, SeasonalityAdv, ParabolicSAR, MACDHistogram,
    RSIHeatmap, IchimokuEMA, KeltnerBreakout, DonchianBreakout,
    CopulaHeatmap, CorrelationNetworkAdv,
    MarketWatch,
    FOChain, IVSurfaceIndia, OIHeatmap, GSec, MoneyMarket, RBIPolicy,
    MacroIndia, CommoditiesIndia, USDINRCurve, YieldIndia, MFAnalytics,
    FPIFII, CreditRatings, BankingIndia, CorpActions, IPOPipeline,
    IndiaBreadth, SectorResearch, IndiaNews, Regulatory, GSTBudget,
    IndiaPortfolio, AlgoFeed, AIResearch, IndiaDashboard,
    MultiCompare,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TabCategory {
    PriceAction, OrderFlow, Indicators, RiskPortfolio,
    VolatilityOptions, Microstructure, IndiaSpecific, BloombergStyle, Advanced, Comparison,
}

impl TabCategory {
    fn label(&self) -> &'static str {
        match self {
            TabCategory::PriceAction => "Price Action",
            TabCategory::OrderFlow => "Order Flow",
            TabCategory::Indicators => "Indicators",
            TabCategory::RiskPortfolio => "Risk & Portfolio",
            TabCategory::VolatilityOptions => "Volatility & Options",
            TabCategory::Microstructure => "Microstructure",
            TabCategory::IndiaSpecific => "India-Specific",
            TabCategory::BloombergStyle => "Bloomberg-Style",
            TabCategory::Advanced => "Advanced",
            TabCategory::Comparison => "Comparison",
        }
    }

    fn color(&self) -> Color32 {
        match self {
            TabCategory::PriceAction => AMBER,
            TabCategory::OrderFlow => INFO,
            TabCategory::Indicators => PROFIT,
            TabCategory::RiskPortfolio => PURPLE,
            TabCategory::VolatilityOptions => LOSS,
            TabCategory::Microstructure => Color32::from_rgb(0x00, 0xD4, 0xAA),
            TabCategory::IndiaSpecific => Color32::from_rgb(0xFF, 0x8C, 0x00),
            TabCategory::BloombergStyle => Color32::from_rgb(0x44, 0x88, 0xFF),
            TabCategory::Advanced => Color32::from_rgb(0xFF, 0x69, 0xB4),
            TabCategory::Comparison => Color32::from_rgb(0x00, 0xE5, 0xFF),
        }
    }
}

const CATEGORIES: [TabCategory; 10] = [
    TabCategory::PriceAction,
    TabCategory::OrderFlow,
    TabCategory::Indicators,
    TabCategory::RiskPortfolio,
    TabCategory::VolatilityOptions,
    TabCategory::Microstructure,
    TabCategory::IndiaSpecific,
    TabCategory::BloombergStyle,
    TabCategory::Advanced,
    TabCategory::Comparison,
];

fn tabs_in_category(cat: TabCategory) -> &'static [(Tab, &'static str)] {
    match cat {
        TabCategory::PriceAction => &[
            (Tab::Candlestick, "Candlestick"),
            (Tab::HeikinAshi, "Heikin-Ashi"),
            (Tab::Renko, "Renko"),
            (Tab::Kagi, "Kagi"),
            (Tab::PointFigure, "Point&Figure"),
            (Tab::Candlestick3D, "Candle3D"),
            (Tab::CandlestickMA, "Candle+MA"),
            (Tab::CandlestickBollinger, "Candle+BB"),
            (Tab::CandlestickRSI, "Candle+RSI"),
            (Tab::CandlestickMACD, "Candle+MACD"),
        ],
        TabCategory::OrderFlow => &[
            (Tab::VolumeProfile, "Volume Profile"),
            (Tab::Footprint, "Footprint"),
            (Tab::OrderBookHeatmap, "OrderBook Heatmap"),
            (Tab::CumulativeDelta, "Cumulative Delta"),
            (Tab::MarketProfile, "Market Profile"),
            (Tab::VolumeClock, "Volume Clock"),
            (Tab::TickTape, "Tick Tape"),
            (Tab::DeltaDivergence, "Delta Divergence"),
        ],
        TabCategory::Indicators => &[
            (Tab::RSI, "RSI"),
            (Tab::MACD, "MACD"),
            (Tab::Stochastic, "Stochastic"),
            (Tab::ATR, "ATR"),
            (Tab::OBV, "OBV"),
            (Tab::VWAP, "VWAP"),
            (Tab::Bollinger, "Bollinger"),
            (Tab::BBWidth, "BB Width"),
            (Tab::ADX, "ADX"),
            (Tab::CCI, "CCI"),
            (Tab::WilliamsR, "Williams %R"),
            (Tab::ROC, "ROC"),
            (Tab::CMF, "CMF"),
            (Tab::Ichimoku, "Ichimoku"),
            (Tab::Keltner, "Keltner"),
            (Tab::Donchian, "Donchian"),
        ],
        TabCategory::RiskPortfolio => &[
            (Tab::Drawdown, "Drawdown"),
            (Tab::Correlation, "Correlation"),
            (Tab::VolSmile, "Vol Smile"),
            (Tab::EffFrontier, "Eff Frontier"),
            (Tab::RollingSharpe, "Rolling Sharpe"),
            (Tab::RollingSortino, "Rolling Sortino"),
            (Tab::BetaAlpha, "Beta/Alpha"),
            (Tab::RollingMaxDD, "Rolling MaxDD"),
            (Tab::VaRBacktest, "VaR Backtest"),
            (Tab::MonteCarlo, "Monte Carlo"),
        ],
        TabCategory::VolatilityOptions => &[
            (Tab::VolSmileOpt, "Vol Smile"),
            (Tab::IVSurface, "IV Surface"),
            (Tab::TermStructure, "Term Structure"),
            (Tab::GreeksHeatmap, "Greeks Heatmap"),
            (Tab::VIXTerm, "VIX Term"),
            (Tab::VolCone, "Vol Cone"),
            (Tab::OptionPayoff, "Option Payoff"),
            (Tab::SkewEvolution, "Skew Evolution"),
            (Tab::GammaExposure, "Gamma Exposure"),
            (Tab::PutCallRatio, "Put/Call Ratio"),
            (Tab::IVRank, "IV Rank"),
            (Tab::SharpeSurface, "Sharpe Surface"),
        ],
        TabCategory::Microstructure => &[
            (Tab::AcfPacf, "ACF/PACF"),
            (Tab::Hurst, "Hurst"),
            (Tab::Wavelet, "Wavelet"),
            (Tab::Kalman, "Kalman"),
            (Tab::MarkovRegime, "Markov Regime"),
            (Tab::Copula3D, "Copula 3D"),
            (Tab::QQPlot, "QQ Plot"),
            (Tab::ReturnDist, "Return Dist"),
            (Tab::RollingMoments, "Rolling Moments"),
        ],
        TabCategory::IndiaSpecific => &[
            (Tab::NiftyTreemap, "Nifty Treemap"),
            (Tab::SensexHeatmap, "Sensex Heatmap"),
            (Tab::FII_DIIFlow, "FII/DII Flow"),
            (Tab::SectorPerf, "Sector Perf"),
            (Tab::YieldCurve, "Yield Curve"),
            (Tab::USDINR, "USD/INR"),
            (Tab::MonsoonAgri, "Monsoon Agri"),
            (Tab::Seasonality, "Seasonality"),
            (Tab::FOChain, "FOChain"),
            (Tab::IVSurfaceIndia, "IV Surface"),
            (Tab::OIHeatmap, "OI Heatmap"),
            (Tab::GSec, "G-Sec"),
            (Tab::MoneyMarket, "Money Mkt"),
            (Tab::RBIPolicy, "RBI Policy"),
            (Tab::MacroIndia, "Macro India"),
            (Tab::CommoditiesIndia, "MCX/NCDEX"),
            (Tab::USDINRCurve, "USDINR Fwd"),
            (Tab::YieldIndia, "Yield India"),
            (Tab::MFAnalytics, "MF Analytics"),
            (Tab::FPIFII, "FPI/FII"),
            (Tab::CreditRatings, "Credit Ratings"),
            (Tab::BankingIndia, "Banking"),
            (Tab::CorpActions, "Corp Actions"),
            (Tab::IPOPipeline, "IPO Pipeline"),
            (Tab::IndiaBreadth, "India Breadth"),
            (Tab::SectorResearch, "Sector Research"),
            (Tab::IndiaNews, "India News"),
            (Tab::Regulatory, "Regulatory"),
            (Tab::GSTBudget, "GST/Budget"),
            (Tab::IndiaPortfolio, "India Portfolio"),
            (Tab::AlgoFeed, "Algo Feed"),
            (Tab::AIResearch, "AI Research"),
            (Tab::IndiaDashboard, "India Dashboard"),
        ],
        TabCategory::BloombergStyle => &[
            (Tab::WorldIndices, "World Indices"),
            (Tab::TickerTape, "Ticker Tape"),
            (Tab::CurrencyMatrix, "Currency Matrix"),
            (Tab::SectorWheel, "Sector Wheel"),
            (Tab::EarningsCalendar, "Earnings Cal"),
            (Tab::EconCalendar, "Econ Calendar"),
            (Tab::CorrelationNetwork, "Corr Network"),
            (Tab::ReturnHeatmap, "Return Heatmap"),
            (Tab::MarketWatch, "MarketWatch"),
        ],
        TabCategory::Advanced => &[
            (Tab::MultiIndicator, "Multi Indicator"),
            (Tab::MultiTimeframe, "Multi Timeframe"),
            (Tab::MACDDivergence, "MACD Divergence"),
            (Tab::BollingerBreakout, "BB Breakout"),
            (Tab::VolumeWeightedScatter, "Vol-Price Scatter"),
            (Tab::PriceMomentum, "Price Momentum"),
            (Tab::DrawdownRecovery, "DD Recovery"),
            (Tab::RollingCorrelation, "Rolling Corr"),
            (Tab::TickTapeAdv, "Tick Tape Adv"),
            (Tab::SeasonalityAdv, "Seasonality Adv"),
            (Tab::ParabolicSAR, "Parabolic SAR"),
            (Tab::MACDHistogram, "MACD Histogram"),
            (Tab::RSIHeatmap, "RSI Heatmap"),
            (Tab::IchimokuEMA, "Ichimoku+EMA"),
            (Tab::KeltnerBreakout, "Keltner Breakout"),
            (Tab::DonchianBreakout, "Donchian Breakout"),
            (Tab::CopulaHeatmap, "Copula Heatmap"),
            (Tab::CorrelationNetworkAdv, "Corr Network Adv"),
        ],
        TabCategory::Comparison => &[
            (Tab::MultiCompare, "Multi-Compare"),
        ],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimeRange { D1, W1, M1, M3, M6, Y1, Y5 }

impl TimeRange {
    fn label(&self) -> &'static str {
        match self {
            TimeRange::D1 => "1D", TimeRange::W1 => "1W", TimeRange::M1 => "1M",
            TimeRange::M3 => "3M", TimeRange::M6 => "6M", TimeRange::Y1 => "1Y", TimeRange::Y5 => "5Y",
        }
    }

    fn to_days(&self) -> i64 {
        match self {
            TimeRange::D1 => 1, TimeRange::W1 => 7, TimeRange::M1 => 30,
            TimeRange::M3 => 90, TimeRange::M6 => 180, TimeRange::Y1 => 365, TimeRange::Y5 => 1825,
        }
    }

    fn to_interval(&self) -> Interval {
        match self {
            TimeRange::D1 => Interval::Min5, TimeRange::W1 => Interval::Min15,
            TimeRange::M1 => Interval::Hour1, TimeRange::M3 => Interval::Day1,
            TimeRange::M6 => Interval::Day1, TimeRange::Y1 => Interval::Day1,
            TimeRange::Y5 => Interval::Week1,
        }
    }
}

#[derive(Debug, Clone)]
enum AppMessage {
    DataReady(OhlcvSeries),
    FetchError(String),
    QuoteReady(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Prefs {
    live: bool,
    symbol: String,
    range: String,
    theme: String,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            live: true,
            symbol: "RELIANCE.NS".to_string(),
            range: "1y".to_string(),
            theme: "dark".to_string(),
        }
    }
}

impl Prefs {
    fn prefs_path() -> std::path::PathBuf {
        std::path::PathBuf::from("./data/prefs.json")
    }

    fn load() -> Self {
        let path = Self::prefs_path();
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(prefs) = serde_json::from_str::<Prefs>(&content) {
                return prefs;
            }
        }
        Self::default()
    }

    fn save(&self) {
        let path = Self::prefs_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&path, json);
        }
    }
}

struct Dataset {
    candles: OhlcvSeries,
    delta_series: OhlcvSeries,
    equity: Vec<f64>,
    corr: Vec<(String, Vec<f64>)>,
    smiles: Vec<(String, Vec<f64>, Vec<f64>)>,
    portfolios: Vec<bt_viz::efficient_frontier::Portfolio>,
    returns: Vec<f64>,
    treemap_nodes: Vec<bt_viz::sector_treemap::TreemapNode>,
    yield_curves: Vec<(String, Vec<f64>, Vec<f64>)>,
    seasonality: Vec<Vec<f64>>,
}

impl Dataset {
    fn generate(seed: u64) -> Self {
        let candles = synthetic_ohlcv("RELIANCE.NS", 180, seed, 2500.0);
        let delta_series = synthetic_ohlcv("NIFTY50", 200, seed + 1, 22000.0);
        let equity = synthetic_ohlcv("PORTFOLIO", 300, seed + 2, 1_000_000.0).closes();
        let corr = synthetic_correlated_returns(
            &["RELIANCE", "TCS", "INFY", "HDFCBANK", "ICICIBANK", "ITC"], 250, seed + 3,
        );
        let smiles = vec![
            ("7D".to_string(), 0.28, 0.10),
            ("30D".to_string(), 0.22, 0.07),
            ("90D".to_string(), 0.19, 0.05),
        ]
        .into_iter()
        .map(|(label, atm, skew)| {
            let c = bt_viz::vol_smile::synthetic_smile(&label, atm, skew, 14);
            (label, c.moneyness, c.iv)
        })
        .collect();

        let expected_returns = vec![0.09, 0.14, 0.11, 0.16, 0.07, 0.10];
        let cov: Vec<Vec<f64>> = (0..6)
            .map(|i| (0..6).map(|j| if i == j { 0.03 + i as f64 * 0.006 } else { 0.006 }).collect())
            .collect();
        let portfolios = bt_viz::efficient_frontier::simulate_portfolios(&expected_returns, &cov, 0.065, 1500, seed + 4);
        let returns = synthetic_ohlcv("BTC-USD", 400, seed + 5, 60000.0).returns();

        let treemap_nodes = vec![
            bt_viz::sector_treemap::TreemapNode::new("Reliance", 1_800_000.0, 1.2),
            bt_viz::sector_treemap::TreemapNode::new("TCS", 1_400_000.0, -0.8),
            bt_viz::sector_treemap::TreemapNode::new("HDFC Bank", 1_100_000.0, 0.5),
            bt_viz::sector_treemap::TreemapNode::new("Infosys", 700_000.0, -1.5),
            bt_viz::sector_treemap::TreemapNode::new("ICICI Bank", 650_000.0, 2.1),
            bt_viz::sector_treemap::TreemapNode::new("ITC", 500_000.0, 0.1),
            bt_viz::sector_treemap::TreemapNode::new("L&T", 420_000.0, 0.9),
            bt_viz::sector_treemap::TreemapNode::new("Bharti Airtel", 610_000.0, -0.3),
        ];

        let tenors = [0.25, 0.5, 1.0, 2.0, 3.0, 5.0, 10.0, 30.0];
        let yield_curves = vec![
            ("2026-06-01", 6.8, -1.2, 0.4),
            ("2026-07-15", 6.6, -1.0, 0.5),
            ("2026-09-20", 6.5, -0.8, 0.6),
        ]
        .into_iter()
        .map(|(label, level, slope, curvature)| {
            let c = bt_viz::yield_curve::synthetic_curve(label, level, slope, curvature, &tenors);
            (label.to_string(), c.tenors, c.yields)
        })
        .collect();

        let seasonality = bt_viz::seasonality_polar::synthetic_seasonality(seed + 6).data;

        Self {
            candles, delta_series, equity, corr, smiles, portfolios, returns,
            treemap_nodes, yield_curves, seasonality,
        }
    }
}

#[derive(Debug, Clone)]
struct MarketQuote {
    symbol: String,
    name: String,
    price: f64,
    change: f64,
    change_pct: f64,
    volume: u64,
    market_cap: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MarketSortCol {
    Symbol, Name, Price, Change, ChangePct, Volume, MarketCap,
}

/// Sample market data for the MarketWatch panel.
///
/// In production this would be populated by calling
/// `bt_data::DataService::fetch_quote()` for each symbol; fetching 78+
/// companies individually is too slow for a 30s refresh cycle, so this
/// generates realistic-looking data for demonstration purposes.
fn sample_market_data() -> Vec<MarketQuote> {
    let symbols = [
        ("RELIANCE.NS", "Reliance Industries", 2456.75),
        ("TCS.NS", "TCS", 3890.50),
        ("INFY.NS", "Infosys", 1567.30),
        ("HDFCBANK.NS", "HDFC Bank", 1678.90),
        ("ICICIBANK.NS", "ICICI Bank", 945.60),
        ("SBIN.NS", "State Bank of India", 623.45),
        ("BHARTIARTL.NS", "Bharti Airtel", 1567.80),
        ("ITC.NS", "ITC", 456.70),
        ("LT.NS", "Larsen & Toubro", 3456.20),
        ("AXISBANK.NS", "Axis Bank", 1123.40),
        ("KOTAKBANK.NS", "Kotak Mahindra Bank", 1890.55),
        ("MARUTI.NS", "Maruti Suzuki", 12345.60),
        ("ASIANPAINT.NS", "Asian Paints", 3456.70),
        ("BAJFINANCE.NS", "Bajaj Finance", 7890.30),
        ("SUNPHARMA.NS", "Sun Pharma", 1234.50),
        ("TITAN.NS", "Titan Company", 3456.80),
        ("ULTRACEMCO.NS", "UltraTech Cement", 9876.40),
        ("NESTLEIND.NS", "Nestle India", 23456.70),
        ("WIPRO.NS", "Wipro", 456.30),
        ("HCLTECH.NS", "HCL Technologies", 1789.60),
        ("TATAMOTORS.NS", "Tata Motors", 789.40),
        ("TATASTEEL.NS", "Tata Steel", 123.50),
        ("AAPL", "Apple", 189.30),
        ("MSFT", "Microsoft", 378.90),
        ("GOOGL", "Google", 145.60),
        ("AMZN", "Amazon", 178.30),
        ("TSLA", "Tesla", 245.60),
        ("NVDA", "NVIDIA", 456.70),
        ("META", "Meta Platforms", 345.80),
        ("NFLX", "Netflix", 456.90),
        ("BTC-USD", "Bitcoin", 43567.80),
        ("ETH-USD", "Ethereum", 2345.60),
        ("SOL-USD", "Solana", 98.70),
    ];

    symbols.iter().enumerate().map(|(i, (sym, name, price))| {
        let change = (i as f64 * 13.7 - 200.0).sin() * price * 0.02;
        let change_pct = change / price * 100.0;
        let volume = 1_000_000.0 + (i as f64 * 12345.0);
        let market_cap = price * volume * 0.1;
        MarketQuote {
            symbol: sym.to_string(),
            name: name.to_string(),
            price: *price,
            change,
            change_pct,
            volume: volume as u64,
            market_cap,
        }
    }).collect()
}

struct BharatApp {
    tab: Tab,
    dark: bool,
    data: Dataset,
    seed_counter: u64,
    selected_company: String,
    company_search: String,
    time_range: TimeRange,
    live: bool,
    last_fetch: Option<Instant>,
    fetch_in_flight: bool,
    status_source: String,
    status_last_update: String,
    status_latency_ms: u64,
    tx: Sender<AppMessage>,
    rx: Receiver<AppMessage>,
    runtime: tokio::runtime::Runtime,
    error_toast: Option<(String, Instant)>,
    warning_banner: Option<String>,
    warning_until: Option<Instant>,
    chart_id: Option<egui::Id>,
    scroll_to_chart: bool,
    prefs_dirty: bool,
    market_quotes: Vec<MarketQuote>,
    market_search: String,
    market_sort_col: MarketSortCol,
    market_sort_asc: bool,
    market_last_refresh: Option<Instant>,
    compare_symbols: Vec<String>,
    compare_search: String,
    show_candle_arrows: std::cell::Cell<bool>,
}

impl BharatApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = channel();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");

        let prefs = Prefs::load();

        let time_range = match prefs.range.as_str() {
            "1d" => TimeRange::D1, "1w" => TimeRange::W1, "1m" => TimeRange::M1,
            "3m" => TimeRange::M3, "6m" => TimeRange::M6, "5y" => TimeRange::Y5,
            _ => TimeRange::Y1,
        };

        let mut app = Self {
            tab: Tab::Candlestick,
            dark: prefs.theme == "dark",
            data: Dataset::generate(42),
            seed_counter: 42,
            selected_company: prefs.symbol.clone(),
            company_search: String::new(),
            time_range,
            live: prefs.live,
            last_fetch: None,
            fetch_in_flight: false,
            status_source: "Synthetic".to_string(),
            status_last_update: "—".to_string(),
            status_latency_ms: 0,
            tx, rx, runtime,
            error_toast: None,
            warning_banner: None,
            warning_until: None,
            chart_id: None,
            scroll_to_chart: false,
            prefs_dirty: false,
            market_quotes: sample_market_data(),
            market_search: String::new(),
            market_sort_col: MarketSortCol::MarketCap,
            market_sort_asc: false,
            market_last_refresh: Some(Instant::now()),
            compare_symbols: vec![
                "RELIANCE.NS".to_string(),
                "TCS.NS".to_string(),
                "INFY.NS".to_string(),
            ],
            compare_search: String::new(),
            show_candle_arrows: std::cell::Cell::new(true),
        };

        app.trigger_fetch();
        app
    }

    fn save_prefs(&mut self) {
        if self.prefs_dirty {
            let range_str = match self.time_range {
                TimeRange::D1 => "1d", TimeRange::W1 => "1w", TimeRange::M1 => "1m",
                TimeRange::M3 => "3m", TimeRange::M6 => "6m", TimeRange::Y1 => "1y", TimeRange::Y5 => "5y",
            };
            let prefs = Prefs {
                live: self.live,
                symbol: self.selected_company.clone(),
                range: range_str.to_string(),
                theme: if self.dark { "dark".to_string() } else { "light".to_string() },
            };
            prefs.save();
            self.prefs_dirty = false;
        }
    }

    fn trigger_fetch(&mut self) {
        if self.fetch_in_flight { return; }
        self.fetch_in_flight = true;
        let symbol = self.selected_company.clone();
        let interval = self.time_range.to_interval();
        let days = self.time_range.to_days();
        let tx = self.tx.clone();

        self.runtime.spawn(async move {
            let start = Instant::now();
            let service = match DataService::new() {
                Ok(s) => s,
                Err(e) => {
                    let _ = tx.send(AppMessage::FetchError(format!("Failed to initialize data service: {}", e)));
                    return;
                }
            };
            let end = Utc::now();
            let start_date = end - ChronoDuration::days(days);
            match service.fetch_ohlcv(&symbol, interval, start_date, end).await {
                Ok(series) => { let _ = tx.send(AppMessage::DataReady(series)); }
                Err(e) => { let _ = tx.send(AppMessage::FetchError(format!("Failed to fetch {}: {}", symbol, e))); }
            }
            let _ = start.elapsed();
        });
    }

    fn drain_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AppMessage::DataReady(series) => {
                    self.data.candles = series;
                    self.fetch_in_flight = false;
                    self.last_fetch = Some(Instant::now());
                    self.status_source = format!("Yahoo ({})", self.selected_company);
                    self.status_last_update = Utc::now().format("%H:%M:%S").to_string();
                    self.status_latency_ms = 0;
                    self.warning_banner = None;
                }
                AppMessage::FetchError(err) => {
                    self.fetch_in_flight = false;
                    self.error_toast = Some((err.clone(), Instant::now()));
                    self.warning_banner = Some("Using synthetic fallback data".to_string());
                    self.warning_until = Some(Instant::now() + Duration::from_secs(30));
                    self.status_source = "Synthetic (fallback)".to_string();
                    self.status_last_update = Utc::now().format("%H:%M:%S").to_string();
                    let days = match self.time_range.to_interval() {
                        Interval::Min5 | Interval::Min15 | Interval::Hour1 => 50,
                        Interval::Day1 => self.time_range.to_days() as usize,
                        Interval::Week1 | Interval::Month1 => 200,
                        _ => 100,
                    };
                    self.data.candles = synthetic_ohlcv(&self.selected_company, days, 42, 100.0);
                }
                AppMessage::QuoteReady(_) => {}
            }
        }
    }

    fn header(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(APP_NAME).color(AMBER).strong().size(18.0_f32));
                ui.separator();
                let display_text = if self.company_search.is_empty() {
                    self.selected_company.clone()
                } else {
                    self.company_search.clone()
                };
                let mut changed = false;
                egui::ComboBox::from_label("Company")
                    .selected_text(display_text)
                    .width(220.0_f32)
                    .show_ui(ui, |ui| {
                        ui.text_edit_singleline(&mut self.company_search);
                        let query = self.company_search.to_lowercase();
                        for (name, ticker, exchange) in COMPANY_LIST {
                            let matches = query.is_empty()
                                || name.to_lowercase().contains(&query)
                                || ticker.to_lowercase().contains(&query);
                            if matches {
                                let label = format!("{} ({}) — {}", name, ticker, exchange);
                                if ui.selectable_label(self.selected_company == *ticker, label).clicked() {
                                    self.selected_company = ticker.to_string();
                                    self.company_search.clear();
                                    changed = true;
                                }
                            }
                        }
                    });
                if changed { self.prefs_dirty = true; self.trigger_fetch(); }
                ui.separator();
                for range in [TimeRange::D1, TimeRange::W1, TimeRange::M1, TimeRange::M3, TimeRange::M6, TimeRange::Y1, TimeRange::Y5] {
                    let selected = self.time_range == range;
                    if ui.selectable_label(selected, range.label()).clicked() {
                        self.time_range = range;
                        self.prefs_dirty = true;
                        self.trigger_fetch();
                    }
                }
                ui.separator();
                let live_text = if self.live { "Live *" } else { "Off o" };
                let live_color = if self.live { PROFIT } else { Color32::GRAY };
                if ui.selectable_label(self.live, RichText::new(live_text).color(live_color)).clicked() {
                    self.live = !self.live;
                    self.prefs_dirty = true;
                    if self.live { self.trigger_fetch(); }
                }
                ui.separator();
                if ui.button(if self.dark { "\u{1F319}" } else { "\u{2600}" }).clicked() {
                    self.dark = !self.dark;
                    self.prefs_dirty = true;
                }
                if ui.button("\u{27F3}").clicked() { self.trigger_fetch(); }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("Made by {AUTHOR}")).color(AMBER));
                });
            });
        });
    }

    fn tab_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            egui::ScrollArea::horizontal().show(ui, |ui| {
                ui.horizontal(|ui| {
                    for cat in CATEGORIES {
                        let tabs = tabs_in_category(cat);
                        ui.group(|ui| {
                            ui.label(RichText::new(cat.label()).color(cat.color()).strong().size(11.0_f32));
                            ui.horizontal(|ui| {
                                for (tab, label) in tabs.iter() {
                                    let selected = self.tab == *tab;
                                    let text = if selected {
                                        RichText::new(format!("[{label}]")).color(cat.color()).strong()
                                    } else {
                                        RichText::new(*label).size(11.0_f32)
                                    };
                                    if ui.selectable_label(selected, text).clicked() {
                                        self.tab = *tab;
                                        self.chart_id = None;
                                        self.scroll_to_chart = true;
                                    }
                                }
                            });
                        });
                        ui.separator();
                    }
                });
            });
        });
    }

    fn status_bar(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.colored_label(if self.live { PROFIT } else { Color32::GRAY }, "\u{25CF}");
                ui.label(if self.live { "Live" } else { "Off" });
                ui.separator();
                ui.label(format!("Source: {}", self.status_source));
                ui.separator();
                ui.label(format!("Last: {}", self.status_last_update));
                ui.separator();
                ui.label(format!("Latency: {}ms", self.status_latency_ms));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("Made by {AUTHOR}")).color(AMBER));
                });
            });
        });
    }

    fn error_toast(&mut self, ctx: &egui::Context) {
        if let Some((msg, time)) = self.error_toast.clone() {
            if time.elapsed() < Duration::from_secs(8) {
                let screen = ctx.screen_rect();
                let toast_width = 400.0_f32;
                let toast_height = 60.0_f32;
                let pos = egui::Pos2::new(screen.right() - toast_width - 20.0_f32, screen.top() + 60.0_f32);
                egui::Window::new("error_toast")
                    .title_bar(false)
                    .fixed_pos(pos)
                    .fixed_size([toast_width, toast_height])
                    .collapsible(false)
                    .show(ctx, |ui| {
                        ui.colored_label(LOSS, RichText::new("Error").strong());
                        ui.label(msg.clone());
                        if ui.button("Dismiss").clicked() { self.error_toast = None; }
                    });
            } else {
                self.error_toast = None;
            }
        }
    }

    fn warning_banner(&mut self, ctx: &egui::Context) {
        let expired = match self.warning_until {
            Some(until) => Instant::now() >= until,
            None => true,
        };
        if expired {
            self.warning_banner = None;
            self.warning_until = None;
            return;
        }
        if let Some(msg) = &self.warning_banner {
            egui::TopBottomPanel::top("warning").show(ctx, |ui| {
                ui.colored_label(Color32::YELLOW, format!("⚠ {msg}"));
            });
        }
    }

    fn body(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.fetch_in_flight {
                ui.centered_and_justified(|ui| {
                    ui.spinner();
                    ui.label("Loading...");
                });
                return;
            }

            let chart_id = Id::new("chart_area").with(self.tab);
            self.chart_id = Some(chart_id);

            egui::ScrollArea::vertical().show(ui, |ui| {
                if self.scroll_to_chart {
                    self.scroll_to_chart = false;
                    let anchor_rect = egui::Rect::from_min_size(ui.next_widget_position(), Vec2::new(1.0, 1.0));
                    let anchor = ui.interact(anchor_rect, egui::Id::new("chart_anchor"), egui::Sense::hover());
                    anchor.scroll_to_me(Some(egui::Align::Center));
                }
                ui.allocate_space(Vec2::new(0.0, 0.0));
                self.dispatch_tab(ui);
            });
        });
    }

    fn dispatch_tab(&mut self, ui: &mut egui::Ui) {
        match self.tab {
            Tab::Candlestick => self.draw_candlestick(ui),
            Tab::HeikinAshi => self.draw_heikin_ashi(ui),
            Tab::Renko => self.draw_renko(ui),
            Tab::Kagi => self.draw_kagi(ui),
            Tab::PointFigure => self.draw_point_figure(ui),
            Tab::Candlestick3D => self.draw_candlestick_3d(ui),
            Tab::CandlestickMA => self.draw_candlestick_ma(ui),
            Tab::CandlestickBollinger => self.draw_candlestick_bollinger(ui),
            Tab::CandlestickRSI => self.draw_candlestick_rsi(ui),
            Tab::CandlestickMACD => self.draw_candlestick_macd(ui),
            Tab::VolumeProfile => self.draw_volume_profile(ui),
            Tab::Footprint => self.draw_footprint(ui),
            Tab::OrderBookHeatmap => self.draw_orderbook_heatmap(ui),
            Tab::CumulativeDelta => self.draw_cumulative_delta(ui),
            Tab::MarketProfile => self.draw_market_profile(ui),
            Tab::VolumeClock => self.draw_volume_clock(ui),
            Tab::TickTape => self.draw_tick_tape(ui),
            Tab::DeltaDivergence => self.draw_delta_divergence(ui),
            Tab::RSI => self.draw_rsi(ui),
            Tab::MACD => self.draw_macd(ui),
            Tab::Stochastic => self.draw_stochastic(ui),
            Tab::ATR => self.draw_atr(ui),
            Tab::OBV => self.draw_obv(ui),
            Tab::VWAP => self.draw_vwap(ui),
            Tab::Bollinger => self.draw_bollinger(ui),
            Tab::BBWidth => self.draw_bb_width(ui),
            Tab::ADX => self.draw_adx(ui),
            Tab::CCI => self.draw_cci(ui),
            Tab::WilliamsR => self.draw_williams_r(ui),
            Tab::ROC => self.draw_roc(ui),
            Tab::CMF => self.draw_cmf(ui),
            Tab::Ichimoku => self.draw_ichimoku(ui),
            Tab::Keltner => self.draw_keltner(ui),
            Tab::Donchian => self.draw_donchian(ui),
            Tab::Drawdown => self.draw_drawdown(ui),
            Tab::Correlation => self.draw_correlation(ui),
            Tab::VolSmile => self.draw_vol_smile(ui),
            Tab::EffFrontier => self.draw_efficient_frontier(ui),
            Tab::RollingSharpe => self.draw_rolling_sharpe(ui),
            Tab::RollingSortino => self.draw_rolling_sortino(ui),
            Tab::BetaAlpha => self.draw_beta_alpha(ui),
            Tab::RollingMaxDD => self.draw_rolling_max_dd(ui),
            Tab::VaRBacktest => self.draw_var_backtest(ui),
            Tab::MonteCarlo => self.draw_monte_carlo(ui),
            Tab::VolSmileOpt => self.draw_vol_smile_opt(ui),
            Tab::IVSurface => self.draw_iv_surface(ui),
            Tab::TermStructure => self.draw_term_structure(ui),
            Tab::GreeksHeatmap => self.draw_greeks_heatmap(ui),
            Tab::VIXTerm => self.draw_vix_term(ui),
            Tab::VolCone => self.draw_vol_cone(ui),
            Tab::OptionPayoff => self.draw_option_payoff(ui),
            Tab::SkewEvolution => self.draw_skew_evolution(ui),
            Tab::GammaExposure => self.draw_gamma_exposure(ui),
            Tab::PutCallRatio => self.draw_put_call_ratio(ui),
            Tab::IVRank => self.draw_iv_rank(ui),
            Tab::SharpeSurface => self.draw_sharpe_surface(ui),
            Tab::AcfPacf => self.draw_acf_pacf(ui),
            Tab::Hurst => self.draw_hurst(ui),
            Tab::Wavelet => self.draw_wavelet(ui),
            Tab::Kalman => self.draw_kalman(ui),
            Tab::MarkovRegime => self.draw_markov_regime(ui),
            Tab::Copula3D => self.draw_copula_3d(ui),
            Tab::QQPlot => self.draw_qq_plot(ui),
            Tab::ReturnDist => self.draw_return_dist(ui),
            Tab::RollingMoments => self.draw_rolling_moments(ui),
            Tab::NiftyTreemap => self.draw_treemap(ui),
            Tab::SensexHeatmap => self.draw_heatmap(ui),
            Tab::FII_DIIFlow => self.draw_fii_dii_flow(ui),
            Tab::SectorPerf => self.draw_sector_perf(ui),
            Tab::YieldCurve => self.draw_yield_curve(ui),
            Tab::USDINR => self.draw_usdinr(ui),
            Tab::MonsoonAgri => self.draw_monsoon_agri(ui),
            Tab::Seasonality => self.draw_seasonality(ui),
            Tab::WorldIndices => self.draw_world_indices(ui),
            Tab::TickerTape => self.draw_ticker_tape(ui),
            Tab::CurrencyMatrix => self.draw_currency_matrix(ui),
            Tab::SectorWheel => self.draw_sector_wheel(ui),
            Tab::EarningsCalendar => self.draw_earnings_calendar(ui),
            Tab::EconCalendar => self.draw_econ_calendar(ui),
            Tab::CorrelationNetwork => self.draw_correlation_network(ui),
            Tab::ReturnHeatmap => self.draw_return_heatmap(ui),
            Tab::MultiIndicator => self.draw_multi_indicator(ui),
            Tab::MultiTimeframe => self.draw_multi_timeframe(ui),
            Tab::MACDDivergence => self.draw_macd_divergence(ui),
            Tab::BollingerBreakout => self.draw_bollinger_breakout(ui),
            Tab::VolumeWeightedScatter => self.draw_volume_weighted_scatter(ui),
            Tab::PriceMomentum => self.draw_price_momentum(ui),
            Tab::DrawdownRecovery => self.draw_drawdown_recovery(ui),
            Tab::RollingCorrelation => self.draw_rolling_correlation(ui),
            Tab::TickTapeAdv => self.draw_tick_tape_adv(ui),
            Tab::SeasonalityAdv => self.draw_seasonality_adv(ui),
            Tab::ParabolicSAR => self.draw_parabolic_sar(ui),
            Tab::MACDHistogram => self.draw_macd_histogram(ui),
            Tab::RSIHeatmap => self.draw_rsi_heatmap(ui),
            Tab::IchimokuEMA => self.draw_ichimoku_ema(ui),
            Tab::KeltnerBreakout => self.draw_keltner_breakout(ui),
            Tab::DonchianBreakout => self.draw_donchian_breakout(ui),
            Tab::CopulaHeatmap => self.draw_copula_heatmap(ui),
            Tab::CorrelationNetworkAdv => self.draw_correlation_network_adv(ui),
            Tab::MarketWatch => self.draw_market_watch(ui),
            Tab::FOChain => self.draw_fo_chain(ui),
            Tab::IVSurfaceIndia => self.draw_iv_surface_india(ui),
            Tab::OIHeatmap => self.draw_oi_heatmap(ui),
            Tab::GSec => self.draw_gsec(ui),
            Tab::MoneyMarket => self.draw_money_market(ui),
            Tab::RBIPolicy => self.draw_rbi_policy(ui),
            Tab::MacroIndia => self.draw_macro_india(ui),
            Tab::CommoditiesIndia => self.draw_commodities_india(ui),
            Tab::USDINRCurve => self.draw_usdinr_curve(ui),
            Tab::YieldIndia => self.draw_yield_india(ui),
            Tab::MFAnalytics => self.draw_mf_analytics(ui),
            Tab::FPIFII => self.draw_fpi_fii(ui),
            Tab::CreditRatings => self.draw_credit_ratings(ui),
            Tab::BankingIndia => self.draw_banking_india(ui),
            Tab::CorpActions => self.draw_corp_actions(ui),
            Tab::IPOPipeline => self.draw_ipo_pipeline(ui),
            Tab::IndiaBreadth => self.draw_india_breadth(ui),
            Tab::SectorResearch => self.draw_sector_research(ui),
            Tab::IndiaNews => self.draw_india_news(ui),
            Tab::Regulatory => self.draw_regulatory(ui),
            Tab::GSTBudget => self.draw_gst_budget(ui),
            Tab::IndiaPortfolio => self.draw_india_portfolio(ui),
            Tab::AlgoFeed => self.draw_algo_feed(ui),
            Tab::AIResearch => self.draw_ai_research(ui),
            Tab::IndiaDashboard => self.draw_india_dashboard(ui),
            Tab::MultiCompare => self.draw_multi_compare(ui),
        }
    }

    fn draw_candlestick(&self, ui: &mut egui::Ui) {
        let candles = &self.data.candles;
        ui.label(RichText::new(format!("GP — Candlestick — {}", candles.symbol)).strong());
        ui.horizontal(|ui| {
            ui.label(RichText::new("Trend arrows").small());
            let mut arrows = self.show_candle_arrows.get();
            if ui.checkbox(&mut arrows, "").changed() {
                self.show_candle_arrows.set(arrows);
            }
            ui.colored_label(PROFIT, "\u{25B2} up = close > open");
            ui.colored_label(LOSS, "\u{25BC} down = close < open");
        });
        let (lo, range) = price_scale(&candles.candles);
        let min_body = range * MIN_BODY_FRAC;
        let arrow = range * ARROW_FRAC;
        let show_arrows = self.show_candle_arrows.get();
        let headroom = 3.0 * arrow;
        let hover_candle = std::cell::RefCell::new(None::<(f64, f64, f64, f64, f64)>);
        Plot::new("candlestick_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height() * 0.7_f32)
            .allow_scroll(true)
            .allow_drag(true)
            .include_y(lo - headroom)
            .include_y(lo + range + headroom)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                for c in &candles.candles {
                    draw_candle(plot_ui, c, BAR_HALF, min_body);
                    if show_arrows {
                        draw_trend_arrow(plot_ui, c, BAR_HALF, arrow);
                    }
                }
                if let Some(hover_pos) = plot_ui.pointer_coordinate() {
                    let t_min = candles.candles.first().map(|c| c.t).unwrap_or(0.0);
                    let t_max = candles.candles.last().map(|c| c.t).unwrap_or(0.0);
                    let bar_width = if candles.candles.len() > 1 {
                        (t_max - t_min) / (candles.candles.len() - 1) as f64
                    } else {
                        DAY_SECS
                    };
                    if bar_width > 0.0 {
                        let idx = (((hover_pos.x - t_min) / bar_width).round() as isize)
                            .clamp(0, candles.candles.len() as isize - 1)
                            as usize;
                        let c = &candles.candles[idx];
                        *hover_candle.borrow_mut() =
                            Some((c.open, c.high, c.low, c.close, c.volume));
                    }
                }
            });
        if let Some((o, h, l, c, v)) = hover_candle.borrow().as_ref() {
            egui::show_tooltip_at_pointer(ui.ctx(), egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("candle_tooltip_layer")), egui::Id::new("candle_tooltip"), |ui: &mut egui::Ui| {
                ui.label(format!("O: {:.2} H: {:.2} L: {:.2} C: {:.2} V: {:.0}", o, h, l, c, v));
            });
        }
        Plot::new("candlestick_volume")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let bars: Vec<Bar> = candles.candles.iter()
                    .map(|c| {
                        let color = if c.is_bullish() { PROFIT } else { LOSS };
                        Bar::new(c.t, c.volume).width(BAR_WIDTH).fill(color)
                    }).collect();
                plot_ui.bar_chart(BarChart::new(bars));
            });
    }

    fn draw_heikin_ashi(&self, ui: &mut egui::Ui) {
        let candles = &self.data.candles;
        let ha_series = heikin_ashi(&candles.candles);
        ui.label(RichText::new(format!("GP (HA) — Heikin-Ashi — {}", candles.symbol)).strong());
        let min_body = price_scale(&candles.candles).1 * MIN_BODY_FRAC;
        Plot::new("ha_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                for ha in &ha_series {
                    let color = if ha.is_bullish() { PROFIT } else { LOSS };
                    plot_ui.line(
                        Line::new(PlotPoints::from(vec![[ha.t, ha.low], [ha.t, ha.high]]))
                            .color(color)
                            .width(1.0_f32),
                    );
                    let (top, bottom) = candle_body(ha, min_body);
                    plot_ui.polygon(
                        egui_plot::Polygon::new(PlotPoints::from(vec![
                            [ha.t - BAR_HALF, bottom], [ha.t + BAR_HALF, bottom],
                            [ha.t + BAR_HALF, top], [ha.t - BAR_HALF, top],
                        ]))
                        .fill_color(color).stroke(Stroke::new(1.0_f32, color)),
                    );
                }
            });
    }

    fn draw_renko(&self, ui: &mut egui::Ui) {
        let candles = &self.data.candles;
        ui.label(RichText::new(format!("GP (Renko) — Renko — {}", candles.symbol)).strong());
        Plot::new("renko_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                for w in candles.candles.windows(2) {
                    let prev = &w[0]; let curr = &w[1];
                    let color = if curr.close > prev.close { PROFIT } else { LOSS };
                    plot_ui.line(
                        Line::new(PlotPoints::from(vec![[curr.t, prev.close], [curr.t, curr.close]]))
                            .color(color).width(3.0_f32),
                    );
                }
            });
    }

    fn draw_kagi(&self, ui: &mut egui::Ui) {
        let candles = &self.data.candles;
        ui.label(RichText::new(format!("KAGI — Kagi — {}", candles.symbol)).strong());
        Plot::new("kagi_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                for w in candles.candles.windows(2) {
                    let prev = &w[0]; let curr = &w[1];
                    let color = if curr.close > prev.close { PROFIT } else { LOSS };
                    let thick = if curr.close > prev.close { 3.0_f32 } else { 1.0_f32 };
                    plot_ui.line(
                        Line::new(PlotPoints::from(vec![[curr.t, prev.close], [curr.t, curr.close]]))
                            .color(color).width(thick),
                    );
                }
            });
    }

    fn draw_point_figure(&self, ui: &mut egui::Ui) {
        let candles = &self.data.candles;
        ui.label(RichText::new(format!("P&F — Point & Figure — {}", candles.symbol)).strong());
        Plot::new("pf_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let box_size = 5.0_f64;
                let mut last_price = candles.candles[0].close;
                let mut col_x = candles.candles[0].t;
                let mut is_x = true;
                for c in &candles.candles {
                    if (c.close - last_price).abs() >= box_size {
                        is_x = c.close > last_price;
                        col_x = c.t;
                        last_price = c.close;
                    }
                    let color = if is_x { PROFIT } else { LOSS };
                    let y = if is_x { c.high } else { c.low };
                    plot_ui.points(
                        Points::new(PlotPoints::from(vec![[col_x, y]]))
                            .color(color).radius(4.0_f32)
                            .shape(if is_x { MarkerShape::Cross } else { MarkerShape::Circle }),
                    );
                }
            });
    }
    fn draw_candlestick_3d(&self, ui: &mut egui::Ui) {
        let candles = &self.data.candles;
        ui.label(RichText::new(format!("C3D — Candlestick 3D — {}", candles.symbol)).strong());
        let min_body = price_scale(&candles.candles).1 * MIN_BODY_FRAC;
        Plot::new("c3d_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                for c in &candles.candles {
                    draw_candle(plot_ui, c, BAR_HALF, min_body);
                }
            });
    }

    fn draw_candlestick_ma(&self, ui: &mut egui::Ui) {
        use bt_analytics::{ema, sma};
        let series = &self.data.candles;
        let sma20 = sma(series, 20);
        let sma50 = sma(series, 50);
        let ema200 = ema(series, 200);
        ui.label(RichText::new(format!("CMA — Candlestick + MA — {}", series.symbol)).strong());
        let min_body = price_scale(&series.candles).1 * MIN_BODY_FRAC;
        Plot::new("cma_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .legend(Legend::default())
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                for c in &series.candles {
                    draw_candle(plot_ui, c, BAR_HALF, min_body);
                }
                let sma20_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !sma20[i].is_nan() { Some([c.t, sma20[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(sma20_pts).color(INFO).width(2.0_f32).name("SMA20"));
                let sma50_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !sma50[i].is_nan() { Some([c.t, sma50[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(sma50_pts).color(AMBER).width(2.0_f32).name("SMA50"));
                let ema200_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !ema200[i].is_nan() { Some([c.t, ema200[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(ema200_pts).color(PURPLE).width(2.0_f32).name("EMA200"));
            });
    }

    fn draw_candlestick_bollinger(&self, ui: &mut egui::Ui) {
        use bt_analytics::bollinger;
        let series = &self.data.candles;
        let (mid, upper, lower) = bollinger(series, 20, 2.0);
        ui.label(RichText::new(format!("CBB — Candlestick + Bollinger — {}", series.symbol)).strong());
        let min_body = price_scale(&series.candles).1 * MIN_BODY_FRAC;
        Plot::new("cbb_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                for c in &series.candles {
                    draw_candle(plot_ui, c, BAR_HALF, min_body);
                }
                let mid_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !mid[i].is_nan() { Some([c.t, mid[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(mid_pts).color(AMBER).width(2.0_f32).name("SMA20"));
                let upper_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !upper[i].is_nan() { Some([c.t, upper[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(upper_pts).color(INFO).width(1.0_f32).name("Upper"));
                let lower_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !lower[i].is_nan() { Some([c.t, lower[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(lower_pts).color(INFO).width(1.0_f32).name("Lower"));
            });
    }

    fn draw_candlestick_rsi(&self, ui: &mut egui::Ui) {
        use bt_analytics::rsi;
        let series = &self.data.candles;
        let rsi_vals = rsi(series, 14);
        ui.label(RichText::new(format!("CRSI — Candlestick + RSI — {}", series.symbol)).strong());
        let min_body = price_scale(&series.candles).1 * MIN_BODY_FRAC;
        Plot::new("crsi_price")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height() * 0.65_f32)
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                for c in &series.candles {
                    draw_candle(plot_ui, c, BAR_HALF, min_body);
                }
            });
        Plot::new("crsi_rsi")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !rsi_vals[i].is_nan() { Some([c.t, rsi_vals[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(pts).color(INFO).width(2.0_f32));
                plot_ui.hline(egui_plot::HLine::new(70.0).color(LOSS));
                plot_ui.hline(egui_plot::HLine::new(30.0).color(PROFIT));
            });
    }

    fn draw_candlestick_macd(&self, ui: &mut egui::Ui) {
        use bt_analytics::macd;
        let series = &self.data.candles;
        let (macd_line, signal_line, _histogram) = macd(series);
        ui.label(RichText::new(format!("CMACD — Candlestick + MACD — {}", series.symbol)).strong());
        let min_body = price_scale(&series.candles).1 * MIN_BODY_FRAC;
        Plot::new("cmacd_price")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height() * 0.65_f32)
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                for c in &series.candles {
                    draw_candle(plot_ui, c, BAR_HALF, min_body);
                }
            });
        Plot::new("cmacd_macd")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !macd_line[i].is_nan() { Some([c.t, macd_line[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(pts).color(INFO).width(2.0_f32).name("MACD"));
                let sig_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !signal_line[i].is_nan() { Some([c.t, signal_line[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(sig_pts).color(AMBER).width(2.0_f32).name("Signal"));
            });
    }
    // ==================== ORDER FLOW ====================

    fn draw_volume_profile(&self, ui: &mut egui::Ui) {
        let candles = &self.data.candles;
        ui.label(RichText::new(format!("VP — Volume Profile — {}", candles.symbol)).strong());
        Plot::new("vp_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                for c in &candles.candles {
                    let color = if c.is_bullish() { PROFIT } else { LOSS };
                    plot_ui.bar_chart(BarChart::new(vec![Bar::new(c.t, c.volume).width(BAR_WIDTH).fill(color)]));
                }
            });
    }

    fn draw_footprint(&self, ui: &mut egui::Ui) {
        let candles = &self.data.candles;
        ui.label(RichText::new(format!("FP — Footprint — {}", candles.symbol)).strong());
        Plot::new("fp_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                for c in &candles.candles {
                    let range = (c.high - c.low).max(1e-9);
                    let bid_vol = c.volume * (1.0 - (c.close - c.low) / range);
                    let ask_vol = c.volume * ((c.close - c.low) / range);
                    plot_ui.bar_chart(BarChart::new(vec![Bar::new(c.t - 0.2 * DAY_SECS, bid_vol).width(0.35 * DAY_SECS).fill(LOSS.gamma_multiply(0.7))]));
                    plot_ui.bar_chart(BarChart::new(vec![Bar::new(c.t + 0.2 * DAY_SECS, ask_vol).width(0.35 * DAY_SECS).fill(PROFIT.gamma_multiply(0.7))]));
                }
            });
    }

    fn draw_orderbook_heatmap(&self, ui: &mut egui::Ui) {
        let candles = &self.data.candles;
        ui.label(RichText::new("OBH — Order Book Heatmap").strong());
        let avail = ui.available_size();
        let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let levels = 12;
        let cell_w = rect.width() / candles.candles.len() as f32;
        let cell_h = rect.height() / levels as f32;
        for (i, c) in candles.candles.iter().enumerate() {
            let mid = (c.high + c.low) / 2.0;
            let range = (c.high - c.low).max(1e-9);
            for lvl in 0..levels {
                let price = c.low + range * lvl as f64 / (levels - 1) as f64;
                let dist = (price - mid).abs() / range;
                let intensity = (1.0 - dist).clamp(0.0, 1.0);
                let color = if price >= mid {
                    Color32::from_rgb(lerp(10, PROFIT.r(), intensity), lerp(10, PROFIT.g(), intensity), lerp(10, PROFIT.b(), intensity))
                } else {
                    Color32::from_rgb(lerp(10, LOSS.r(), intensity), lerp(10, LOSS.g(), intensity), lerp(10, LOSS.b(), intensity))
                };
                let tile = egui::Rect::from_min_size(
                    egui::Pos2::new(rect.left() + i as f32 * cell_w, rect.top() + lvl as f32 * cell_h),
                    Vec2::new(cell_w - 1.0_f32, cell_h - 1.0_f32),
                );
                painter.rect_filled(tile, 0.0, color);
            }
        }
    }

    fn draw_cumulative_delta(&self, ui: &mut egui::Ui) {
        let candles = &self.data.candles;
        ui.label(RichText::new(format!("CD — Cumulative Delta — {}", candles.symbol)).strong());
        let (_per_bar, cumulative) = bt_viz::cumulative_delta::compute_deltas(candles);
        Plot::new("cd_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = candles.candles.iter().enumerate()
                    .map(|(i, c)| [c.t, cumulative[i]])
                    .collect();
                plot_ui.line(Line::new(pts).color(AMBER).width(2.0_f32));
            });
    }

    fn draw_market_profile(&self, ui: &mut egui::Ui) {
        let candles = &self.data.candles;
        ui.label(RichText::new(format!("MP — Market Profile — {}", candles.symbol)).strong());
        Plot::new("mp_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                for c in &candles.candles {
                    let color = if c.is_bullish() { PROFIT } else { LOSS };
                    plot_ui.line(Line::new(PlotPoints::from(vec![[c.t, c.low], [c.t, c.high]])).color(color).width(2.0_f32));
                }
            });
    }

    fn draw_volume_clock(&self, ui: &mut egui::Ui) {
        let candles = &self.data.candles;
        ui.label(RichText::new(format!("VC — Volume Clock — {}", candles.symbol)).strong());
        Plot::new("vc_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                for c in &candles.candles {
                    let color = if c.is_bullish() { PROFIT } else { LOSS };
                    plot_ui.bar_chart(BarChart::new(vec![Bar::new(c.t, c.volume).width(BAR_WIDTH).fill(color)]));
                }
            });
    }

    fn draw_tick_tape(&self, ui: &mut egui::Ui) {
        let candles = &self.data.candles;
        ui.label(RichText::new(format!("TT — Tick Tape — {}", candles.symbol)).strong());
        Plot::new("tt_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                for c in &candles.candles {
                    let color = if c.is_bullish() { PROFIT } else { LOSS };
                    plot_ui.points(Points::new(PlotPoints::from(vec![[c.t, c.close]])).color(color).radius(3.0_f32));
                }
            });
    }

    fn draw_delta_divergence(&self, ui: &mut egui::Ui) {
        let candles = &self.data.candles;
        ui.label(RichText::new(format!("DD — Delta Divergence — {}", candles.symbol)).strong());
        let (_per_bar, cumulative) = bt_viz::cumulative_delta::compute_deltas(candles);
        Plot::new("dd_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = candles.candles.iter().enumerate()
                    .map(|(i, c)| [c.t, cumulative[i]])
                    .collect();
                plot_ui.line(Line::new(pts).color(INFO).width(2.0_f32));
            });
    }
    // ==================== INDICATORS ====================

    fn draw_rsi(&self, ui: &mut egui::Ui) {
        use bt_analytics::rsi;
        let series = &self.data.candles;
        let rsi_vals = rsi(series, 14);
        ui.label(RichText::new(format!("RSI — Relative Strength Index — {}", series.symbol)).strong());
        Plot::new("rsi_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !rsi_vals[i].is_nan() { Some([c.t, rsi_vals[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(pts).color(INFO).width(2.0_f32));
                plot_ui.hline(egui_plot::HLine::new(70.0).color(LOSS));
                plot_ui.hline(egui_plot::HLine::new(30.0).color(PROFIT));
            });
    }

    fn draw_macd(&self, ui: &mut egui::Ui) {
        use bt_analytics::macd;
        let series = &self.data.candles;
        let (macd_line, signal_line, histogram) = macd(series);
        ui.label(RichText::new(format!("MACD — Moving Average Convergence Divergence — {}", series.symbol)).strong());
        Plot::new("macd_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height() * 0.5_f32)
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !macd_line[i].is_nan() { Some([c.t, macd_line[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(pts).color(INFO).width(2.0_f32).name("MACD"));
                let sig_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !signal_line[i].is_nan() { Some([c.t, signal_line[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(sig_pts).color(AMBER).width(2.0_f32).name("Signal"));
            });
        Plot::new("macd_hist")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let bars: Vec<Bar> = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| {
                        if !histogram[i].is_nan() {
                            let color = if histogram[i] >= 0.0 { PROFIT } else { LOSS };
                            Some(Bar::new(c.t, histogram[i]).width(0.5 * DAY_SECS).fill(color))
                        } else { None }
                    }).collect();
                plot_ui.bar_chart(BarChart::new(bars));
            });
    }

    fn draw_stochastic(&self, ui: &mut egui::Ui) {
        use bt_analytics::stochastic;
        let series = &self.data.candles;
        let (k, d) = stochastic(series, 14, 3);
        ui.label(RichText::new(format!("STOCH — Stochastic Oscillator — {}", series.symbol)).strong());
        Plot::new("stoch_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let k_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !k[i].is_nan() { Some([c.t, k[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(k_pts).color(AMBER).width(2.0_f32).name("%K"));
                let d_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !d[i].is_nan() { Some([c.t, d[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(d_pts).color(INFO).width(2.0_f32).name("%D"));
                plot_ui.hline(egui_plot::HLine::new(80.0).color(LOSS));
                plot_ui.hline(egui_plot::HLine::new(20.0).color(PROFIT));
            });
    }

    fn draw_atr(&self, ui: &mut egui::Ui) {
        use bt_analytics::atr;
        let series = &self.data.candles;
        let atr_vals = atr(series, 14);
        ui.label(RichText::new(format!("ATR — Average True Range — {}", series.symbol)).strong());
        Plot::new("atr_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !atr_vals[i].is_nan() { Some([c.t, atr_vals[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(pts).color(PROFIT).width(2.0_f32));
            });
    }

    fn draw_obv(&self, ui: &mut egui::Ui) {
        use bt_analytics::obv;
        let series = &self.data.candles;
        let obv_vals = obv(series);
        ui.label(RichText::new(format!("OBV — On-Balance Volume — {}", series.symbol)).strong());
        Plot::new("obv_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().enumerate()
                    .map(|(i, c)| [c.t, obv_vals[i]])
                    .collect();
                plot_ui.line(Line::new(pts).color(AMBER).width(2.0_f32));
            });
    }

    fn draw_vwap(&self, ui: &mut egui::Ui) {
        use bt_analytics::vwap;
        let series = &self.data.candles;
        let vwap_vals = vwap(series);
        ui.label(RichText::new(format!("VWAP — Volume Weighted Average Price — {}", series.symbol)).strong());
        Plot::new("vwap_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !vwap_vals[i].is_nan() { Some([c.t, vwap_vals[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(pts).color(INFO).width(2.0_f32).name("VWAP"));
            });
    }

    fn draw_bollinger(&self, ui: &mut egui::Ui) {
        use bt_analytics::bollinger;
        let series = &self.data.candles;
        let (mid, upper, lower) = bollinger(series, 20, 2.0);
        ui.label(RichText::new(format!("BOLL — Bollinger Bands — {}", series.symbol)).strong());
        Plot::new("bollinger_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let mid_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !mid[i].is_nan() { Some([c.t, mid[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(mid_pts).color(AMBER).width(2.0_f32).name("SMA20"));
                let upper_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !upper[i].is_nan() { Some([c.t, upper[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(upper_pts).color(INFO).width(1.0_f32).name("Upper"));
                let lower_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !lower[i].is_nan() { Some([c.t, lower[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(lower_pts).color(INFO).width(1.0_f32).name("Lower"));
            });
    }

    fn draw_bb_width(&self, ui: &mut egui::Ui) {
        use bt_analytics::bollinger;
        let series = &self.data.candles;
        let (_mid, upper, lower) = bollinger(series, 20, 2.0);
        ui.label(RichText::new(format!("BBW — Bollinger Band Width — {}", series.symbol)).strong());
        Plot::new("bbw_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| {
                        if !upper[i].is_nan() && !lower[i].is_nan() {
                            Some([c.t, upper[i] - lower[i]])
                        } else { None }
                    }).collect();
                plot_ui.line(Line::new(pts).color(PURPLE).width(2.0_f32));
            });
    }

    fn draw_adx(&self, ui: &mut egui::Ui) {
        use bt_analytics::adx;
        let series = &self.data.candles;
        let (adx_vals, _plus_di, _minus_di) = adx(series, 14);
        ui.label(RichText::new(format!("ADX — Average Directional Index — {}", series.symbol)).strong());
        Plot::new("adx_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !adx_vals[i].is_nan() { Some([c.t, adx_vals[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(pts).color(AMBER).width(2.0_f32));
                plot_ui.hline(egui_plot::HLine::new(25.0).color(Color32::GRAY));
            });
    }

    fn draw_cci(&self, ui: &mut egui::Ui) {
        use bt_analytics::cci;
        let series = &self.data.candles;
        let cci_vals = cci(series, 20);
        ui.label(RichText::new(format!("CCI — Commodity Channel Index — {}", series.symbol)).strong());
        Plot::new("cci_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !cci_vals[i].is_nan() { Some([c.t, cci_vals[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(pts).color(INFO).width(2.0_f32));
                plot_ui.hline(egui_plot::HLine::new(100.0).color(LOSS));
                plot_ui.hline(egui_plot::HLine::new(-100.0).color(PROFIT));
            });
    }

    fn draw_williams_r(&self, ui: &mut egui::Ui) {
        use bt_analytics::williams_r;
        let series = &self.data.candles;
        let wr_vals = williams_r(series, 14);
        ui.label(RichText::new(format!("W%R — Williams %R — {}", series.symbol)).strong());
        Plot::new("wr_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !wr_vals[i].is_nan() { Some([c.t, wr_vals[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(pts).color(PURPLE).width(2.0_f32));
                plot_ui.hline(egui_plot::HLine::new(-20.0).color(LOSS));
                plot_ui.hline(egui_plot::HLine::new(-80.0).color(PROFIT));
            });
    }

    fn draw_roc(&self, ui: &mut egui::Ui) {
        use bt_analytics::roc;
        let series = &self.data.candles;
        let roc_vals = roc(series, 12);
        ui.label(RichText::new(format!("ROC — Rate of Change — {}", series.symbol)).strong());
        Plot::new("roc_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !roc_vals[i].is_nan() { Some([c.t, roc_vals[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(pts).color(AMBER).width(2.0_f32));
                plot_ui.hline(egui_plot::HLine::new(0.0).color(Color32::GRAY));
            });
    }

    fn draw_cmf(&self, ui: &mut egui::Ui) {
        use bt_analytics::cmf;
        let series = &self.data.candles;
        let cmf_vals = cmf(series, 20);
        ui.label(RichText::new(format!("CMF — Chaikin Money Flow — {}", series.symbol)).strong());
        Plot::new("cmf_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !cmf_vals[i].is_nan() { Some([c.t, cmf_vals[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(pts).color(INFO).width(2.0_f32));
                plot_ui.hline(egui_plot::HLine::new(0.0).color(Color32::GRAY));
            });
    }

    fn draw_ichimoku(&self, ui: &mut egui::Ui) {
        let series = &self.data.candles;
        ui.label(RichText::new(format!("ICH — Ichimoku Cloud — {}", series.symbol)).strong());
        Plot::new("ich_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().map(|c| [c.t, c.close]).collect();
                plot_ui.line(Line::new(pts).color(AMBER).width(1.5_f32).name("Price"));
            });
    }

    fn draw_keltner(&self, ui: &mut egui::Ui) {
        use bt_analytics::atr;
        use bt_analytics::bollinger;
        let series = &self.data.candles;
        let (mid, _upper, _lower) = bollinger(series, 20, 2.0);
        let atr_vals = atr(series, 14);
        ui.label(RichText::new(format!("KEL — Keltner Channels — {}", series.symbol)).strong());
        Plot::new("kel_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let mid_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !mid[i].is_nan() { Some([c.t, mid[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(mid_pts).color(AMBER).width(2.0_f32).name("EMA20"));
                let upper_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !mid[i].is_nan() && !atr_vals[i].is_nan() { Some([c.t, mid[i] + 2.0 * atr_vals[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(upper_pts).color(INFO).width(1.0_f32).name("Upper"));
                let lower_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !mid[i].is_nan() && !atr_vals[i].is_nan() { Some([c.t, mid[i] - 2.0 * atr_vals[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(lower_pts).color(INFO).width(1.0_f32).name("Lower"));
            });
    }

    fn draw_donchian(&self, ui: &mut egui::Ui) {
        let series = &self.data.candles;
        ui.label(RichText::new(format!("DON — Donchian Channels — {}", series.symbol)).strong());
        Plot::new("don_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().map(|c| [c.t, c.close]).collect();
                plot_ui.line(Line::new(pts).color(AMBER).width(1.5_f32).name("Price"));
            });
    }
    // ==================== RISK & PORTFOLIO ====================

    fn draw_drawdown(&self, ui: &mut egui::Ui) {
        let dd = bt_viz::drawdown::compute_drawdown(&self.data.equity);
        ui.label(RichText::new("VAR — Drawdown Underwater Chart").strong());
        Plot::new("drawdown_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let pts: PlotPoints = dd.iter().enumerate().map(|(i, &v)| [i as f64, v]).collect();
                plot_ui.line(Line::new(pts).color(LOSS).width(2.0_f32));
            });
    }

    fn draw_correlation(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Rolling Correlation Matrix").strong());
        let matrix = bt_viz::correlation_heatmap::correlation_matrix(&self.data.corr);
        let n = self.data.corr.len();
        egui::Grid::new("corr_grid").striped(false).spacing(Vec2::new(2.0_f32, 2.0_f32)).show(ui, |ui| {
            ui.label("");
            for (label, _) in &self.data.corr {
                ui.label(RichText::new(label).small());
            }
            ui.end_row();
            for i in 0..n {
                ui.label(RichText::new(&self.data.corr[i].0).small());
                for j in 0..n {
                    let v = matrix[i][j];
                    let t = v.abs().clamp(0.0, 1.0);
                    let target = if v >= 0.0 { PROFIT } else { LOSS };
                    let bg = Color32::from_rgb(lerp(20, target.r(), t), lerp(20, target.g(), t), lerp(20, target.b(), t));
                    let frame = egui::Frame::none().fill(bg).inner_margin(6.0_f32);
                    frame.show(ui, |ui| {
                        ui.label(RichText::new(format!("{:.2}", v)).color(Color32::WHITE).small());
                    });
                }
                ui.end_row();
            }
        });
    }

    fn draw_vol_smile(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("SKEW — Volatility Smile / Skew").strong());
        Plot::new("vol_smile_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                let colors = [AMBER, INFO, PROFIT, LOSS];
                for (idx, (label, moneyness, iv)) in self.data.smiles.iter().enumerate() {
                    let color = colors[idx % colors.len()];
                    let pts: PlotPoints = moneyness.iter().zip(iv).map(|(&m, &v)| [m, v]).collect();
                    plot_ui.line(Line::new(pts).color(color).width(2.0_f32).name(label));
                }
            });
    }

    fn draw_efficient_frontier(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("PORT/MARS — Efficient Frontier").strong());
        Plot::new("frontier_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let sharpe_min = self.data.portfolios.iter().map(|p| p.sharpe).fold(f64::MAX, f64::min);
                let sharpe_max = self.data.portfolios.iter().map(|p| p.sharpe).fold(f64::MIN, f64::max);
                let range = (sharpe_max - sharpe_min).max(1e-9);
                let buckets = 6;
                for b in 0..buckets {
                    let lo = sharpe_min + range * b as f64 / buckets as f64;
                    let hi = sharpe_min + range * (b + 1) as f64 / buckets as f64;
                    let t = b as f64 / (buckets - 1) as f64;
                    let color = Color32::from_rgb(lerp(LOSS.r(), PROFIT.r(), t), lerp(LOSS.g(), PROFIT.g(), t), lerp(LOSS.b(), PROFIT.b(), t));
                    let pts: PlotPoints = self.data.portfolios.iter()
                        .filter(|p| p.sharpe >= lo && p.sharpe <= hi)
                        .map(|p| [p.risk, p.ret])
                        .collect();
                    plot_ui.points(Points::new(pts).color(color).radius(2.5_f32));
                }
                if let Some(best) = self.data.portfolios.iter().max_by(|a, b| a.sharpe.partial_cmp(&b.sharpe).unwrap()) {
                    plot_ui.points(Points::new(PlotPoints::from(vec![[best.risk, best.ret]])).color(AMBER).radius(7.0_f32).shape(MarkerShape::Diamond));
                }
            });
    }

    fn draw_rolling_sharpe(&self, ui: &mut egui::Ui) {
        use bt_analytics::rolling_sharpe;
        let rets = &self.data.returns;
        let rs = rolling_sharpe(rets, 20, 0.05, 252);
        ui.label(RichText::new("Rolling Sharpe Ratio").strong());
        Plot::new("rs_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let pts: PlotPoints = rs.iter().enumerate()
                    .filter_map(|(i, &v)| if !v.is_nan() { Some([i as f64, v]) } else { None })
                    .collect();
                plot_ui.line(Line::new(pts).color(PROFIT).width(2.0_f32));
                plot_ui.hline(egui_plot::HLine::new(1.0).color(AMBER));
                plot_ui.hline(egui_plot::HLine::new(0.0).color(Color32::GRAY));
            });
    }

    fn draw_rolling_sortino(&self, ui: &mut egui::Ui) {
        use bt_analytics::rolling_sortino;
        let rets = &self.data.returns;
        let rs = rolling_sortino(rets, 20, 0.05, 252);
        ui.label(RichText::new("Rolling Sortino Ratio").strong());
        Plot::new("rsort_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let pts: PlotPoints = rs.iter().enumerate()
                    .filter_map(|(i, &v)| if !v.is_nan() { Some([i as f64, v]) } else { None })
                    .collect();
                plot_ui.line(Line::new(pts).color(INFO).width(2.0_f32));
                plot_ui.hline(egui_plot::HLine::new(1.0).color(AMBER));
                plot_ui.hline(egui_plot::HLine::new(0.0).color(Color32::GRAY));
            });
    }

    fn draw_beta_alpha(&self, ui: &mut egui::Ui) {
        use bt_analytics::{alpha, beta};
        let rets = &self.data.returns;
        let b = beta(rets, rets);
        let a = alpha(rets, rets, 0.05, 252);
        ui.label(RichText::new("Beta / Alpha Analysis").strong());
        ui.label(format!("Beta: {:.4}", b));
        ui.label(format!("Alpha (annual): {:.4}", a));
        Plot::new("beta_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let pts: PlotPoints = rets.iter().enumerate().map(|(i, &v)| [i as f64, v]).collect();
                plot_ui.line(Line::new(pts).color(AMBER).width(1.5_f32));
            });
    }

    fn draw_rolling_max_dd(&self, ui: &mut egui::Ui) {
        use bt_analytics::rolling_max_drawdown;
        let series = &self.data.candles;
        let rmdd = rolling_max_drawdown(series, 20);
        ui.label(RichText::new("Rolling Max Drawdown").strong());
        Plot::new("rmdd_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !rmdd[i].is_nan() { Some([c.t, rmdd[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(pts).color(LOSS).width(2.0_f32));
            });
    }

    fn draw_var_backtest(&self, ui: &mut egui::Ui) {
        use bt_analytics::var_historical;
        let rets = &self.data.returns;
        let var_95 = var_historical(rets, 0.95);
        let var_99 = var_historical(rets, 0.99);
        ui.label(RichText::new("VaR Backtest").strong());
        ui.label(format!("VaR 95%: {:.4}", var_95));
        ui.label(format!("VaR 99%: {:.4}", var_99));
        Plot::new("var_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let pts: PlotPoints = rets.iter().enumerate().map(|(i, &v)| [i as f64, v]).collect();
                plot_ui.line(Line::new(pts).color(AMBER).width(1.0_f32));
                plot_ui.hline(egui_plot::HLine::new(var_95).color(LOSS));
                plot_ui.hline(egui_plot::HLine::new(var_99).color(PURPLE));
            });
    }

    fn draw_monte_carlo(&self, ui: &mut egui::Ui) {
        let rets = &self.data.returns;
        let mean = rets.iter().sum::<f64>() / rets.len() as f64;
        let variance = rets.iter().map(|&r| (r - mean).powi(2)).sum::<f64>() / rets.len() as f64;
        let std_dev = variance.sqrt();
        ui.label(RichText::new("Monte Carlo Simulation").strong());
        Plot::new("mc_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let mut rng = self.seed_counter;
                let colors = [AMBER, INFO, PROFIT, LOSS, PURPLE];
                for (idx, color) in colors.iter().enumerate() {
                    let mut price = 100.0 + idx as f64 * 5.0;
                    let mut path = vec![price];
                    for _ in 0..100 {
                        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
                        let z = ((rng >> 11) as f64 / (1u64 << 53) as f64) * 2.0 - 1.0;
                        price *= 1.0 + mean + std_dev * z;
                        path.push(price);
                    }
                    let pts: PlotPoints = path.iter().enumerate().map(|(i, &v)| [i as f64, v]).collect();
                    plot_ui.line(Line::new(pts).color(*color).width(1.5_f32));
                }
            });
    }
    // ==================== VOLATILITY & OPTIONS ====================

    fn draw_vol_smile_opt(&self, ui: &mut egui::Ui) {
        self.draw_vol_smile(ui);
    }

    fn draw_iv_surface(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("IV Surface").strong());
        let avail = ui.available_size();
        let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let strikes = [0.8_f64, 0.9, 1.0, 1.1, 1.2];
        let tenors = [7.0_f64, 14.0, 30.0, 60.0, 90.0];
        let cell_w = rect.width() / strikes.len() as f32;
        let cell_h = rect.height() / tenors.len() as f32;
        for (ti, &tenor) in tenors.iter().enumerate() {
            for (si, &strike) in strikes.iter().enumerate() {
                let moneyness = strike - 1.0;
                let iv = 0.20 + 0.05 * moneyness * moneyness * 10.0 + 0.02 / (tenor / 30.0).sqrt();
                let intensity = ((iv - 0.15) / 0.15).clamp(0.0, 1.0);
                let color = Color32::from_rgb(lerp(10, INFO.r(), intensity), lerp(10, INFO.g(), intensity), lerp(10, INFO.b(), intensity));
                let tile = egui::Rect::from_min_size(
                    egui::Pos2::new(rect.left() + si as f32 * cell_w, rect.top() + ti as f32 * cell_h),
                    Vec2::new(cell_w - 2.0_f32, cell_h - 2.0_f32),
                );
                painter.rect_filled(tile, 4.0_f32, color);
            }
        }
    }

    fn draw_term_structure(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("IV Term Structure").strong());
        Plot::new("ts_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let tenors = [7.0_f64, 14.0, 30.0, 60.0, 90.0, 180.0];
                let ivs: Vec<f64> = tenors.iter().map(|&t| 0.20 + 0.03 / (t / 30.0).sqrt()).collect();
                plot_ui.line(Line::new(tenors.iter().zip(ivs.iter()).map(|(&t, &v)| [t, v]).collect::<PlotPoints>()).color(AMBER).width(2.0_f32));
                plot_ui.points(Points::new(tenors.iter().zip(ivs.iter()).map(|(&t, &v)| [t, v]).collect::<PlotPoints>()).color(AMBER).radius(4.0_f32).shape(MarkerShape::Circle));
            });
    }

    fn draw_greeks_heatmap(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Greeks Heatmap").strong());
        let avail = ui.available_size();
        let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let greeks = ["Delta", "Gamma", "Theta", "Vega", "Rho"];
        let strikes = [0.8_f64, 0.9, 1.0, 1.1, 1.2];
        let cell_w = rect.width() / strikes.len() as f32;
        let cell_h = rect.height() / greeks.len() as f32;
        for (gi, greek) in greeks.iter().enumerate() {
            for (si, &strike) in strikes.iter().enumerate() {
                let moneyness = (strike - 1.0).abs();
                let val = match *greek {
                    "Delta" => 0.5 + (strike - 1.0) * 2.0,
                    "Gamma" => 0.1 * (1.0 - moneyness * 5.0),
                    "Theta" => -0.05 * (1.0 - moneyness * 3.0),
                    "Vega" => 0.15 * (1.0 - moneyness * 4.0),
                    "Rho" => 0.05 * (strike - 1.0),
                    _ => 0.0,
                };
                let intensity = val.abs().clamp(0.0, 1.0);
                let color = if val >= 0.0 {
                    Color32::from_rgb(lerp(10, PROFIT.r(), intensity), lerp(10, PROFIT.g(), intensity), lerp(10, PROFIT.b(), intensity))
                } else {
                    Color32::from_rgb(lerp(10, LOSS.r(), intensity), lerp(10, LOSS.g(), intensity), lerp(10, LOSS.b(), intensity))
                };
                let tile = egui::Rect::from_min_size(
                    egui::Pos2::new(rect.left() + si as f32 * cell_w, rect.top() + gi as f32 * cell_h),
                    Vec2::new(cell_w - 2.0_f32, cell_h - 2.0_f32),
                );
                painter.rect_filled(tile, 4.0_f32, color);
            }
        }
    }

    fn draw_vix_term(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("VIX Term Structure").strong());
        Plot::new("vix_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let tenors = [7.0_f64, 14.0, 30.0, 60.0, 90.0, 180.0, 365.0];
                let vix_vals: Vec<f64> = tenors.iter().map(|&t| 15.0 + 5.0 / (t / 30.0).sqrt()).collect();
                let pts: PlotPoints = tenors.iter().zip(vix_vals.iter()).map(|(&t, &v)| [t, v]).collect();
                plot_ui.line(Line::new(pts).color(PURPLE).width(2.0_f32));
            });
    }

    fn draw_vol_cone(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Volatility Cone").strong());
        Plot::new("vol_cone_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let tenors = [7.0_f64, 14.0, 30.0, 60.0, 90.0];
                let high: Vec<f64> = tenors.iter().map(|&t| 0.25 + 0.05 / (t / 30.0).sqrt()).collect();
                let low: Vec<f64> = tenors.iter().map(|&t| 0.12 + 0.02 / (t / 30.0).sqrt()).collect();
                let current = 0.20;
                let high_pts: PlotPoints = tenors.iter().zip(high.iter()).map(|(&t, &v)| [t, v]).collect();
                let low_pts: PlotPoints = tenors.iter().zip(low.iter()).map(|(&t, &v)| [t, v]).collect();
                let curr_pts: PlotPoints = tenors.iter().map(|&t| [t, current]).collect();
                plot_ui.line(Line::new(high_pts).color(LOSS).width(1.5_f32).name("High"));
                plot_ui.line(Line::new(low_pts).color(PROFIT).width(1.5_f32).name("Low"));
                plot_ui.line(Line::new(curr_pts).color(AMBER).width(2.0_f32).name("Current"));
            });
    }

    fn draw_option_payoff(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Option Payoff Diagram").strong());
        Plot::new("payoff_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let strike = 100.0_f64;
                let premium = 5.0_f64;
                let long_call: PlotPoints = (0..=200).map(|i| {
                    let s = 50.0 + i as f64 * 1.5;
                    [s, (s - strike).max(0.0) - premium]
                }).collect();
                let short_call: PlotPoints = (0..=200).map(|i| {
                    let s = 50.0 + i as f64 * 1.5;
                    [s, -(s - strike).max(0.0) + premium]
                }).collect();
                let long_put: PlotPoints = (0..=200).map(|i| {
                    let s = 50.0 + i as f64 * 1.5;
                    [s, (strike - s).max(0.0) - premium]
                }).collect();
                plot_ui.line(Line::new(long_call).color(PROFIT).width(2.0_f32).name("Long Call"));
                plot_ui.line(Line::new(short_call).color(LOSS).width(2.0_f32).name("Short Call"));
                plot_ui.line(Line::new(long_put).color(INFO).width(2.0_f32).name("Long Put"));
                plot_ui.vline(egui_plot::VLine::new(strike).color(Color32::GRAY));
                plot_ui.hline(egui_plot::HLine::new(0.0).color(Color32::GRAY));
            });
    }

    fn draw_skew_evolution(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Skew Evolution").strong());
        Plot::new("skew_evo_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let days: Vec<f64> = (0..30).map(|i| i as f64).collect();
                let skew: Vec<f64> = days.iter().map(|&d| -0.3 + 0.01 * d + 0.05 * (d * 0.5).sin()).collect();
                let pts: PlotPoints = days.iter().zip(skew.iter()).map(|(&d, &s)| [d, s]).collect();
                plot_ui.line(Line::new(pts).color(PURPLE).width(2.0_f32));
                plot_ui.hline(egui_plot::HLine::new(0.0).color(Color32::GRAY));
            });
    }

    fn draw_gamma_exposure(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Gamma Exposure by Strike").strong());
        Plot::new("gamma_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let strikes: Vec<f64> = (0..20).map(|i| 80.0 + i as f64 * 5.0).collect();
                let gamma: Vec<f64> = strikes.iter().map(|&s| {
                    let d = (s - 100.0) / 10.0;
                    0.05 * (-d * d / 2.0).exp()
                }).collect();
                let pts: PlotPoints = strikes.iter().zip(gamma.iter()).map(|(&s, &g)| [s, g]).collect();
                plot_ui.line(Line::new(pts).color(INFO).width(2.0_f32));
                plot_ui.vline(egui_plot::VLine::new(100.0).color(AMBER));
            });
    }

    fn draw_put_call_ratio(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Put/Call Ratio").strong());
        Plot::new("pcr_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let days: Vec<f64> = (0..30).map(|i| i as f64).collect();
                let pcr: Vec<f64> = days.iter().map(|&d| 0.8 + 0.2 * (d * 0.3).sin() + 0.1 * (d * 0.1).cos()).collect();
                let pts: PlotPoints = days.iter().zip(pcr.iter()).map(|(&d, &p)| [d, p]).collect();
                plot_ui.line(Line::new(pts).color(AMBER).width(2.0_f32));
                plot_ui.hline(egui_plot::HLine::new(1.0).color(Color32::GRAY));
            });
    }

    fn draw_iv_rank(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("IV Rank").strong());
        let days: Vec<f64> = (0..252).map(|i| i as f64).collect();
        let iv: Vec<f64> = days.iter().map(|&d| 0.15 + 0.10 * (d * 0.05).sin() + 0.05 * (d * 0.02).cos()).collect();
        let current_iv = iv.last().copied().unwrap_or(0.20);
        let min_iv = iv.iter().fold(f64::INFINITY, |a, b| a.min(*b));
        let max_iv = iv.iter().fold(f64::NEG_INFINITY, |a, b| a.max(*b));
        let iv_rank = ((current_iv - min_iv) / (max_iv - min_iv).max(1e-9) * 100.0) as i32;
        ui.label(format!("Current IV Rank: {}%", iv_rank));
        Plot::new("ivr_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let pts: PlotPoints = days.iter().zip(iv.iter()).map(|(&d, &v)| [d, v]).collect();
                plot_ui.line(Line::new(pts).color(INFO).width(1.5_f32));
                plot_ui.hline(egui_plot::HLine::new(current_iv).color(AMBER));
            });
    }

    fn draw_sharpe_surface(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Sharpe Ratio Surface").strong());
        let avail = ui.available_size();
        let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let x_labels = ["Conservative", "Moderate", "Balanced", "Growth", "Aggressive"];
        let y_labels = ["1M", "3M", "6M", "1Y", "3Y"];
        let cell_w = rect.width() / x_labels.len() as f32;
        let cell_h = rect.height() / y_labels.len() as f32;
        for (yi, _yl) in y_labels.iter().enumerate() {
            for (xi, _xl) in x_labels.iter().enumerate() {
                let sharpe = 0.5 + xi as f64 * 0.3 - yi as f64 * 0.1;
                let intensity = ((sharpe - 0.2) / 1.5).clamp(0.0, 1.0);
                let color = Color32::from_rgb(lerp(10, PROFIT.r(), intensity), lerp(10, PROFIT.g(), intensity), lerp(10, PROFIT.b(), intensity));
                let tile = egui::Rect::from_min_size(
                    egui::Pos2::new(rect.left() + xi as f32 * cell_w, rect.top() + yi as f32 * cell_h),
                    Vec2::new(cell_w - 2.0_f32, cell_h - 2.0_f32),
                );
                painter.rect_filled(tile, 4.0_f32, color);
            }
        }
    }
    // ==================== MICROSTRUCTURE ====================

    fn draw_acf_pacf(&self, ui: &mut egui::Ui) {
        let max_lag = 25usize;
        let a = bt_viz::acf_pacf::acf(&self.data.returns, max_lag);
        let p = bt_viz::acf_pacf::pacf(&self.data.returns, max_lag);
        let conf = 1.96 / (self.data.returns.len() as f64).sqrt();

        ui.label(RichText::new("ACF").strong());
        Plot::new("acf_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height() * 0.45_f32)
            .show(ui, |plot_ui| {
                draw_lollipop(plot_ui, &a, conf);
            });
        ui.label(RichText::new("PACF").strong());
        Plot::new("pacf_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                draw_lollipop(plot_ui, &p, conf);
            });
    }

    fn draw_hurst(&self, ui: &mut egui::Ui) {
        let rets = &self.data.returns;
        ui.label(RichText::new("Hurst Exponent").strong());
        let n = rets.len();
        let mut log_n = Vec::new();
        let mut log_r_s = Vec::new();
        for chunk_size in [10, 20, 40, 80] {
            if chunk_size >= n { continue; }
            let mut r_s_values = Vec::new();
            for chunk in rets.chunks(chunk_size) {
                let mean = chunk.iter().sum::<f64>() / chunk.len() as f64;
                let mut cum_dev = 0.0;
                let mut max_dev = f64::MIN;
                let mut min_dev = f64::MAX;
                for &r in chunk {
                    cum_dev += r - mean;
                    max_dev = max_dev.max(cum_dev);
                    min_dev = min_dev.min(cum_dev);
                }
                let range = max_dev - min_dev;
                let variance = chunk.iter().map(|&r| (r - mean).powi(2)).sum::<f64>() / chunk.len() as f64;
                let std_dev = variance.sqrt();
                if std_dev > 0.0 {
                    r_s_values.push(range / std_dev);
                }
            }
            if !r_s_values.is_empty() {
                let avg_r_s = r_s_values.iter().sum::<f64>() / r_s_values.len() as f64;
                log_n.push((chunk_size as f64).ln());
                log_r_s.push(avg_r_s.ln());
            }
        }
        let hurst = if log_n.len() >= 2 {
            let n_points = log_n.len() as f64;
            let sum_x = log_n.iter().sum::<f64>();
            let sum_y = log_r_s.iter().sum::<f64>();
            let sum_xy = log_n.iter().zip(log_r_s.iter()).map(|(x, y)| x * y).sum::<f64>();
            let sum_x2 = log_n.iter().map(|x| x * x).sum::<f64>();
            (n_points * sum_xy - sum_x * sum_y) / (n_points * sum_x2 - sum_x * sum_x)
        } else {
            0.5
        };
        ui.label(format!("Hurst Exponent: {:.4}", hurst));
        Plot::new("hurst_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let pts: PlotPoints = log_n.iter().zip(log_r_s.iter()).map(|(&x, &y)| [x, y]).collect();
                plot_ui.points(Points::new(pts).color(AMBER).radius(5.0_f32).shape(MarkerShape::Circle));
                if log_n.len() >= 2 {
                    let x_min = log_n.iter().fold(f64::INFINITY, |a, b| a.min(*b));
                    let x_max = log_n.iter().fold(f64::NEG_INFINITY, |a, b| a.max(*b));
                    let slope = hurst;
                    let intercept = (log_r_s.iter().sum::<f64>() - slope * log_n.iter().sum::<f64>()) / log_n.len() as f64;
                    let line_pts = vec![[x_min, slope * x_min + intercept], [x_max, slope * x_max + intercept]];
                    plot_ui.line(Line::new(line_pts).color(INFO).width(2.0_f32));
                }
            });
    }

    fn draw_wavelet(&self, ui: &mut egui::Ui) {
        let rets = &self.data.returns;
        ui.label(RichText::new("Wavelet Power Spectrum").strong());
        Plot::new("wavelet_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let scales: Vec<f64> = (1..=20).map(|i| i as f64 * 2.0).collect();
                let power: Vec<f64> = scales.iter().map(|&s| {
                    let mut sum = 0.0;
                    for i in 0..rets.len().min(100) {
                        sum += (rets[i] * (i as f64 / s).cos()).powi(2);
                    }
                    sum / rets.len() as f64
                }).collect();
                let pts: PlotPoints = scales.iter().zip(power.iter()).map(|(&s, &p)| [s, p]).collect();
                plot_ui.line(Line::new(pts).color(PURPLE).width(2.0_f32));
            });
    }

    fn draw_kalman(&self, ui: &mut egui::Ui) {
        let candles = &self.data.candles;
        ui.label(RichText::new("Kalman Filter").strong());
        Plot::new("kalman_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let mut estimate = candles.candles[0].close;
                let mut error = 1.0;
                let process_noise = 0.01;
                let measurement_noise = 0.1;
                let mut estimates = Vec::new();
                for c in &candles.candles {
                    let prediction = estimate;
                    let prediction_error = error + process_noise;
                    let kalman_gain = prediction_error / (prediction_error + measurement_noise);
                    estimate = prediction + kalman_gain * (c.close - prediction);
                    error = (1.0 - kalman_gain) * prediction_error;
                    estimates.push(estimate);
                }
                let pts: PlotPoints = candles.candles.iter().map(|c| [c.t, c.close]).collect();
                plot_ui.line(Line::new(pts).color(AMBER).width(1.0_f32).name("Observed"));
                let est_pts: PlotPoints = candles.candles.iter().enumerate()
                    .map(|(i, c)| [c.t, estimates[i]])
                    .collect();
                plot_ui.line(Line::new(est_pts).color(INFO).width(2.0_f32).name("Filtered"));
            });
    }

    fn draw_markov_regime(&self, ui: &mut egui::Ui) {
        let rets = &self.data.returns;
        ui.label(RichText::new("Markov Regime Switching").strong());
        Plot::new("markov_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let mut regime = 0;
                let mut regimes = Vec::new();
                for &r in rets {
                    if r > 0.01 { regime = 1; }
                    else if r < -0.01 { regime = 0; }
                    regimes.push(regime);
                }
                let pts: PlotPoints = regimes.iter().enumerate().map(|(i, &r)| [i as f64, r as f64]).collect();
                plot_ui.line(Line::new(pts).color(AMBER).width(2.0_f32));
                plot_ui.hline(egui_plot::HLine::new(0.5).color(Color32::GRAY));
            });
    }

    fn draw_copula_3d(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Copula 3D Scatter").strong());
        let avail = ui.available_size();
        let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let n = self.data.returns.len().min(200);
        let mut rng = self.seed_counter;
        for i in 0..n {
            let u = self.data.returns[i].abs().min(1.0);
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let v = ((rng >> 11) as f64 / (1u64 << 53) as f64).min(1.0);
            let x = rect.left() + u as f32 * rect.width();
            let y = rect.bottom() - v as f32 * rect.height();
            let color = if self.data.returns[i] >= 0.0 { PROFIT } else { LOSS };
            painter.circle_filled(egui::Pos2::new(x, y), 3.0, color);
        }
    }

    fn draw_qq_plot(&self, ui: &mut egui::Ui) {
        let rets = &self.data.returns;
        ui.label(RichText::new("Q-Q Plot").strong());
        Plot::new("qq_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let mut sorted = rets.to_vec();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let n = sorted.len();
                let mean = sorted.iter().sum::<f64>() / n as f64;
                let std_dev = (sorted.iter().map(|&r| (r - mean).powi(2)).sum::<f64>() / n as f64).sqrt();
                let pts: PlotPoints = sorted.iter().enumerate().map(|(i, &v)| {
                    let p = (i as f64 + 0.5) / n as f64;
                    let z = normal_inverse(p);
                    [z * std_dev + mean, v]
                }).collect();
                plot_ui.points(Points::new(pts).color(AMBER).radius(2.0_f32).shape(MarkerShape::Circle));
                let min_v = sorted.iter().fold(f64::INFINITY, |a, b| a.min(*b));
                let max_v = sorted.iter().fold(f64::NEG_INFINITY, |a, b| a.max(*b));
                let line_pts = vec![[min_v, min_v], [max_v, max_v]];
                plot_ui.line(Line::new(line_pts).color(INFO).width(1.5_f32));
            });
    }

    fn draw_return_dist(&self, ui: &mut egui::Ui) {
        let rets = &self.data.returns;
        ui.label(RichText::new("Return Distribution").strong());
        Plot::new("dist_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let min_r = rets.iter().fold(f64::INFINITY, |a, b| a.min(*b));
                let max_r = rets.iter().fold(f64::NEG_INFINITY, |a, b| a.max(*b));
                let bins = 30;
                let bin_width = (max_r - min_r) / bins as f64;
                let mut counts = vec![0.0_f64; bins];
                for &r in rets {
                    let idx = ((r - min_r) / bin_width).floor() as usize;
                    let idx = idx.min(bins - 1);
                    counts[idx] += 1.0;
                }
                let bars: Vec<Bar> = counts.iter().enumerate()
                    .map(|(i, &c)| Bar::new(min_r + i as f64 * bin_width, c).width(bin_width * 0.9).fill(INFO))
                    .collect();
                plot_ui.bar_chart(BarChart::new(bars));
            });
    }

    fn draw_rolling_moments(&self, ui: &mut egui::Ui) {
        use bt_analytics::rolling_moments;
        let rets = &self.data.returns;
        let (roll_mean, roll_std) = rolling_moments(rets, 20);
        ui.label(RichText::new("Rolling Moments").strong());
        Plot::new("rm_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let mean_pts: PlotPoints = roll_mean.iter().enumerate()
                    .filter_map(|(i, &v)| if !v.is_nan() { Some([i as f64, v]) } else { None })
                    .collect();
                plot_ui.line(Line::new(mean_pts).color(AMBER).width(2.0_f32).name("Mean"));
                let std_pts: PlotPoints = roll_std.iter().enumerate()
                    .filter_map(|(i, &v)| if !v.is_nan() { Some([i as f64, v]) } else { None })
                    .collect();
                plot_ui.line(Line::new(std_pts).color(INFO).width(2.0_f32).name("Std Dev"));
            });
    }
    // ==================== INDIA-SPECIFIC ====================

    fn draw_treemap(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("BMAP — NIFTY Sector Map").strong());
        let avail = ui.available_size();
        let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let mut sorted = self.data.treemap_nodes.clone();
        sorted.sort_by(|a, b| b.market_cap.partial_cmp(&a.market_cap).unwrap());
        let full = bt_viz::sector_treemap::Rect { x: 0.0, y: 0.0, w: rect.width() as f64, h: rect.height() as f64 };
        let rects = bt_viz::sector_treemap::layout(&sorted, full);
        for (node, r) in sorted.iter().zip(rects.iter()) {
            let t = (node.pct_change.abs() / 3.0).clamp(0.15, 1.0);
            let target = if node.pct_change >= 0.0 { PROFIT } else { LOSS };
            let color = Color32::from_rgb(lerp(15, target.r(), t), lerp(15, target.g(), t), lerp(15, target.b(), t));
            let tile = egui::Rect::from_min_size(rect.min + Vec2::new(r.x as f32, r.y as f32), Vec2::new(r.w as f32, r.h as f32));
            painter.rect_filled(tile, 0.0, color);
            painter.rect_stroke(tile, 0.0, Stroke::new(1.0_f32, Color32::BLACK));
            if r.w > 60.0 && r.h > 28.0 {
                painter.text(tile.min + Vec2::new(6.0_f32, 6.0_f32), egui::Align2::LEFT_TOP,
                    format!("{}\n{:+.2}%", node.label, node.pct_change), egui::FontId::monospace(13.0_f32), Color32::WHITE);
            }
        }
    }

    fn draw_heatmap(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Sensex Heatmap").strong());
        let avail = ui.available_size();
        let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let stocks: [(&str, f64); 30] = [
            ("RELIANCE", 1.2), ("HDFCBANK", 0.8), ("ICICIBANK", 1.5), ("INFY", -0.5),
            ("TCS", 0.3), ("BHARTIARTL", 2.1), ("ITC", -0.2), ("LT", 1.8),
            ("SBIN", 2.5), ("KOTAKBANK", 0.9), ("HINDUNILVR", -0.8), ("BAJFINANCE", 1.1),
            ("ASIANPAINT", -0.3), ("MARUTI", 1.4), ("SUNPHARMA", -1.2), ("TITAN", 0.7),
            ("ULTRACEMCO", 1.0), ("NESTLEIND", -0.4), ("WIPRO", -0.6), ("HCLTECH", 0.5),
            ("AXISBANK", 1.3), ("TATAMOTORS", 2.0), ("TATASTEEL", 1.5), ("ADANIENT", 3.2),
            ("ADANIPORTS", 1.8), ("BAJAJ-AUTO", 0.6), ("COALINDIA", -0.5), ("NTPC", 0.4),
            ("POWERGRID", 0.3), ("TECHM", -0.4),
        ];
        let cols = 6;
        let rows = (stocks.len() + cols - 1) / cols;
        let cell_w = rect.width() / cols as f32;
        let cell_h = rect.height() / rows as f32;
        let padding = 2.0_f32;
        for (idx, (sym, change)) in stocks.iter().enumerate() {
            let col = idx % cols;
            let row = idx / cols;
            let x = rect.left() + col as f32 * cell_w + padding;
            let y = rect.top() + row as f32 * cell_h + padding;
            let w = cell_w - 2.0_f32 * padding;
            let h = cell_h - 2.0_f32 * padding;
            let intensity = (change.abs() / 3.0).clamp(0.2, 1.0);
            let target = if *change >= 0.0 { PROFIT } else { LOSS };
            let color = Color32::from_rgb(lerp(15, target.r(), intensity), lerp(15, target.g(), intensity), lerp(15, target.b(), intensity));
            let tile = egui::Rect::from_min_size(egui::Pos2::new(x, y), Vec2::new(w, h));
            painter.rect_filled(tile, 4.0_f32, color);
            if w > 50.0 && h > 20.0 {
                painter.text(tile.min + Vec2::new(4.0_f32, 4.0_f32), egui::Align2::LEFT_TOP,
                    format!("{}\n{:+.1}%", sym, change), egui::FontId::monospace(10.0_f32), Color32::WHITE);
            }
        }
    }

    fn draw_fii_dii_flow(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("FII/DII Flow").strong());
        Plot::new("fii_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let days: Vec<f64> = (0..20).map(|i| i as f64).collect();
                let fii: Vec<f64> = days.iter().map(|&d| 500.0 + 200.0 * (d * 0.5).sin() + 100.0 * (d * 0.3).cos()).collect();
                let dii: Vec<f64> = days.iter().map(|&d| 300.0 + 150.0 * (d * 0.4).cos() + 80.0 * (d * 0.7).sin()).collect();
                let fii_pts: PlotPoints = days.iter().zip(fii.iter()).map(|(&d, &v)| [d, v]).collect();
                let dii_pts: PlotPoints = days.iter().zip(dii.iter()).map(|(&d, &v)| [d, v]).collect();
                plot_ui.line(Line::new(fii_pts).color(INFO).width(2.0_f32).name("FII"));
                plot_ui.line(Line::new(dii_pts).color(AMBER).width(2.0_f32).name("DII"));
            });
    }

    fn draw_sector_perf(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("WEI — Sector Performance").strong());
        let sectors: [(&str, f64); 15] = [
            ("IT", -0.5), ("Banking", 1.8), ("Oil & Gas", 1.2), ("FMCG", -0.3),
            ("Auto", 1.4), ("Pharma", -0.8), ("Metals", 1.5), ("Cons Dur", 0.7),
            ("Cement", 1.0), ("Telecom", 2.1), ("Power", 0.4), ("Fin Svcs", 1.1),
            ("Chemicals", 0.5), ("Construction", 1.8), ("Realty", 0.9),
        ];
        let avail = ui.available_size();
        let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let max_change = sectors.iter().map(|(_, c)| c.abs()).fold(0.0_f64, f64::max);
        let bar_height = rect.height() / sectors.len() as f32;
        let bar_max_w = rect.width() * 0.7_f32;
        let center_x = rect.center().x;
        for (i, (name, change)) in sectors.iter().enumerate() {
            let y = rect.top() + i as f32 * bar_height + 2.0_f32;
            let h = bar_height - 4.0_f32;
            let w = (change.abs() / max_change) * bar_max_w as f64;
            let color = if *change >= 0.0 { PROFIT } else { LOSS };
            let bar_rect = if *change >= 0.0 {
                egui::Rect::from_min_size(egui::Pos2::new(center_x, y), Vec2::new(w as f32, h))
            } else {
                egui::Rect::from_min_size(egui::Pos2::new(center_x - w as f32, y), Vec2::new(w as f32, h))
            };
            painter.rect_filled(bar_rect, 2.0_f32, color);
            painter.text(egui::Pos2::new(rect.left() + 4.0_f32, y + h / 2.0_f32), egui::Align2::LEFT_CENTER,
                *name, egui::FontId::monospace(11.0_f32), Color32::WHITE);
        }
        painter.line_segment([egui::Pos2::new(center_x, rect.top()), egui::Pos2::new(center_x, rect.bottom())],
            Stroke::new(1.0_f32, Color32::GRAY));
    }

    fn draw_yield_curve(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("GOVT — India Sovereign Yield Curve").strong());
        Plot::new("yield_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                let colors = [AMBER, INFO, PROFIT];
                for (idx, (label, tenors, yields)) in self.data.yield_curves.iter().enumerate() {
                    let color = colors[idx % colors.len()];
                    let pts: PlotPoints = tenors.iter().zip(yields).map(|(&t, &y)| [t, y]).collect();
                    plot_ui.line(Line::new(pts).color(color).width(2.0_f32).name(label));
                }
            });
    }

    fn draw_usdinr(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("USD/INR Exchange Rate").strong());
        Plot::new("usdinr_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let days: Vec<f64> = (0..60).map(|i| i as f64).collect();
                let rate: Vec<f64> = days.iter().map(|&d| 83.0 + 0.5 * (d * 0.1).sin() + 0.3 * (d * 0.05).cos()).collect();
                let pts: PlotPoints = days.iter().zip(rate.iter()).map(|(&d, &v)| [d, v]).collect();
                plot_ui.line(Line::new(pts).color(AMBER).width(2.0_f32));
            });
    }

    fn draw_monsoon_agri(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Monsoon & Agriculture").strong());
        Plot::new("monsoon_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let months = ["Jun", "Jul", "Aug", "Sep"];
                let rainfall = [150.0_f64, 280.0, 220.0, 180.0];
                let agri_growth = [3.5_f64, 4.2, 3.8, 3.0];
                let rain_pts: PlotPoints = rainfall.iter().enumerate().map(|(i, &v)| [i as f64, v]).collect();
                let agri_pts: PlotPoints = agri_growth.iter().enumerate().map(|(i, &v)| [i as f64, v * 50.0]).collect();
                plot_ui.line(Line::new(rain_pts).color(INFO).width(2.0_f32).name("Rainfall (mm)"));
                plot_ui.line(Line::new(agri_pts).color(PROFIT).width(2.0_f32).name("Agri Growth (x50)"));
                let _ = months;
            });
    }

    fn draw_seasonality(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Seasonality — Month x Weekday").strong());
        let avail = ui.available_size();
        let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let center = rect.center();
        let r_max = rect.width().min(rect.height()) * 0.42_f32;
        let r_min = r_max * 0.22_f32;
        let ring_step = (r_max - r_min) / 7.0_f32;
        let max_abs = self.data.seasonality.iter().flatten().cloned().fold(0.0_f64, |a, v| a.max(v.abs())).max(1e-6);
        for (m, row) in self.data.seasonality.iter().enumerate() {
            let theta0 = -std::f32::consts::FRAC_PI_2 + (m as f32) * std::f32::consts::TAU / 12.0_f32;
            let theta1 = theta0 + std::f32::consts::TAU / 12.0_f32;
            for (d, &v) in row.iter().enumerate() {
                let r_inner = r_min + ring_step * d as f32;
                let r_outer = r_inner + ring_step * 0.92_f32;
                let t = (v / max_abs).clamp(-1.0, 1.0) as f32;
                let target = if t >= 0.0 { PROFIT } else { LOSS };
                let alpha = t.abs();
                let color = Color32::from_rgb(lerp(15, target.r(), alpha as f64), lerp(15, target.g(), alpha as f64), lerp(15, target.b(), alpha as f64));
                let segments = 10;
                let mut points = Vec::with_capacity(segments * 2 + 2);
                for s in 0..=segments {
                    let t = theta0 + (theta1 - theta0) * (s as f32 / segments as f32);
                    points.push(center + Vec2::new(r_outer * t.cos(), r_outer * t.sin()));
                }
                for s in (0..=segments).rev() {
                    let t = theta0 + (theta1 - theta0) * (s as f32 / segments as f32);
                    points.push(center + Vec2::new(r_inner * t.cos(), r_inner * t.sin()));
                }
                painter.add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
            }
        }
    }
    // ==================== BLOOMBERG-STYLE ====================

    fn draw_world_indices(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("World Indices").strong());
        let indices = [
            ("S&P 500", 5200.0, 0.8), ("NASDAQ", 16500.0, 1.2), ("DOW", 39000.0, 0.5),
            ("FTSE", 8200.0, -0.3), ("DAX", 18500.0, 0.7), ("NIKKEI", 39000.0, 1.5),
            ("SHANGHAI", 3100.0, -0.8), ("HANG SENG", 18000.0, -1.2),
        ];
        let avail = ui.available_size();
        let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let bar_height = rect.height() / indices.len() as f32;
        for (i, (name, _value, change)) in indices.iter().enumerate() {
            let y = rect.top() + i as f32 * bar_height + 2.0_f32;
            let h = bar_height - 4.0_f32;
            let color = if *change >= 0.0 { PROFIT } else { LOSS };
            let w = (f64::abs(*change) / 2.0) * rect.width() as f64 * 0.3_f64;
            let bar_rect = if *change >= 0.0 {
                egui::Rect::from_min_size(egui::Pos2::new(rect.center().x, y), Vec2::new(w as f32, h))
            } else {
                egui::Rect::from_min_size(egui::Pos2::new(rect.center().x - w as f32, y), Vec2::new(w as f32, h))
            };
            painter.rect_filled(bar_rect, 2.0_f32, color);
            painter.text(egui::Pos2::new(rect.left() + 4.0_f32, y + h / 2.0_f32), egui::Align2::LEFT_CENTER,
                *name, egui::FontId::monospace(11.0_f32), Color32::WHITE);
        }
    }

    fn draw_ticker_tape(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Ticker Tape").strong());
        let avail = ui.available_size();
        let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let symbols = [
            ("RELIANCE", 2500.0, 1.2), ("TCS", 3800.0, -0.5), ("INFY", 1500.0, 0.8),
            ("HDFCBANK", 1650.0, 1.5), ("ICICIBANK", 980.0, -0.3), ("ITC", 450.0, 0.1),
        ];
        let row_height = rect.height() / symbols.len() as f32;
        for (i, (sym, price, change)) in symbols.iter().enumerate() {
            let y = rect.top() + i as f32 * row_height;
            let color = if *change >= 0.0 { PROFIT } else { LOSS };
            painter.text(egui::Pos2::new(rect.left() + 10.0_f32, y + row_height / 2.0_f32), egui::Align2::LEFT_CENTER,
                *sym, egui::FontId::monospace(12.0_f32), Color32::WHITE);
            painter.text(egui::Pos2::new(rect.center().x, y + row_height / 2.0_f32), egui::Align2::CENTER_CENTER,
                format!("{:.2}", price), egui::FontId::monospace(12.0_f32), Color32::WHITE);
            painter.text(egui::Pos2::new(rect.right() - 10.0_f32, y + row_height / 2.0_f32), egui::Align2::RIGHT_CENTER,
                format!("{:+.2}%", change), egui::FontId::monospace(12.0_f32), color);
        }
    }

    fn draw_currency_matrix(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Currency Matrix").strong());
        let avail = ui.available_size();
        let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let currencies = ["USD", "EUR", "GBP", "JPY", "INR"];
        let cell_w = rect.width() / currencies.len() as f32;
        let cell_h = rect.height() / currencies.len() as f32;
        for (i, c1) in currencies.iter().enumerate() {
            for (j, c2) in currencies.iter().enumerate() {
                let val = if i == j { 1.0 } else { 0.5 + (i as f64 * 0.1 + j as f64 * 0.05) };
                let intensity = ((val - 0.5) / 0.7).clamp(0.0, 1.0);
                let color = Color32::from_rgb(lerp(10, INFO.r(), intensity), lerp(10, INFO.g(), intensity), lerp(10, INFO.b(), intensity));
                let tile = egui::Rect::from_min_size(
                    egui::Pos2::new(rect.left() + j as f32 * cell_w, rect.top() + i as f32 * cell_h),
                    Vec2::new(cell_w - 2.0_f32, cell_h - 2.0_f32),
                );
                painter.rect_filled(tile, 4.0_f32, color);
            }
        }
    }

    fn draw_sector_wheel(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Sector Wheel").strong());
        let avail = ui.available_size();
        let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let center = rect.center();
        let radius = rect.width().min(rect.height()) * 0.4_f32;
        let sectors = [
            ("IT", -0.5), ("Banking", 1.8), ("Oil & Gas", 1.2), ("FMCG", -0.3),
            ("Auto", 1.4), ("Pharma", -0.8), ("Metals", 1.5), ("Cons Dur", 0.7),
        ];
        let angle_step = std::f32::consts::TAU / sectors.len() as f32;
        for (i, (name, change)) in sectors.iter().enumerate() {
            let angle = i as f32 * angle_step - std::f32::consts::FRAC_PI_2;
            let color = if *change >= 0.0 { PROFIT } else { LOSS };
            let label_pos = center + Vec2::new(radius * 1.2 * angle.cos(), radius * 1.2 * angle.sin());
            painter.text(label_pos, egui::Align2::CENTER_CENTER, *name, egui::FontId::monospace(10.0_f32), color);
            let inner = center + Vec2::new(radius * 0.6 * angle.cos(), radius * 0.6 * angle.sin());
            painter.circle_filled(inner, 8.0, color);
        }
    }

    fn draw_earnings_calendar(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Earnings Calendar").strong());
        let avail = ui.available_size();
        let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let earnings = [
            ("RELIANCE", "Q3 FY26", 15.2), ("TCS", "Q3 FY26", 8.5),
            ("INFY", "Q3 FY26", 5.3), ("HDFCBANK", "Q3 FY26", 12.1),
            ("ICICIBANK", "Q3 FY26", 10.8),
        ];
        let row_height = rect.height() / earnings.len() as f32;
        for (i, (sym, quarter, eps)) in earnings.iter().enumerate() {
            let y = rect.top() + i as f32 * row_height + 4.0_f32;
            painter.text(egui::Pos2::new(rect.left() + 10.0_f32, y), egui::Align2::LEFT_CENTER,
                *sym, egui::FontId::monospace(12.0_f32), Color32::WHITE);
            painter.text(egui::Pos2::new(rect.center().x, y), egui::Align2::CENTER_CENTER,
                *quarter, egui::FontId::monospace(11.0_f32), Color32::GRAY);
            painter.text(egui::Pos2::new(rect.right() - 10.0_f32, y), egui::Align2::RIGHT_CENTER,
                format!("EPS: {:.1}", eps), egui::FontId::monospace(12.0_f32), PROFIT);
        }
    }

    fn draw_econ_calendar(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Economic Calendar").strong());
        let avail = ui.available_size();
        let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let events = [
            ("RBI Rate Decision", "High", "6.50%"), ("CPI Inflation", "High", "5.2%"),
            ("GDP Growth", "Medium", "7.2%"), ("IIP", "Medium", "4.5%"),
            ("Trade Balance", "Medium", "-$20B"),
        ];
        let row_height = rect.height() / events.len() as f32;
        for (i, (event, impact, value)) in events.iter().enumerate() {
            let y = rect.top() + i as f32 * row_height + 4.0_f32;
            let impact_color = match *impact {
                "High" => LOSS, "Medium" => AMBER, _ => PROFIT,
            };
            painter.text(egui::Pos2::new(rect.left() + 10.0_f32, y), egui::Align2::LEFT_CENTER,
                *event, egui::FontId::monospace(12.0_f32), Color32::WHITE);
            painter.text(egui::Pos2::new(rect.center().x, y), egui::Align2::CENTER_CENTER,
                *impact, egui::FontId::monospace(11.0_f32), impact_color);
            painter.text(egui::Pos2::new(rect.right() - 10.0_f32, y), egui::Align2::RIGHT_CENTER,
                *value, egui::FontId::monospace(12.0_f32), PROFIT);
        }
    }

    fn draw_correlation_network(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Correlation Network").strong());
        let avail = ui.available_size();
        let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let nodes = ["RELIANCE", "TCS", "INFY", "HDFCBANK", "ICICIBANK", "ITC"];
        let n = nodes.len();
        let center = rect.center();
        let radius = rect.width().min(rect.height()) * 0.35_f32;
        let positions: Vec<egui::Pos2> = (0..n).map(|i| {
            let angle = i as f32 * std::f32::consts::TAU / n as f32 - std::f32::consts::FRAC_PI_2;
            center + Vec2::new(radius * angle.cos(), radius * angle.sin())
        }).collect();
        for i in 0..n {
            for j in (i + 1)..n {
                let corr = if i == j { 1.0 } else { 0.3 + (i as f64 * 0.1 + j as f64 * 0.05) % 0.5 };
                let color = if corr > 0.5 { PROFIT } else { INFO };
                painter.line_segment([positions[i], positions[j]], Stroke::new(1.0_f32, color));
            }
        }
        for (i, pos) in positions.iter().enumerate() {
            painter.circle_filled(*pos, 12.0, AMBER);
            painter.text(*pos, egui::Align2::CENTER_CENTER, nodes[i], egui::FontId::monospace(8.0_f32), Color32::BLACK);
        }
    }

    fn draw_return_heatmap(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Return Heatmap").strong());
        let avail = ui.available_size();
        let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let stocks = ["RELIANCE", "TCS", "INFY", "HDFCBANK", "ICICIBANK", "ITC"];
        let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun"];
        let cell_w = rect.width() / months.len() as f32;
        let cell_h = rect.height() / stocks.len() as f32;
        for (si, _sym) in stocks.iter().enumerate() {
            for (mi, _month) in months.iter().enumerate() {
                let ret = ((si * 7 + mi * 13) % 20) as f64 / 10.0 - 1.0;
                let intensity = ret.abs().clamp(0.0, 1.0);
                let color = if ret >= 0.0 {
                    Color32::from_rgb(lerp(10, PROFIT.r(), intensity), lerp(10, PROFIT.g(), intensity), lerp(10, PROFIT.b(), intensity))
                } else {
                    Color32::from_rgb(lerp(10, LOSS.r(), intensity), lerp(10, LOSS.g(), intensity), lerp(10, LOSS.b(), intensity))
                };
                let tile = egui::Rect::from_min_size(
                    egui::Pos2::new(rect.left() + mi as f32 * cell_w, rect.top() + si as f32 * cell_h),
                    Vec2::new(cell_w - 2.0_f32, cell_h - 2.0_f32),
                );
                painter.rect_filled(tile, 4.0_f32, color);
            }
        }
    }
    // ==================== ADVANCED ====================

    fn draw_multi_indicator(&self, ui: &mut egui::Ui) {
        use bt_analytics::{ema, rsi, sma};
        let series = &self.data.candles;
        let sma20 = sma(series, 20);
        let ema50 = ema(series, 50);
        let rsi_vals = rsi(series, 14);
        ui.label(RichText::new(format!("MULTI — Multi Indicator — {}", series.symbol)).strong());
        Plot::new("multi_price")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height() * 0.5_f32)
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().map(|c| [c.t, c.close]).collect();
                plot_ui.line(Line::new(pts).color(AMBER).width(1.5_f32).name("Price"));
                let sma_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !sma20[i].is_nan() { Some([c.t, sma20[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(sma_pts).color(INFO).width(1.5_f32).name("SMA20"));
                let ema_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !ema50[i].is_nan() { Some([c.t, ema50[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(ema_pts).color(PURPLE).width(1.5_f32).name("EMA50"));
            });
        Plot::new("multi_rsi")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !rsi_vals[i].is_nan() { Some([c.t, rsi_vals[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(pts).color(PROFIT).width(2.0_f32));
                plot_ui.hline(egui_plot::HLine::new(70.0).color(LOSS));
                plot_ui.hline(egui_plot::HLine::new(30.0).color(PROFIT));
            });
    }

    fn draw_multi_timeframe(&self, ui: &mut egui::Ui) {
        let series = &self.data.candles;
        ui.label(RichText::new(format!("MTF — Multi Timeframe — {}", series.symbol)).strong());
        Plot::new("mtf_daily")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height() * 0.33_f32)
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().map(|c| [c.t, c.close]).collect();
                plot_ui.line(Line::new(pts).color(AMBER).width(1.5_f32).name("Daily"));
            });
        Plot::new("mtf_weekly")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height() * 0.33_f32)
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let weekly: Vec<(f64, f64)> = series.candles.chunks(5)
                    .map(|chunk| (chunk[0].t, chunk.last().unwrap().close))
                    .collect();
                let pts: PlotPoints = weekly.iter().map(|&(t, c)| [t, c]).collect();
                plot_ui.line(Line::new(pts).color(INFO).width(2.0_f32).name("Weekly"));
            });
        Plot::new("mtf_monthly")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let monthly: Vec<(f64, f64)> = series.candles.chunks(20)
                    .map(|chunk| (chunk[0].t, chunk.last().unwrap().close))
                    .collect();
                let pts: PlotPoints = monthly.iter().map(|&(t, c)| [t, c]).collect();
                plot_ui.line(Line::new(pts).color(PURPLE).width(2.5_f32).name("Monthly"));
            });
    }

    fn draw_macd_divergence(&self, ui: &mut egui::Ui) {
        use bt_analytics::macd;
        let series = &self.data.candles;
        let (macd_line, _signal_line, _histogram) = macd(series);
        ui.label(RichText::new(format!("MACD-DIV — MACD Divergence — {}", series.symbol)).strong());
        Plot::new("macd_div_price")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height() * 0.5_f32)
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().map(|c| [c.t, c.close]).collect();
                plot_ui.line(Line::new(pts).color(AMBER).width(1.5_f32).name("Price"));
            });
        Plot::new("macd_div_macd")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !macd_line[i].is_nan() { Some([c.t, macd_line[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(pts).color(INFO).width(2.0_f32).name("MACD"));
                plot_ui.hline(egui_plot::HLine::new(0.0).color(Color32::GRAY));
            });
    }

    fn draw_bollinger_breakout(&self, ui: &mut egui::Ui) {
        use bt_analytics::bollinger;
        let series = &self.data.candles;
        let (_mid, upper, lower) = bollinger(series, 20, 2.0);
        ui.label(RichText::new(format!("BB-BO — Bollinger Breakout — {}", series.symbol)).strong());
        Plot::new("bb_bo_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().map(|c| [c.t, c.close]).collect();
                plot_ui.line(Line::new(pts).color(AMBER).width(1.5_f32).name("Price"));
                let upper_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !upper[i].is_nan() { Some([c.t, upper[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(upper_pts).color(LOSS).width(1.0_f32).name("Upper"));
                let lower_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !lower[i].is_nan() { Some([c.t, lower[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(lower_pts).color(PROFIT).width(1.0_f32).name("Lower"));
            });
    }

    fn draw_volume_weighted_scatter(&self, ui: &mut egui::Ui) {
        let candles = &self.data.candles;
        ui.label(RichText::new(format!("VWS — Volume-Price Scatter — {}", candles.symbol)).strong());
        Plot::new("vws_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let max_vol = candles.candles.iter().map(|c| c.volume).fold(0.0_f64, f64::max);
                for c in &candles.candles {
                    let color = if c.is_bullish() { PROFIT } else { LOSS };
                    let radius = 2.0_f64 + 6.0 * (c.volume / max_vol);
                    plot_ui.points(Points::new(PlotPoints::from(vec![[c.t, c.close]]))
                        .color(color).radius(radius as f32).shape(MarkerShape::Circle));
                }
            });
    }

    fn draw_price_momentum(&self, ui: &mut egui::Ui) {
        use bt_analytics::roc;
        let series = &self.data.candles;
        let roc_vals = roc(series, 10);
        ui.label(RichText::new(format!("PM — Price Momentum — {}", series.symbol)).strong());
        Plot::new("pm_price")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height() * 0.5_f32)
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().map(|c| [c.t, c.close]).collect();
                plot_ui.line(Line::new(pts).color(AMBER).width(1.5_f32).name("Price"));
            });
        Plot::new("pm_roc")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !roc_vals[i].is_nan() { Some([c.t, roc_vals[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(pts).color(INFO).width(2.0_f32).name("ROC"));
                plot_ui.hline(egui_plot::HLine::new(0.0).color(Color32::GRAY));
            });
    }

    fn draw_drawdown_recovery(&self, ui: &mut egui::Ui) {
        let dd = bt_viz::drawdown::compute_drawdown(&self.data.equity);
        ui.label(RichText::new("Drawdown & Recovery").strong());
        Plot::new("dd_rec_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let pts: PlotPoints = dd.iter().enumerate().map(|(i, &v)| [i as f64, v]).collect();
                plot_ui.line(Line::new(pts).color(LOSS).width(2.0_f32).name("Drawdown"));
                let recovery: PlotPoints = dd.iter().enumerate()
                    .filter_map(|(i, &v)| if v == 0.0 { Some([i as f64, 0.0]) } else { None })
                    .collect();
                plot_ui.points(Points::new(recovery).color(PROFIT).radius(5.0_f32).shape(MarkerShape::Circle).name("Recovery"));
            });
    }

    fn draw_rolling_correlation(&self, ui: &mut egui::Ui) {
        use bt_analytics::rolling_correlation;
        let rets = &self.data.returns;
        let rc = rolling_correlation(rets, rets, 20);
        ui.label(RichText::new("Rolling Correlation").strong());
        Plot::new("rc_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let pts: PlotPoints = rc.iter().enumerate()
                    .filter_map(|(i, &v)| if !v.is_nan() { Some([i as f64, v]) } else { None })
                    .collect();
                plot_ui.line(Line::new(pts).color(AMBER).width(2.0_f32));
                plot_ui.hline(egui_plot::HLine::new(0.0).color(Color32::GRAY));
            });
    }

    fn draw_tick_tape_adv(&self, ui: &mut egui::Ui) {
        let candles = &self.data.candles;
        ui.label(RichText::new(format!("TTA — Tick Tape Advanced — {}", candles.symbol)).strong());
        Plot::new("tta_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                for c in &candles.candles {
                    let color = if c.is_bullish() { PROFIT } else { LOSS };
                    plot_ui.points(Points::new(PlotPoints::from(vec![[c.t, c.close]]))
                        .color(color).radius(4.0_f32).shape(MarkerShape::Circle));
                }
            });
    }

    fn draw_seasonality_adv(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Advanced Seasonality").strong());
        let avail = ui.available_size();
        let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
        let cell_w = rect.width() / months.len() as f32;
        let cell_h = rect.height() / 4.0_f32;
        let metrics = ["Return", "Volume", "Volatility", "Drawdown"];
        for (mi, _metric) in metrics.iter().enumerate() {
            for (moi, _month) in months.iter().enumerate() {
                let val = ((mi * 7 + moi * 13) % 20) as f64 / 10.0 - 1.0;
                let intensity = val.abs().clamp(0.0, 1.0);
                let color = if val >= 0.0 {
                    Color32::from_rgb(lerp(10, PROFIT.r(), intensity), lerp(10, PROFIT.g(), intensity), lerp(10, PROFIT.b(), intensity))
                } else {
                    Color32::from_rgb(lerp(10, LOSS.r(), intensity), lerp(10, LOSS.g(), intensity), lerp(10, LOSS.b(), intensity))
                };
                let tile = egui::Rect::from_min_size(
                    egui::Pos2::new(rect.left() + moi as f32 * cell_w, rect.top() + mi as f32 * cell_h),
                    Vec2::new(cell_w - 2.0_f32, cell_h - 2.0_f32),
                );
                painter.rect_filled(tile, 4.0_f32, color);
            }
        }
    }

    fn draw_parabolic_sar(&self, ui: &mut egui::Ui) {
        use bt_analytics::parabolic_sar;
        let series = &self.data.candles;
        let sar = parabolic_sar(series, 0.02, 0.02, 0.2);
        ui.label(RichText::new(format!("PSAR — Parabolic SAR — {}", series.symbol)).strong());
        Plot::new("psar_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().map(|c| [c.t, c.close]).collect();
                plot_ui.line(Line::new(pts).color(AMBER).width(1.5_f32).name("Price"));
                let sar_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !sar[i].is_nan() { Some([c.t, sar[i]]) } else { None })
                    .collect();
                plot_ui.points(Points::new(sar_pts).color(INFO).radius(3.0_f32).shape(MarkerShape::Circle).name("SAR"));
            });
    }

    fn draw_macd_histogram(&self, ui: &mut egui::Ui) {
        use bt_analytics::macd;
        let series = &self.data.candles;
        let (_macd_line, _signal_line, histogram) = macd(series);
        ui.label(RichText::new(format!("MACD-H — MACD Histogram — {}", series.symbol)).strong());
        Plot::new("macd_h_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let bars: Vec<Bar> = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| {
                        if !histogram[i].is_nan() {
                            let color = if histogram[i] >= 0.0 { PROFIT } else { LOSS };
                            Some(Bar::new(c.t, histogram[i]).width(0.5 * DAY_SECS).fill(color))
                        } else { None }
                    }).collect();
                plot_ui.bar_chart(BarChart::new(bars));
                plot_ui.hline(egui_plot::HLine::new(0.0).color(Color32::GRAY));
            });
    }

    fn draw_rsi_heatmap(&self, ui: &mut egui::Ui) {
        use bt_analytics::rsi;
        let series = &self.data.candles;
        let rsi_vals = rsi(series, 14);
        ui.label(RichText::new(format!("RSI-H — RSI Heatmap — {}", series.symbol)).strong());
        let avail = ui.available_size();
        let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let cell_w = rect.width() / rsi_vals.len() as f32;
        let cell_h = rect.height() / 3.0_f32;
        for (i, &val) in rsi_vals.iter().enumerate() {
            if val.is_nan() { continue; }
            let color = if val > 70.0 {
                LOSS
            } else if val < 30.0 {
                PROFIT
            } else {
                AMBER
            };
            for row in 0..3 {
                let tile = egui::Rect::from_min_size(
                    egui::Pos2::new(rect.left() + i as f32 * cell_w, rect.top() + row as f32 * cell_h),
                    Vec2::new(cell_w - 1.0_f32, cell_h - 1.0_f32),
                );
                painter.rect_filled(tile, 0.0, color);
            }
        }
    }

    fn draw_ichimoku_ema(&self, ui: &mut egui::Ui) {
        use bt_analytics::ema;
        let series = &self.data.candles;
        let ema9 = ema(series, 9);
        let ema26 = ema(series, 26);
        let ema52 = ema(series, 52);
        ui.label(RichText::new(format!("ICH-E — Ichimoku + EMA — {}", series.symbol)).strong());
        Plot::new("ich_ema_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().map(|c| [c.t, c.close]).collect();
                plot_ui.line(Line::new(pts).color(AMBER).width(1.5_f32).name("Price"));
                let ema9_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !ema9[i].is_nan() { Some([c.t, ema9[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(ema9_pts).color(PROFIT).width(1.5_f32).name("EMA9"));
                let ema26_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !ema26[i].is_nan() { Some([c.t, ema26[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(ema26_pts).color(LOSS).width(1.5_f32).name("EMA26"));
                let ema52_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !ema52[i].is_nan() { Some([c.t, ema52[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(ema52_pts).color(INFO).width(1.5_f32).name("EMA52"));
            });
    }

    fn draw_keltner_breakout(&self, ui: &mut egui::Ui) {
        use bt_analytics::atr;
        use bt_analytics::bollinger;
        let series = &self.data.candles;
        let (mid, _upper, _lower) = bollinger(series, 20, 2.0);
        let atr_vals = atr(series, 14);
        ui.label(RichText::new(format!("KEL-BO — Keltner Breakout — {}", series.symbol)).strong());
        Plot::new("kel_bo_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().map(|c| [c.t, c.close]).collect();
                plot_ui.line(Line::new(pts).color(AMBER).width(1.5_f32).name("Price"));
                let upper_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !mid[i].is_nan() && !atr_vals[i].is_nan() { Some([c.t, mid[i] + 2.0 * atr_vals[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(upper_pts).color(LOSS).width(1.0_f32).name("Upper"));
                let lower_pts: PlotPoints = series.candles.iter().enumerate()
                    .filter_map(|(i, c)| if !mid[i].is_nan() && !atr_vals[i].is_nan() { Some([c.t, mid[i] - 2.0 * atr_vals[i]]) } else { None })
                    .collect();
                plot_ui.line(Line::new(lower_pts).color(PROFIT).width(1.0_f32).name("Lower"));
            });
    }

    fn draw_donchian_breakout(&self, ui: &mut egui::Ui) {
        let series = &self.data.candles;
        ui.label(RichText::new(format!("DON-BO — Donchian Breakout — {}", series.symbol)).strong());
        Plot::new("don_bo_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .label_formatter(|_axis: &str, p: &egui_plot::PlotPoint| format_ts(p.x))
            .show(ui, |plot_ui| {
                let pts: PlotPoints = series.candles.iter().map(|c| [c.t, c.close]).collect();
                plot_ui.line(Line::new(pts).color(AMBER).width(1.5_f32).name("Price"));
            });
    }

    fn draw_copula_heatmap(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Copula Heatmap").strong());
        let avail = ui.available_size();
        let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let n = 10;
        let cell_w = rect.width() / n as f32;
        let cell_h = rect.height() / n as f32;
        for i in 0..n {
            for j in 0..n {
                let u = i as f64 / n as f64;
                let v = j as f64 / n as f64;
                let copula = u * v + 0.1 * (1.0 - u) * (1.0 - v);
                let intensity = copula.clamp(0.0, 1.0);
                let color = Color32::from_rgb(lerp(10, INFO.r(), intensity), lerp(10, INFO.g(), intensity), lerp(10, INFO.b(), intensity));
                let tile = egui::Rect::from_min_size(
                    egui::Pos2::new(rect.left() + j as f32 * cell_w, rect.top() + i as f32 * cell_h),
                    Vec2::new(cell_w - 1.0_f32, cell_h - 1.0_f32),
                );
                painter.rect_filled(tile, 0.0, color);
            }
        }
    }

    fn draw_correlation_network_adv(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Advanced Correlation Network").strong());
        let avail = ui.available_size();
        let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let nodes = ["RELIANCE", "TCS", "INFY", "HDFCBANK", "ICICIBANK", "ITC"];
        let n = nodes.len();
        let center = rect.center();
        let radius = rect.width().min(rect.height()) * 0.35_f32;
        let positions: Vec<egui::Pos2> = (0..n).map(|i| {
            let angle = i as f32 * std::f32::consts::TAU / n as f32 - std::f32::consts::FRAC_PI_2;
            center + Vec2::new(radius * angle.cos(), radius * angle.sin())
        }).collect();
        for i in 0..n {
            for j in (i + 1)..n {
                let corr = 0.3 + (i as f64 * 0.1 + j as f64 * 0.05) % 0.5;
                let color = if corr > 0.5 { PROFIT } else { INFO };
                painter.line_segment([positions[i], positions[j]], Stroke::new(2.0_f32, color));
            }
        }
        for (i, pos) in positions.iter().enumerate() {
            painter.circle_filled(*pos, 15.0, AMBER);
            painter.text(*pos, egui::Align2::CENTER_CENTER, nodes[i], egui::FontId::monospace(8.0_f32), Color32::BLACK);
        }
    }

    fn refresh_market_data(&mut self) {
        // Production: bt_data::DataService::fetch_quote() per symbol.
        // 78+ individual HTTP calls are too slow for a 30s refresh cycle,
        // so the demo regenerates sample data instead.
        self.market_quotes = sample_market_data();
        self.market_last_refresh = Some(Instant::now());
    }

    fn draw_market_watch(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("MarketWatch — World Equity Indices (WEI)").strong());
        ui.horizontal(|ui| {
            ui.label(RichText::new("Search:").color(Color32::GRAY));
            ui.add(egui::TextEdit::singleline(&mut self.market_search).desired_width(220.0_f32));
            ui.separator();
            let ago = self.market_last_refresh.map(|t| t.elapsed().as_secs()).unwrap_or(0);
            ui.label(RichText::new(format!("Last refresh: {}s ago", ago)).color(Color32::GRAY));
            if self.live {
                ui.colored_label(PROFIT, "Auto-refresh: 30s");
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new("Sample data — production uses DataService::fetch_quote()")
                    .color(Color32::GRAY).weak());
            });
        });
        ui.separator();

        let query = self.market_search.to_lowercase();
        let (sort_col, sort_asc) = (self.market_sort_col, self.market_sort_asc);
        let mut rows: Vec<MarketQuote> = self.market_quotes.iter()
            .filter(|q| query.is_empty()
                || q.symbol.to_lowercase().contains(&query)
                || q.name.to_lowercase().contains(&query))
            .cloned()
            .collect();

        rows.sort_by(|a, b| {
            let ord = match sort_col {
                MarketSortCol::Symbol => a.symbol.cmp(&b.symbol),
                MarketSortCol::Name => a.name.cmp(&b.name),
                MarketSortCol::Price => a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal),
                MarketSortCol::Change => a.change.partial_cmp(&b.change).unwrap_or(std::cmp::Ordering::Equal),
                MarketSortCol::ChangePct => a.change_pct.partial_cmp(&b.change_pct).unwrap_or(std::cmp::Ordering::Equal),
                MarketSortCol::Volume => a.volume.cmp(&b.volume),
                MarketSortCol::MarketCap => a.market_cap.partial_cmp(&b.market_cap).unwrap_or(std::cmp::Ordering::Equal),
            };
            if sort_asc { ord } else { ord.reverse() }
        });

        let mut header_btn = |ui: &mut egui::Ui, col: MarketSortCol, label: &str| {
            let active = self.market_sort_col == col;
            let arrow = if active { if self.market_sort_asc { " \u{25B2}" } else { " \u{25BC}" } } else { "" };
            let text = RichText::new(format!("{}{}", label, arrow))
                .color(if active { AMBER } else { Color32::WHITE }).strong();
            if ui.button(text).clicked() {
                if self.market_sort_col == col {
                    self.market_sort_asc = !self.market_sort_asc;
                } else {
                    self.market_sort_col = col;
                    self.market_sort_asc = col != MarketSortCol::MarketCap;
                }
            }
        };

        egui::Grid::new("market_watch_table")
            .striped(true)
            .spacing(Vec2::new(10.0_f32, 2.0_f32))
            .min_col_width(70.0_f32)
            .show(ui, |ui| {
                header_btn(ui, MarketSortCol::Symbol, "Symbol");
                header_btn(ui, MarketSortCol::Name, "Name");
                header_btn(ui, MarketSortCol::Price, "Price");
                header_btn(ui, MarketSortCol::Change, "Change");
                header_btn(ui, MarketSortCol::ChangePct, "Change%");
                header_btn(ui, MarketSortCol::Volume, "Volume");
                header_btn(ui, MarketSortCol::MarketCap, "Market Cap");
                ui.end_row();

                for q in &rows {
                    ui.label(RichText::new(&q.symbol).monospace().strong().size(12.0_f32));
                    ui.label(RichText::new(&q.name).size(11.0_f32));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("{:.2}", q.price)).monospace().size(12.0_f32));
                    });
                    let color = if q.change >= 0.0 { PROFIT } else { LOSS };
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.colored_label(color, RichText::new(format!("{:+.2}", q.change)).monospace().size(12.0_f32));
                    });
                    let color = if q.change_pct >= 0.0 { PROFIT } else { LOSS };
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.colored_label(color, RichText::new(format!("{:+.2}%", q.change_pct)).monospace().size(12.0_f32));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(fmt_big_num(q.volume as f64)).monospace().size(12.0_f32));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(fmt_big_num(q.market_cap)).monospace().size(12.0_f32));
                    });
                    ui.end_row();
                }
            });
    }

    // ==================== INDIA-SPECIFIC (NEW) ====================

    fn draw_fo_chain(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("FOChain — NSE F&O Options Chain (NIFTY 50)").strong());
        let spot = 24_842.10_f64;
        let chain = india::sample_options_chain(spot);
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Spot: {:.2}", spot)).color(AMBER).monospace());
            ui.separator();
            ui.label(RichText::new("Sample data — production uses NSE F&O feed").color(Color32::GRAY).weak());
        });
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("fo_chain_table")
                .striped(true)
                .spacing(Vec2::new(8.0_f32, 2.0_f32))
                .min_col_width(62.0_f32)
                .show(ui, |ui| {
                    for h in ["Strike", "Call OI", "Chg OI", "C IV", "Delta", "Gamma", "Theta", "Vega",
                              "Put OI", "Chg OI", "P IV", "Delta", "Gamma", "Theta", "Vega"] {
                        ui.label(RichText::new(h).strong().monospace().size(11.0_f32));
                    }
                    ui.end_row();
                    for e in &chain {
                        ui.label(RichText::new(format!("{:.0}", e.strike)).monospace().size(11.0_f32));
                        ui.label(RichText::new(fmt_big_num(e.call_oi as f64)).monospace().size(11.0_f32));
                        let c = if e.call_chg_oi >= 0 { PROFIT } else { LOSS };
                        ui.colored_label(c, RichText::new(format!("{:+}", e.call_chg_oi)).monospace().size(11.0_f32));
                        ui.label(RichText::new(format!("{:.2}%", e.call_iv * 100.0)).monospace().size(11.0_f32));
                        ui.label(RichText::new(format!("{:.3}", e.call_delta)).monospace().size(11.0_f32));
                        ui.label(RichText::new(format!("{:.4}", e.call_gamma)).monospace().size(11.0_f32));
                        ui.colored_label(LOSS, RichText::new(format!("{:.3}", e.call_theta)).monospace().size(11.0_f32));
                        ui.label(RichText::new(format!("{:.3}", e.call_vega)).monospace().size(11.0_f32));
                        ui.label(RichText::new(fmt_big_num(e.put_oi as f64)).monospace().size(11.0_f32));
                        let c = if e.put_chg_oi >= 0 { PROFIT } else { LOSS };
                        ui.colored_label(c, RichText::new(format!("{:+}", e.put_chg_oi)).monospace().size(11.0_f32));
                        ui.label(RichText::new(format!("{:.2}%", e.put_iv * 100.0)).monospace().size(11.0_f32));
                        ui.colored_label(LOSS, RichText::new(format!("{:.3}", e.put_delta)).monospace().size(11.0_f32));
                        ui.label(RichText::new(format!("{:.4}", e.put_gamma)).monospace().size(11.0_f32));
                        ui.colored_label(LOSS, RichText::new(format!("{:.3}", e.put_theta)).monospace().size(11.0_f32));
                        ui.label(RichText::new(format!("{:.3}", e.put_vega)).monospace().size(11.0_f32));
                        ui.end_row();
                    }
                });
        });
    }

    fn draw_iv_surface_india(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("IV Surface — NIFTY Implied Volatility").strong());
        let surf = india::sample_iv_surface();
        let avail = ui.available_size();
        let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let label_w = 46.0_f32;
        let label_h = 20.0_f32;
        let grid_rect = egui::Rect::from_min_size(
            egui::Pos2::new(rect.left() + label_w, rect.top()),
            Vec2::new(rect.width() - label_w, rect.height() - label_h),
        );
        let cell_w = grid_rect.width() / surf.strikes.len() as f32;
        let cell_h = grid_rect.height() / surf.tenors.len() as f32;
        let mut max_iv = 0.0_f64;
        for row in &surf.ivs {
            for &v in row {
                max_iv = max_iv.max(v);
            }
        }
        for (ti, &tenor) in surf.tenors.iter().enumerate() {
            for (si, &strike) in surf.strikes.iter().enumerate() {
                let iv = surf.ivs[ti][si];
                let intensity = (iv / max_iv).clamp(0.0, 1.0);
                let color = Color32::from_rgb(
                    lerp(10, AMBER.r(), intensity),
                    lerp(10, AMBER.g(), intensity),
                    lerp(10, AMBER.b(), intensity),
                );
                let tile = egui::Rect::from_min_size(
                    egui::Pos2::new(grid_rect.left() + si as f32 * cell_w, grid_rect.top() + ti as f32 * cell_h),
                    Vec2::new(cell_w - 2.0_f32, cell_h - 2.0_f32),
                );
                painter.rect_filled(tile, 4.0_f32, color);
                painter.text(tile.center(), egui::Align2::CENTER_CENTER,
                    format!("{:.1}%", iv * 100.0), egui::FontId::monospace(11.0_f32), Color32::WHITE);
            }
        }
        for (si, &strike) in surf.strikes.iter().enumerate() {
            painter.text(
                egui::Pos2::new(grid_rect.left() + si as f32 * cell_w + cell_w / 2.0_f32, grid_rect.bottom() + 2.0_f32),
                egui::Align2::CENTER_TOP, format!("{:.0}%", strike * 100.0),
                egui::FontId::monospace(9.0_f32), Color32::GRAY,
            );
        }
        for (ti, &tenor) in surf.tenors.iter().enumerate() {
            painter.text(
                egui::Pos2::new(rect.left() + 2.0_f32, grid_rect.top() + ti as f32 * cell_h + cell_h / 2.0_f32),
                egui::Align2::LEFT_CENTER, format!("{}D", tenor as i32),
                egui::FontId::monospace(9.0_f32), Color32::GRAY,
            );
        }
    }

    fn draw_oi_heatmap(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("OI Heatmap — F&O Open Interest (lakh contracts)").strong());
        let hm = india::sample_oi_heatmap();
        let avail = ui.available_size();
        let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let label_w = 90.0_f32;
        let label_h = 20.0_f32;
        let grid_rect = egui::Rect::from_min_size(
            egui::Pos2::new(rect.left() + label_w, rect.top()),
            Vec2::new(rect.width() - label_w, rect.height() - label_h),
        );
        let cell_w = grid_rect.width() / hm.strikes.len() as f32;
        let cell_h = grid_rect.height() / hm.underlyings.len() as f32;
        let mut max_oi = 0.0_f64;
        for row in &hm.oi {
            for &v in row {
                max_oi = max_oi.max(v);
            }
        }
        for (ui_idx, name) in hm.underlyings.iter().enumerate() {
            for (si, &_strike) in hm.strikes.iter().enumerate() {
                let oi = hm.oi[ui_idx][si];
                let intensity = (oi / max_oi).clamp(0.0, 1.0);
                let color = Color32::from_rgb(
                    lerp(10, INFO.r(), intensity),
                    lerp(10, INFO.g(), intensity),
                    lerp(10, INFO.b(), intensity),
                );
                let tile = egui::Rect::from_min_size(
                    egui::Pos2::new(grid_rect.left() + si as f32 * cell_w, grid_rect.top() + ui_idx as f32 * cell_h),
                    Vec2::new(cell_w - 2.0_f32, cell_h - 2.0_f32),
                );
                painter.rect_filled(tile, 4.0_f32, color);
                if cell_w > 40.0 && cell_h > 18.0 {
                    painter.text(tile.center(), egui::Align2::CENTER_CENTER,
                        format!("{:.1}", oi), egui::FontId::monospace(10.0_f32), Color32::WHITE);
                }
            }
            painter.text(
                egui::Pos2::new(rect.left() + 4.0_f32, grid_rect.top() + ui_idx as f32 * cell_h + cell_h / 2.0_f32),
                egui::Align2::LEFT_CENTER, name.as_str(),
                egui::FontId::monospace(10.0_f32), Color32::WHITE,
            );
        }
        for (si, &strike) in hm.strikes.iter().enumerate() {
            painter.text(
                egui::Pos2::new(grid_rect.left() + si as f32 * cell_w + cell_w / 2.0_f32, grid_rect.bottom() + 2.0_f32),
                egui::Align2::CENTER_TOP, format!("{:.0}%", strike * 100.0),
                egui::FontId::monospace(9.0_f32), Color32::GRAY,
            );
        }
    }

    fn draw_gsec(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("GSec — Government Securities Yield Curve").strong());
        let curve = india::sample_gsec_curve();
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("10Y: {:.2}%", curve.yields[7])).color(AMBER).monospace());
            ui.separator();
            ui.label(RichText::new(format!("30Y: {:.2}%", curve.yields[10])).color(AMBER).monospace());
            ui.separator();
            let slope = curve.yields[7] - curve.yields[0];
            let c = if slope >= 0.0 { PROFIT } else { LOSS };
            ui.colored_label(c, RichText::new(format!("10Y-3M Slope: {:+.0}bp", slope * 100.0)).monospace());
        });
        ui.separator();
        Plot::new("gsec_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .show(ui, |plot_ui| {
                let pts: PlotPoints = curve.tenors.iter().zip(curve.yields.iter()).map(|(&t, &y)| [t, y]).collect();
                plot_ui.line(Line::new(pts).color(AMBER).width(2.0_f32).name("G-Sec"));
                let pts2: PlotPoints = curve.tenors.iter().zip(curve.yields.iter()).map(|(&t, &y)| [t, y]).collect();
                plot_ui.points(Points::new(pts2).color(AMBER).radius(3.5_f32).shape(MarkerShape::Circle));
            });
    }

    fn draw_money_market(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Money Market — MIBOR, TREPS, CP/CD Rates").strong());
        let mm = india::sample_money_market();
        let rates = vec![
            ("MIBOR 1D", mm.mibor_1d, 2.0),
            ("MIBOR 1W", mm.mibor_1w, 3.0),
            ("MIBOR 1M", mm.mibor_1m, 5.0),
            ("MIBOR 3M", mm.mibor_3m, 8.0),
            ("TREPS", mm.treps, -2.0),
            ("CBLO", mm.cblo, -3.0),
            ("T-Bill 91D", mm.t_bill_91, 1.0),
            ("T-Bill 182D", mm.t_bill_182, 4.0),
            ("T-Bill 364D", mm.t_bill_364, 6.0),
            ("CP 3M", mm.cp_3m, 10.0),
            ("CD 3M", mm.cd_3m, 7.0),
            ("Reverse Repo (SDF)", mm.reverse_repo, -25.0),
            ("MSF", mm.msf, 0.0),
        ];
        egui::Grid::new("money_market_grid")
            .striped(true)
            .spacing(Vec2::new(10.0_f32, 2.0_f32))
            .min_col_width(140.0_f32)
            .show(ui, |ui| {
                ui.label(RichText::new("Instrument").strong());
                ui.label(RichText::new("Rate").strong());
                ui.label(RichText::new("Chg (bp)").strong());
                ui.end_row();
                for (name, val, chg) in &rates {
                    ui.label(RichText::new(*name).monospace().size(12.0_f32));
                    ui.label(RichText::new(format!("{:.2}%", val)).monospace().size(12.0_f32));
                    let c = if *chg >= 0.0 { PROFIT } else { LOSS };
                    ui.colored_label(c, RichText::new(format!("{:+.0}", chg)).monospace().size(12.0_f32));
                    ui.end_row();
                }
            });
    }

    fn draw_rbi_policy(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("RBI Policy — Monetary Policy Dashboard").strong());
        let p = india::sample_rbi_policy();
        ui.horizontal(|ui| {
            ui.colored_label(AMBER, RichText::new(format!("Repo: {:.2}%", p.repo_rate)).strong().size(16.0_f32));
            ui.separator();
            ui.label(RichText::new(format!("Stance: {}", p.stance)).strong());
            ui.separator();
            ui.label(RichText::new(format!("Last MPC: {}", p.last_meeting)));
            ui.separator();
            ui.label(RichText::new(format!("Next MPC: {}", p.next_meeting)));
        });
        ui.separator();
        egui::Grid::new("rbi_policy_grid")
            .striped(true)
            .spacing(Vec2::new(10.0_f32, 2.0_f32))
            .min_col_width(150.0_f32)
            .show(ui, |ui| {
                for (name, val) in [
                    ("Repo Rate", format!("{:.2}%", p.repo_rate)),
                    ("Reverse Repo (SDF)", format!("{:.2}%", p.reverse_repo)),
                    ("MSF / Bank Rate", format!("{:.2}%", p.msf)),
                    ("CRR", format!("{:.2}%", p.crr)),
                    ("SLR", format!("{:.2}%", p.slr)),
                    ("Inflation Target", format!("{:.1}% +/-{:.1}%", p.inflation_target, p.inflation_tolerance)),
                    ("Real Rate (est.)", format!("{:.2}%", p.real_rate)),
                    ("Liquidity Stance", p.liquidity_modality.to_string()),
                ] {
                    ui.label(RichText::new(name).strong().size(12.0_f32));
                    ui.label(RichText::new(val).monospace().size(12.0_f32));
                    ui.end_row();
                }
            });
    }

    fn draw_macro_india(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Macro India — CPI, WPI, IIP, GDP, PMI").strong());
        let indicators = india::sample_macro_indicators();
        egui::Grid::new("macro_india_grid")
            .striped(true)
            .spacing(Vec2::new(10.0_f32, 2.0_f32))
            .min_col_width(130.0_f32)
            .show(ui, |ui| {
                for h in ["Indicator", "Value", "YoY", "Prev", "Freq"] {
                    ui.label(RichText::new(h).strong());
                }
                ui.end_row();
                for ind in &indicators {
                    ui.label(RichText::new(ind.name).size(12.0_f32));
                    let val_str = match ind.unit {
                        "idx" => format!("{:.1}", ind.value),
                        "% yoy" => format!("{:.1}%", ind.value),
                        "% GDP" => format!("{:.1}%", ind.value),
                        "USD bn" => format!("{:.1} bn", ind.value),
                        "%" => format!("{:.1}%", ind.value),
                        _ => format!("{:.1}{}", ind.value, ind.unit),
                    };
                    ui.label(RichText::new(val_str).monospace().size(12.0_f32));
                    let good_down = matches!(ind.name, "Trade Deficit" | "Fiscal Deficit" | "CAD" | "Unemployment");
                    let up = ind.yoy >= 0.0;
                    let c = if (up && !good_down) || (!up && good_down) { PROFIT } else { LOSS };
                    ui.colored_label(c, RichText::new(format!("{:+.1}%", ind.yoy)).monospace().size(12.0_f32));
                    ui.label(RichText::new(format!("{:.1}", ind.prev)).monospace().size(11.0_f32));
                    ui.label(RichText::new(ind.frequency).size(11.0_f32));
                    ui.end_row();
                }
            });
    }

    fn draw_commodities_india(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("MCX / NCDEX — Commodity Dashboard").strong());
        let quotes = india::sample_commodities();
        egui::Grid::new("commodities_grid")
            .striped(true)
            .spacing(Vec2::new(10.0_f32, 2.0_f32))
            .min_col_width(110.0_f32)
            .show(ui, |ui| {
                for h in ["Commodity", "Exch", "Price", "Unit", "Chg%", "Lot"] {
                    ui.label(RichText::new(h).strong());
                }
                ui.end_row();
                for q in &quotes {
                    ui.label(RichText::new(q.name).size(12.0_f32));
                    ui.label(RichText::new(q.exchange).color(INFO).monospace().size(11.0_f32));
                    ui.label(RichText::new(format!("{:.1}", q.price)).monospace().size(12.0_f32));
                    ui.label(RichText::new(q.unit).size(11.0_f32));
                    let c = if q.change_pct >= 0.0 { PROFIT } else { LOSS };
                    ui.colored_label(c, RichText::new(format!("{:+.2}%", q.change_pct)).monospace().size(12.0_f32));
                    ui.label(RichText::new(q.lot_size.to_string()).monospace().size(11.0_f32));
                    ui.end_row();
                }
            });
    }

    fn draw_usdinr_curve(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("USD/INR — Spot & Forward Curve").strong());
        let fwd = india::sample_usdinr_forward();
        Plot::new("usdinr_fwd_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height() * 0.55_f32)
            .show(ui, |plot_ui| {
                let pts: PlotPoints = fwd.tenors.iter().zip(fwd.outright.iter()).map(|(&t, &v)| [t, v]).collect();
                plot_ui.line(Line::new(pts).color(AMBER).width(2.0_f32).name("Outright"));
                let pts2: PlotPoints = fwd.tenors.iter().zip(fwd.outright.iter()).map(|(&t, &v)| [t, v]).collect();
                plot_ui.points(Points::new(pts2).color(AMBER).radius(3.5_f32).shape(MarkerShape::Circle));
            });
        egui::Grid::new("usdinr_fwd_grid")
            .striped(true)
            .spacing(Vec2::new(10.0_f32, 2.0_f32))
            .min_col_width(90.0_f32)
            .show(ui, |ui| {
                for h in ["Tenor (days)", "Fwd Points", "Outright"] {
                    ui.label(RichText::new(h).strong());
                }
                ui.end_row();
                for i in 0..fwd.tenors.len() {
                    ui.label(RichText::new(format!("{:.0}", fwd.tenors[i])).monospace().size(11.0_f32));
                    ui.label(RichText::new(format!("{:.0}", fwd.forward_points[i])).monospace().size(11.0_f32));
                    ui.label(RichText::new(format!("{:.2}", fwd.outright[i])).monospace().size(11.0_f32));
                    ui.end_row();
                }
            });
    }

    fn draw_yield_india(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Yield India — Sovereign, SDL, Corporate Curves").strong());
        let y = india::sample_yield_india();
        Plot::new("yield_india_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                let series = [
                    ("Sovereign", &y.sovereign, AMBER),
                    ("SDL", &y.sdl, INFO),
                    ("Corp AAA", &y.corporate_aaa, PROFIT),
                    ("Corp AA", &y.corporate_aa, PURPLE),
                ];
                for (name, vals, color) in series {
                    let pts: PlotPoints = y.tenors.iter().zip(vals.iter()).map(|(&t, &v)| [t, v]).collect();
                    plot_ui.line(Line::new(pts).color(color).width(2.0_f32).name(name));
                }
            });
    }

    fn draw_mf_analytics(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("MF Analytics — Mutual Fund Schemes").strong());
        let schemes = india::sample_mf_schemes();
        egui::Grid::new("mf_grid")
            .striped(true)
            .spacing(Vec2::new(8.0_f32, 2.0_f32))
            .min_col_width(70.0_f32)
            .show(ui, |ui| {
                for h in ["Scheme", "Category", "NAV", "AUM (cr)", "1Y", "3Y", "5Y", "Exp%", "Sharpe", "Beta", "Alpha", "SD%"] {
                    ui.label(RichText::new(h).strong().size(11.0_f32));
                }
                ui.end_row();
                for s in &schemes {
                    ui.label(RichText::new(s.name).size(11.0_f32));
                    ui.label(RichText::new(s.category).size(10.0_f32));
                    ui.label(RichText::new(format!("{:.2}", s.nav)).monospace().size(11.0_f32));
                    ui.label(RichText::new(format!("{:.0}", s.aum_cr)).monospace().size(11.0_f32));
                    for r in [s.ret_1y, s.ret_3y, s.ret_5y] {
                        let c = if r >= 0.0 { PROFIT } else { LOSS };
                        ui.colored_label(c, RichText::new(format!("{:+.1}%", r)).monospace().size(11.0_f32));
                    }
                    ui.label(RichText::new(format!("{:.2}", s.expense)).monospace().size(11.0_f32));
                    ui.label(RichText::new(format!("{:.2}", s.sharpe)).monospace().size(11.0_f32));
                    ui.label(RichText::new(format!("{:.2}", s.beta)).monospace().size(11.0_f32));
                    let c = if s.alpha >= 0.0 { PROFIT } else { LOSS };
                    ui.colored_label(c, RichText::new(format!("{:+.1}", s.alpha)).monospace().size(11.0_f32));
                    ui.label(RichText::new(format!("{:.1}", s.std_dev)).monospace().size(11.0_f32));
                    ui.end_row();
                }
            });
    }

    fn draw_fpi_fii(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("FPI/FII — Flow Dashboard (INR crore)").strong());
        let flows = india::sample_fpi_fii_flows();
        let tot_fpi: f64 = flows.iter().map(|f| f.fpi_equity + f.fpi_debt).sum();
        let tot_dii: f64 = flows.iter().map(|f| f.dii).sum();
        ui.horizontal(|ui| {
            let c = if tot_fpi >= 0.0 { PROFIT } else { LOSS };
            ui.colored_label(c, RichText::new(format!("Cum FPI: {:+.0} cr", tot_fpi)).strong().monospace());
            ui.separator();
            let c = if tot_dii >= 0.0 { PROFIT } else { LOSS };
            ui.colored_label(c, RichText::new(format!("Cum DII: {:+.0} cr", tot_dii)).strong().monospace());
        });
        ui.separator();
        Plot::new("fpi_fii_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height() * 0.45_f32)
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                let mut cum = 0.0_f64;
                let pts: PlotPoints = flows.iter().enumerate().map(|(i, f)| {
                    cum += f.fpi_equity;
                    [i as f64, cum]
                }).collect();
                plot_ui.line(Line::new(pts).color(LOSS).width(2.0_f32).name("Cum FPI Equity"));
                let dii_pts: PlotPoints = flows.iter().enumerate().map(|(i, f)| [i as f64, f.dii]).collect();
                plot_ui.line(Line::new(dii_pts).color(PROFIT).width(2.0_f32).name("DII"));
            });
        egui::Grid::new("fpi_fii_grid")
            .striped(true)
            .spacing(Vec2::new(10.0_f32, 2.0_f32))
            .min_col_width(90.0_f32)
            .show(ui, |ui| {
                for h in ["Date", "FPI Eq", "FPI Debt", "FII Eq", "DII"] {
                    ui.label(RichText::new(h).strong());
                }
                ui.end_row();
                for f in &flows {
                    ui.label(RichText::new(f.date).monospace().size(11.0_f32));
                    let c = if f.fpi_equity >= 0.0 { PROFIT } else { LOSS };
                    ui.colored_label(c, RichText::new(format!("{:+.0}", f.fpi_equity)).monospace().size(11.0_f32));
                    let c = if f.fpi_debt >= 0.0 { PROFIT } else { LOSS };
                    ui.colored_label(c, RichText::new(format!("{:+.0}", f.fpi_debt)).monospace().size(11.0_f32));
                    let c = if f.fii_equity >= 0.0 { PROFIT } else { LOSS };
                    ui.colored_label(c, RichText::new(format!("{:+.0}", f.fii_equity)).monospace().size(11.0_f32));
                    let c = if f.dii >= 0.0 { PROFIT } else { LOSS };
                    ui.colored_label(c, RichText::new(format!("{:+.0}", f.dii)).monospace().size(11.0_f32));
                    ui.end_row();
                }
            });
    }

    fn draw_credit_ratings(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Credit Ratings — CRISIL / ICRA / CARE / IND").strong());
        let entries = india::sample_credit_ratings();
        egui::Grid::new("credit_ratings_grid")
            .striped(true)
            .spacing(Vec2::new(10.0_f32, 2.0_f32))
            .min_col_width(110.0_f32)
            .show(ui, |ui| {
                for h in ["Issuer", "Instrument", "Rating", "Outlook", "Agency", "Amount (cr)", "Action"] {
                    ui.label(RichText::new(h).strong());
                }
                ui.end_row();
                for e in &entries {
                    ui.label(RichText::new(e.issuer).size(12.0_f32));
                    ui.label(RichText::new(e.instrument).monospace().size(11.0_f32));
                    ui.label(RichText::new(e.rating).color(AMBER).strong().monospace().size(11.0_f32));
                    let c = match e.outlook {
                        "Positive" => PROFIT,
                        "Negative" => LOSS,
                        _ => Color32::GRAY,
                    };
                    ui.colored_label(c, RichText::new(e.outlook).size(11.0_f32));
                    ui.label(RichText::new(e.agency).size(11.0_f32));
                    ui.label(RichText::new(format!("{:.0}", e.amount_cr)).monospace().size(11.0_f32));
                    let c = match e.action {
                        "Upgrade" => PROFIT,
                        "Downgrade" => LOSS,
                        _ => Color32::GRAY,
                    };
                    ui.colored_label(c, RichText::new(e.action).size(11.0_f32));
                    ui.end_row();
                }
            });
    }

    fn draw_banking_india(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Banking India — System Indicators").strong());
        let inds = india::sample_banking_indicators();
        egui::Grid::new("banking_india_grid")
            .striped(true)
            .spacing(Vec2::new(10.0_f32, 2.0_f32))
            .min_col_width(150.0_f32)
            .show(ui, |ui| {
                for h in ["Indicator", "Value", "Prev", "Trend"] {
                    ui.label(RichText::new(h).strong());
                }
                ui.end_row();
                for b in &inds {
                    ui.label(RichText::new(b.name).size(12.0_f32));
                    ui.label(RichText::new(format!("{:.1}{}", b.value, b.unit)).monospace().size(12.0_f32));
                    ui.label(RichText::new(format!("{:.1}", b.prev)).monospace().size(11.0_f32));
                    let up = b.value > b.prev;
                    let good = match b.name {
                        "Gross NPA Ratio" | "Net NPA Ratio" | "CD Ratio" => !up,
                        _ => up,
                    };
                    let c = if good { PROFIT } else { LOSS };
                    ui.colored_label(c, RichText::new(if up { "\u{25B2}" } else { "\u{25BC}" }).monospace().size(12.0_f32));
                    ui.end_row();
                }
            });
    }

    fn draw_corp_actions(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Corp Actions — Corporate Actions Calendar").strong());
        let actions = india::sample_corp_actions();
        egui::Grid::new("corp_actions_grid")
            .striped(true)
            .spacing(Vec2::new(10.0_f32, 2.0_f32))
            .min_col_width(90.0_f32)
            .show(ui, |ui| {
                for h in ["Symbol", "Company", "Action", "Ex-Date", "Record", "Detail"] {
                    ui.label(RichText::new(h).strong());
                }
                ui.end_row();
                for a in &actions {
                    ui.label(RichText::new(a.symbol).monospace().strong().size(11.0_f32));
                    ui.label(RichText::new(a.company).size(11.0_f32));
                    let c = match a.action {
                        "Dividend" => PROFIT,
                        "Split" | "Bonus" => INFO,
                        "Rights" => AMBER,
                        _ => Color32::GRAY,
                    };
                    ui.colored_label(c, RichText::new(a.action).strong().size(11.0_f32));
                    ui.label(RichText::new(a.ex_date).monospace().size(11.0_f32));
                    ui.label(RichText::new(a.record_date).monospace().size(11.0_f32));
                    ui.label(RichText::new(a.detail).size(11.0_f32));
                    ui.end_row();
                }
            });
    }

    fn draw_ipo_pipeline(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("IPO Pipeline — Mainboard Tracker").strong());
        let ipos = india::sample_ipo_pipeline();
        egui::Grid::new("ipo_grid")
            .striped(true)
            .spacing(Vec2::new(10.0_f32, 2.0_f32))
            .min_col_width(100.0_f32)
            .show(ui, |ui| {
                for h in ["Company", "Sector", "Size (cr)", "Price Band", "Open", "Close", "Status", "GMP%"] {
                    ui.label(RichText::new(h).strong());
                }
                ui.end_row();
                for i in &ipos {
                    ui.label(RichText::new(i.company).size(12.0_f32));
                    ui.label(RichText::new(i.sector).size(11.0_f32));
                    ui.label(RichText::new(format!("{:.0}", i.issue_size_cr)).monospace().size(11.0_f32));
                    ui.label(RichText::new(i.price_band).monospace().size(11.0_f32));
                    ui.label(RichText::new(i.open_date).monospace().size(11.0_f32));
                    ui.label(RichText::new(i.close_date).monospace().size(11.0_f32));
                    let c = match i.status {
                        "Live" => PROFIT,
                        "Upcoming" => INFO,
                        _ => Color32::GRAY,
                    };
                    ui.colored_label(c, RichText::new(i.status).strong().size(11.0_f32));
                    let c = if i.gmp >= 0.0 { PROFIT } else { LOSS };
                    ui.colored_label(c, RichText::new(format!("{:+.1}%", i.gmp)).monospace().size(11.0_f32));
                    ui.end_row();
                }
            });
    }

    fn draw_india_breadth(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("India Breadth — NSE Market Breadth").strong());
        let b = india::sample_breadth();
        let adv_pct = b.advances as f64 / b.total as f64 * 100.0;
        ui.horizontal(|ui| {
            ui.colored_label(PROFIT, RichText::new(format!("Advances: {}", b.advances)).strong().monospace());
            ui.separator();
            ui.colored_label(LOSS, RichText::new(format!("Declines: {}", b.declines)).strong().monospace());
            ui.separator();
            ui.label(RichText::new(format!("Unchanged: {}", b.unchanged)).monospace());
            ui.separator();
            ui.label(RichText::new(format!("A/D Ratio: {:.2}", b.advances as f64 / b.declines as f64)).monospace());
        });
        ui.separator();
        let avail = ui.available_size();
        let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let bar_h = rect.height() / 4.0;
        let total = (b.advances + b.declines + b.unchanged) as f64;
        let adv_w = rect.width() as f64 * b.advances as f64 / total;
        let dec_w = rect.width() as f64 * b.declines as f64 / total;
        painter.rect_filled(egui::Rect::from_min_size(rect.min, Vec2::new(adv_w as f32, bar_h * 0.6_f32)), 2.0_f32, PROFIT);
        painter.rect_filled(egui::Rect::from_min_size(egui::Pos2::new(rect.min.x + adv_w as f32, rect.min.y), Vec2::new(dec_w as f32, bar_h * 0.6_f32)), 2.0_f32, LOSS);
        painter.text(egui::Pos2::new(rect.min.x + 4.0_f32, rect.min.y + bar_h * 0.6_f32 + 4.0_f32), egui::Align2::LEFT_TOP,
            format!("Advances {:.1}%", adv_pct), egui::FontId::monospace(11.0_f32), Color32::WHITE);
        let y2 = rect.min.y + bar_h;
        let hi_w = rect.width() as f64 * b.new_52w_highs as f64 / b.total as f64;
        let lo_w = rect.width() as f64 * b.new_52w_lows as f64 / b.total as f64;
        painter.rect_filled(egui::Rect::from_min_size(egui::Pos2::new(rect.min.x, y2), Vec2::new(hi_w as f32, bar_h * 0.6_f32)), 2.0_f32, PROFIT);
        painter.rect_filled(egui::Rect::from_min_size(egui::Pos2::new(rect.min.x + hi_w as f32, y2), Vec2::new(lo_w as f32, bar_h * 0.6_f32)), 2.0_f32, LOSS);
        painter.text(egui::Pos2::new(rect.min.x + 4.0_f32, y2 + bar_h * 0.6_f32 + 4.0_f32), egui::Align2::LEFT_TOP,
            format!("52W Highs: {}   52W Lows: {}", b.new_52w_highs, b.new_52w_lows), egui::FontId::monospace(11.0_f32), Color32::WHITE);
        let y3 = rect.min.y + 2.0_f32 * bar_h;
        let d50_w = rect.width() as f64 * b.above_50dma as f64 / b.total as f64;
        let d200_w = rect.width() as f64 * b.above_200dma as f64 / b.total as f64;
        painter.rect_filled(egui::Rect::from_min_size(egui::Pos2::new(rect.min.x, y3), Vec2::new(d50_w as f32, bar_h * 0.6_f32)), 2.0_f32, INFO);
        painter.rect_filled(egui::Rect::from_min_size(egui::Pos2::new(rect.min.x, y3 + bar_h * 0.6_f32), Vec2::new(d200_w as f32, bar_h * 0.6_f32)), 2.0_f32, AMBER);
        painter.text(egui::Pos2::new(rect.min.x + 4.0_f32, y3 + bar_h + 4.0_f32), egui::Align2::LEFT_TOP,
            format!("Above 50DMA: {} ({:.1}%)   Above 200DMA: {} ({:.1}%)",
                b.above_50dma, b.above_50dma as f64 / b.total as f64 * 100.0,
                b.above_200dma, b.above_200dma as f64 / b.total as f64 * 100.0),
            egui::FontId::monospace(11.0_f32), Color32::WHITE);
    }

    fn draw_sector_research(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Sector Research — Valuation & Momentum").strong());
        let sectors = india::sample_sector_research();
        egui::Grid::new("sector_research_grid")
            .striped(true)
            .spacing(Vec2::new(8.0_f32, 2.0_f32))
            .min_col_width(80.0_f32)
            .show(ui, |ui| {
                for h in ["Sector", "P/E", "P/B", "Div%", "EPS Gr%", "ROE%", "D/E", "1M", "3M", "1Y", "Outlook"] {
                    ui.label(RichText::new(h).strong().size(11.0_f32));
                }
                ui.end_row();
                for s in &sectors {
                    ui.label(RichText::new(s.sector).size(11.0_f32));
                    ui.label(RichText::new(format!("{:.1}", s.pe)).monospace().size(11.0_f32));
                    ui.label(RichText::new(format!("{:.1}", s.pb)).monospace().size(11.0_f32));
                    ui.label(RichText::new(format!("{:.1}", s.div_yield)).monospace().size(11.0_f32));
                    ui.label(RichText::new(format!("{:.1}", s.eps_growth)).monospace().size(11.0_f32));
                    ui.label(RichText::new(format!("{:.1}", s.roe)).monospace().size(11.0_f32));
                    ui.label(RichText::new(format!("{:.2}", s.debt_equity)).monospace().size(11.0_f32));
                    for m in [s.mom_1m, s.mom_3m, s.mom_1y] {
                        let c = if m >= 0.0 { PROFIT } else { LOSS };
                        ui.colored_label(c, RichText::new(format!("{:+.1}%", m)).monospace().size(11.0_f32));
                    }
                    let c = match s.outlook {
                        "Overweight" => PROFIT,
                        "Underweight" => LOSS,
                        _ => Color32::GRAY,
                    };
                    ui.colored_label(c, RichText::new(s.outlook).strong().size(11.0_f32));
                    ui.end_row();
                }
            });
    }

    fn draw_india_news(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("India News — Market News Feed").strong());
        let news = india::sample_india_news();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for n in &news {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(n.time).color(Color32::GRAY).monospace().size(11.0_f32));
                    ui.label(RichText::new(n.source).color(INFO).strong().size(11.0_f32));
                    ui.label(RichText::new(n.category).color(AMBER).monospace().size(10.0_f32));
                    let c = if n.sentiment >= 0.2 { PROFIT } else if n.sentiment <= -0.2 { LOSS } else { Color32::GRAY };
                    ui.colored_label(c, RichText::new(n.headline).size(11.0_f32));
                });
                ui.separator();
            }
        });
    }

    fn draw_regulatory(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Regulatory — SEBI / RBI Updates").strong());
        let updates = india::sample_regulatory_updates();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for u in &updates {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(u.date).color(Color32::GRAY).monospace().size(11.0_f32));
                    let rc = match u.regulator {
                        "SEBI" => INFO,
                        "RBI" => AMBER,
                        _ => PURPLE,
                    };
                    ui.colored_label(rc, RichText::new(u.regulator).strong().monospace().size(11.0_f32));
                    let ic = match u.impact {
                        "High" => LOSS,
                        "Medium" => AMBER,
                        _ => Color32::GRAY,
                    };
                    ui.colored_label(ic, RichText::new(format!("{} impact", u.impact)).size(10.0_f32));
                });
                ui.label(RichText::new(u.title).strong().size(12.0_f32));
                ui.label(RichText::new(u.summary).size(11.0_f32));
                ui.separator();
            }
        });
    }

    fn draw_gst_budget(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("GST & Budget — Collections & Fiscal Metrics").strong());
        let gst = india::sample_gst_collections();
        let latest = gst.last().unwrap();
        ui.horizontal(|ui| {
            ui.colored_label(AMBER, RichText::new(format!("{} GST: Rs {:.0} cr", latest.month, latest.gst_cr)).strong().monospace());
            ui.separator();
            let c = if latest.yoy >= 0.0 { PROFIT } else { LOSS };
            ui.colored_label(c, RichText::new(format!("YoY {:+.1}%", latest.yoy)).monospace());
        });
        ui.separator();
        Plot::new("gst_plot")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height() * 0.45_f32)
            .show(ui, |plot_ui| {
                let bars: Vec<Bar> = gst.iter().enumerate()
                    .map(|(i, g)| Bar::new(i as f64, g.gst_cr).width(0.6_f64).fill(AMBER))
                    .collect();
                plot_ui.bar_chart(BarChart::new(bars));
            });
        let metrics = india::sample_budget_metrics();
        egui::Grid::new("budget_grid")
            .striped(true)
            .spacing(Vec2::new(10.0_f32, 2.0_f32))
            .min_col_width(140.0_f32)
            .show(ui, |ui| {
                for h in ["Metric", "FY25", "FY26", "Unit", "Chg"] {
                    ui.label(RichText::new(h).strong());
                }
                ui.end_row();
                for m in &metrics {
                    ui.label(RichText::new(m.name).size(12.0_f32));
                    ui.label(RichText::new(format!("{:.1}", m.fy25)).monospace().size(12.0_f32));
                    ui.label(RichText::new(format!("{:.1}", m.fy26)).monospace().size(12.0_f32));
                    ui.label(RichText::new(m.unit).size(11.0_f32));
                    let chg = m.fy26 - m.fy25;
                    let good = match m.name {
                        "Fiscal Deficit" | "Interest Outgo" | "Subsidies" => chg <= 0.0,
                        _ => chg >= 0.0,
                    };
                    let c = if good { PROFIT } else { LOSS };
                    ui.colored_label(c, RichText::new(format!("{:+.1}", chg)).monospace().size(12.0_f32));
                    ui.end_row();
                }
            });
    }

    fn draw_india_portfolio(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("India Portfolio — Holdings & Tax Analytics").strong());
        let holdings = india::sample_india_portfolio();
        let tot_stcg: f64 = holdings.iter().map(|h| h.stcg).sum();
        let tot_ltcg: f64 = holdings.iter().map(|h| h.ltcg).sum();
        let tot_div: f64 = holdings.iter().map(|h| h.div_income).sum();
        let tot_tax: f64 = holdings.iter().map(|h| h.tax_liability).sum();
        ui.horizontal(|ui| {
            ui.colored_label(LOSS, RichText::new(format!("STCG: {:+.0}", tot_stcg)).monospace());
            ui.separator();
            ui.colored_label(LOSS, RichText::new(format!("LTCG: {:+.0}", tot_ltcg)).monospace());
            ui.separator();
            ui.label(RichText::new(format!("Div Income: {:.0}", tot_div)).monospace());
            ui.separator();
            ui.colored_label(AMBER, RichText::new(format!("Tax: {:.0}", tot_tax)).strong().monospace());
        });
        ui.separator();
        egui::Grid::new("india_portfolio_grid")
            .striped(true)
            .spacing(Vec2::new(8.0_f32, 2.0_f32))
            .min_col_width(80.0_f32)
            .show(ui, |ui| {
                for h in ["Symbol", "Name", "Qty", "Avg", "LTP", "STCG", "LTCG", "Div", "Tax"] {
                    ui.label(RichText::new(h).strong().size(11.0_f32));
                }
                ui.end_row();
                for h in &holdings {
                    ui.label(RichText::new(h.symbol).monospace().strong().size(11.0_f32));
                    ui.label(RichText::new(h.name).size(11.0_f32));
                    ui.label(RichText::new(format!("{:.0}", h.qty)).monospace().size(11.0_f32));
                    ui.label(RichText::new(format!("{:.0}", h.avg_price)).monospace().size(11.0_f32));
                    ui.label(RichText::new(format!("{:.2}", h.ltp)).monospace().size(11.0_f32));
                    let c = if h.stcg >= 0.0 { PROFIT } else { LOSS };
                    ui.colored_label(c, RichText::new(format!("{:+.0}", h.stcg)).monospace().size(11.0_f32));
                    let c = if h.ltcg >= 0.0 { PROFIT } else { LOSS };
                    ui.colored_label(c, RichText::new(format!("{:+.0}", h.ltcg)).monospace().size(11.0_f32));
                    ui.label(RichText::new(format!("{:.0}", h.div_income)).monospace().size(11.0_f32));
                    ui.colored_label(AMBER, RichText::new(format!("{:.0}", h.tax_liability)).monospace().size(11.0_f32));
                    ui.end_row();
                }
            });
    }

    fn draw_algo_feed(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Algo Feed — Market Data & Order Feed Status").strong());
        let feeds = india::sample_algo_feeds();
        egui::Grid::new("algo_feed_grid")
            .striped(true)
            .spacing(Vec2::new(10.0_f32, 2.0_f32))
            .min_col_width(130.0_f32)
            .show(ui, |ui| {
                for h in ["Feed", "Status", "Latency (ms)", "Msg/s", "Uptime%", "Last Error"] {
                    ui.label(RichText::new(h).strong());
                }
                ui.end_row();
                for f in &feeds {
                    ui.label(RichText::new(f.name).size(12.0_f32));
                    let c = match f.status {
                        "Connected" => PROFIT,
                        "Degraded" => AMBER,
                        _ => LOSS,
                    };
                    ui.colored_label(c, RichText::new(f.status).strong().size(12.0_f32));
                    let lc = if f.latency_ms < 50 { PROFIT } else if f.latency_ms < 200 { AMBER } else { LOSS };
                    ui.colored_label(lc, RichText::new(f.latency_ms.to_string()).monospace().size(12.0_f32));
                    ui.label(RichText::new(f.msgs_per_sec.to_string()).monospace().size(12.0_f32));
                    let uc = if f.uptime_pct >= 99.9 { PROFIT } else if f.uptime_pct >= 99.0 { AMBER } else { LOSS };
                    ui.colored_label(uc, RichText::new(format!("{:.2}", f.uptime_pct)).monospace().size(12.0_f32));
                    ui.label(RichText::new(f.last_error).size(11.0_f32));
                    ui.end_row();
                }
            });
    }

    fn draw_ai_research(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("AI Research — Summarized Research Notes").strong());
        let items = india::sample_ai_research();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for it in &items {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(it.date).color(Color32::GRAY).monospace().size(11.0_f32));
                    ui.label(RichText::new(it.source).color(INFO).strong().size(11.0_f32));
                    let c = if it.relevance >= 0.85 { PROFIT } else if it.relevance >= 0.7 { AMBER } else { Color32::GRAY };
                    ui.colored_label(c, RichText::new(format!("Relevance {:.0}%", it.relevance * 100.0)).monospace().size(11.0_f32));
                });
                ui.label(RichText::new(it.title).strong().size(13.0_f32));
                ui.label(RichText::new(it.summary).size(11.0_f32));
                ui.horizontal(|ui| {
                    for t in it.tags {
                        ui.colored_label(PURPLE, RichText::new(format!("#{}", t)).monospace().size(10.0_f32));
                    }
                });
                ui.separator();
            }
        });
    }

    fn draw_india_dashboard(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("India Dashboard — Customizable Widgets").strong());
        let widgets = india::sample_dashboard_widgets();
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("india_dashboard_grid")
                .spacing(Vec2::new(12.0_f32, 8.0_f32))
                .min_col_width(280.0_f32)
                .show(ui, |ui| {
                    for (i, w) in widgets.iter().enumerate() {
                        if i % 3 == 0 && i > 0 {
                            ui.end_row();
                        }
                        ui.label(RichText::new(w.title).strong().size(12.0_f32));
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(w.value).monospace().strong().size(14.0_f32));
                            let c = if w.change_pct >= 0.0 { PROFIT } else { LOSS };
                            ui.colored_label(c, RichText::new(format!("{:+.2}%", w.change_pct)).monospace().size(11.0_f32));
                        });
                        Plot::new(format!("dash_spark_{}", i))
                            .auto_bounds_x().auto_bounds_y()
                            .height(36.0_f32)
                            .show_axes([false, false])
                            .show_grid([false, false])
                            .show(ui, |plot_ui| {
                                let pts: PlotPoints = w.sparkline.iter().enumerate().map(|(j, &v)| [j as f64, v]).collect();
                                let c = if w.change_pct >= 0.0 { PROFIT } else { LOSS };
                                plot_ui.line(Line::new(pts).color(c).width(1.5_f32));
                            });
                    }
                    ui.end_row();
                });
            });
    }

    fn draw_multi_compare(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Multi-Compare — Side-by-Side Stock Comparison").strong());

        ui.horizontal(|ui| {
            ui.label(RichText::new("Add Symbol:").color(Color32::GRAY));
            ui.add(egui::TextEdit::singleline(&mut self.compare_search).desired_width(200.0_f32));
            if ui.button("Add").clicked() {
                let query = self.compare_search.to_lowercase();
                for (name, ticker, _exchange) in COMPANY_LIST {
                    if ticker.to_lowercase() == query || name.to_lowercase() == query {
                        if !self.compare_symbols.contains(&ticker.to_string()) && self.compare_symbols.len() < 5 {
                            self.compare_symbols.push(ticker.to_string());
                        }
                        self.compare_search.clear();
                        break;
                    }
                }
            }
            ui.separator();
            ui.label(RichText::new(format!("{} / 5 selected", self.compare_symbols.len())).color(Color32::GRAY));
        });

        ui.horizontal_wrapped(|ui| {
            let mut to_remove = Vec::new();
            for (i, sym) in self.compare_symbols.iter().enumerate() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(sym).monospace().strong().size(12.0_f32));
                        if ui.button("\u{00D7}").clicked() {
                            to_remove.push(i);
                        }
                    });
                });
            }
            for &i in to_remove.iter().rev() {
                self.compare_symbols.remove(i);
            }
        });

        ui.separator();

        if self.compare_symbols.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("Add at least 2 symbols to compare").color(Color32::GRAY));
            });
            return;
        }

        let colors = [AMBER, INFO, PROFIT, PURPLE, Color32::from_rgb(0xFF, 0x69, 0xB4)];

        egui::Grid::new("multi_compare_table")
            .striped(true)
            .spacing(Vec2::new(8.0_f32, 2.0_f32))
            .min_col_width(80.0_f32)
            .show(ui, |ui| {
                ui.label(RichText::new("Symbol").strong().size(11.0_f32));
                ui.label(RichText::new("Name").strong().size(11.0_f32));
                ui.label(RichText::new("Last Price").strong().size(11.0_f32));
                ui.label(RichText::new("Change%").strong().size(11.0_f32));
                ui.label(RichText::new("1M").strong().size(11.0_f32));
                ui.label(RichText::new("3M").strong().size(11.0_f32));
                ui.label(RichText::new("6M").strong().size(11.0_f32));
                ui.label(RichText::new("1Y").strong().size(11.0_f32));
                ui.label(RichText::new("Volatility").strong().size(11.0_f32));
                ui.label(RichText::new("Sharpe").strong().size(11.0_f32));
                ui.label(RichText::new("Max DD").strong().size(11.0_f32));
                ui.label(RichText::new("Beta").strong().size(11.0_f32));
                ui.label(RichText::new("Alpha").strong().size(11.0_f32));
                ui.end_row();

                for (i, sym) in self.compare_symbols.iter().enumerate() {
                    let seed = 100 + i as u64 * 7;
                    let series = synthetic_ohlcv(sym, 250, seed, 100.0 + i as f64 * 50.0);
                    let closes = series.closes();
                    let last_price = closes.last().copied().unwrap_or(0.0);
                    let first_price = closes.first().copied().unwrap_or(1.0);
                    let change_pct = (last_price - first_price) / first_price * 100.0;

                    let returns = series.returns();
                    let ret_1m = returns.iter().take(22).sum::<f64>() * 100.0;
                    let ret_3m = returns.iter().take(66).sum::<f64>() * 100.0;
                    let ret_6m = returns.iter().take(132).sum::<f64>() * 100.0;
                    let ret_1y = returns.iter().take(250).sum::<f64>() * 100.0;

                    let mean_ret = returns.iter().sum::<f64>() / returns.len().max(1) as f64;
                    let variance = returns.iter().map(|r| (r - mean_ret).powi(2)).sum::<f64>() / returns.len().max(1) as f64;
                    let volatility = variance.sqrt() * (252.0_f64).sqrt() * 100.0;

                    let sharpe = if volatility > 0.0 { (ret_1y / 100.0) / (volatility / 100.0) } else { 0.0 };

                    let mut peak = closes.first().copied().unwrap_or(1.0);
                    let mut max_dd = 0.0_f64;
                    for &c in &closes {
                        if c > peak { peak = c; }
                        let dd = (peak - c) / peak;
                        if dd > max_dd { max_dd = dd; }
                    }
                    let max_dd_pct = max_dd * 100.0;

                    let beta = 0.8 + (i as f64 * 0.15);
                    let alpha = (change_pct - 10.0) / 100.0;

                    let name = COMPANY_LIST.iter()
                        .find(|(_, t, _)| *t == sym.as_str())
                        .map(|(n, _, _)| *n)
                        .unwrap_or("Unknown");

                    let color = colors[i % colors.len()];
                    ui.colored_label(color, RichText::new(sym).monospace().strong().size(11.0_f32));
                    ui.label(RichText::new(name).size(10.0_f32));
                    ui.label(RichText::new(format!("{:.2}", last_price)).monospace().size(11.0_f32));

                    let chg_color = if change_pct >= 0.0 { PROFIT } else { LOSS };
                    ui.colored_label(chg_color, RichText::new(format!("{:+.2}%", change_pct)).monospace().size(11.0_f32));

                    for ret in [ret_1m, ret_3m, ret_6m, ret_1y] {
                        let c = if ret >= 0.0 { PROFIT } else { LOSS };
                        ui.colored_label(c, RichText::new(format!("{:+.1}%", ret)).monospace().size(11.0_f32));
                    }

                    ui.label(RichText::new(format!("{:.1}%", volatility)).monospace().size(11.0_f32));

                    let sharpe_color = if sharpe >= 0.0 { PROFIT } else { LOSS };
                    ui.colored_label(sharpe_color, RichText::new(format!("{:.2}", sharpe)).monospace().size(11.0_f32));

                    ui.colored_label(LOSS, RichText::new(format!("{:.1}%", max_dd_pct)).monospace().size(11.0_f32));
                    ui.label(RichText::new(format!("{:.2}", beta)).monospace().size(11.0_f32));

                    let alpha_color = if alpha >= 0.0 { PROFIT } else { LOSS };
                    ui.colored_label(alpha_color, RichText::new(format!("{:+.2}%", alpha * 100.0)).monospace().size(11.0_f32));

                    ui.end_row();
                }
            });

        ui.separator();

        ui.label(RichText::new("Normalized Price Performance (Base = 100)").strong());

        let chart_anchor = ui.allocate_exact_size(Vec2::new(1.0, 1.0), egui::Sense::hover());
        let (_rect, chart_resp) = chart_anchor;
        chart_resp.scroll_to_me(Some(egui::Align::Center));

        Plot::new("multi_compare_normalized")
            .auto_bounds_x().auto_bounds_y()
            .height(ui.available_height())
            .allow_scroll(true)
            .allow_drag(true)
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                for (i, sym) in self.compare_symbols.iter().enumerate() {
                    let seed = 100 + i as u64 * 7;
                    let series = synthetic_ohlcv(sym, 250, seed, 100.0 + i as f64 * 50.0);
                    let closes = series.closes();
                    let base = closes.first().copied().unwrap_or(1.0);
                    let pts: PlotPoints = closes.iter().enumerate()
                        .map(|(j, &c)| [j as f64, c / base * 100.0])
                        .collect();
                    let color = colors[i % colors.len()];
                    plot_ui.line(Line::new(pts).color(color).width(2.0_f32).name(sym));
                }
            });
    }
}

// ==================== HELPERS ====================

fn draw_lollipop(plot_ui: &mut egui_plot::PlotUi, values: &[f64], conf: f64) {
    plot_ui.polygon(
        egui_plot::Polygon::new(PlotPoints::from(vec![
            [0.0, -conf], [values.len() as f64, -conf],
            [values.len() as f64, conf], [0.0, conf],
        ]))
        .fill_color(INFO.gamma_multiply(0.15))
        .stroke(Stroke::NONE),
    );
    for (lag, &v) in values.iter().enumerate() {
        let color = if v.abs() > conf { AMBER } else { Color32::GRAY };
        plot_ui.line(
            Line::new(PlotPoints::from(vec![[lag as f64, 0.0], [lag as f64, v]]))
                .color(color).width(2.0_f32),
        );
        plot_ui.points(
            Points::new(PlotPoints::from(vec![[lag as f64, v]]))
                .color(color).radius(3.5_f32),
        );
    }
}

fn lerp(a: u8, b: u8, t: f64) -> u8 {
    (a as f64 + (b as f64 - a as f64) * t.clamp(0.0, 1.0)).round() as u8
}

fn fmt_big_num(v: f64) -> String {
    if v >= 1_000_000_000.0 {
        format!("{:.2}B", v / 1_000_000_000.0)
    } else if v >= 1_000_000.0 {
        format!("{:.2}M", v / 1_000_000.0)
    } else if v >= 1_000.0 {
        format!("{:.1}K", v / 1_000.0)
    } else {
        format!("{:.0}", v)
    }
}

fn normal_inverse(p: f64) -> f64 {
    let a = [-3.969683028665376e+01, 2.209460984245205e+02, -2.759285104469687e+02, 1.383577518672690e+02, -3.066479806614716e+01, 2.506628277459239e+00];
    let b = [-5.447609879822406e+01, 1.615858368580409e+02, -1.556989798598866e+02, 6.680131188771972e+01, -1.328068155288572e+01];
    let c = [-7.784894002430293e-03, -3.223964580411365e-01, -2.400758277161838e+00, -2.549732539343734e+00, 4.374664141464968e+00, 2.938163982698783e+00];
    let d = [7.784695709041462e-03, 3.224671290700398e-01, 2.445134137142996e+00, 3.754408661907416e+00];
    let p_low = 0.02425;
    let p_high = 1.0 - p_low;
    if p < p_low {
        let q = (-2.0 * p.ln()).sqrt();
        (((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5]) /
            ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    } else if p <= p_high {
        let q = p - 0.5;
        let r = q * q;
        (((((a[0] * r + a[1]) * r + a[2]) * r + a[3]) * r + a[4]) * r + a[5]) * q /
            (((((b[0] * r + b[1]) * r + b[2]) * r + b[3]) * r + b[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5]) /
            ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    }
}

impl eframe::App for BharatApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_messages();
        self.save_prefs();

        if self.live {
            if let Some(last) = self.last_fetch {
                if last.elapsed() >= Duration::from_secs(30) {
                    self.trigger_fetch();
                }
            } else if !self.fetch_in_flight {
                self.trigger_fetch();
            }
            if self.tab == Tab::MarketWatch {
                let stale = self.market_last_refresh
                    .map(|t| t.elapsed() >= Duration::from_secs(30))
                    .unwrap_or(true);
                if stale {
                    self.refresh_market_data();
                }
            }
        }

        ctx.set_visuals(if self.dark { egui::Visuals::dark() } else { egui::Visuals::light() });
        self.header(ctx);
        self.warning_banner(ctx);
        self.tab_bar(ctx);
        self.status_bar(ctx);
        self.error_toast(ctx);
        self.body(ctx);
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0_f32, 820.0_f32])
            .with_title(format!("{} — Made by {}", APP_NAME, AUTHOR)),
        ..Default::default()
    };
    eframe::run_native(
        &format!("{} v3 — Made by {}", APP_NAME, AUTHOR),
        options,
        Box::new(|cc| Ok(Box::new(BharatApp::new(cc)))),
    )
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefs_default() {
        let prefs = Prefs::default();
        assert!(prefs.live);
        assert_eq!(prefs.symbol, "RELIANCE.NS");
        assert_eq!(prefs.range, "1y");
        assert_eq!(prefs.theme, "dark");
    }

    #[test]
    fn test_time_range_labels() {
        assert_eq!(TimeRange::D1.label(), "1D");
        assert_eq!(TimeRange::Y5.label(), "5Y");
    }

    #[test]
    fn test_time_range_days() {
        assert_eq!(TimeRange::D1.to_days(), 1);
        assert_eq!(TimeRange::Y1.to_days(), 365);
        assert_eq!(TimeRange::Y5.to_days(), 1825);
    }

    #[test]
    fn test_banner_constants() {
        assert!(!APP_NAME.is_empty());
        assert!(!AUTHOR.is_empty());
        assert!(!TAGLINE.is_empty());
    }

    #[test]
    fn test_price_scale_uses_low_and_high() {
        let series = synthetic_ohlcv("SCALE", 10, 7, 100.0);
        let (lo, range) = price_scale(&series.candles);
        let expect_lo = series.candles.iter().map(|c| c.low).fold(f64::MAX, f64::min);
        let expect_hi = series.candles.iter().map(|c| c.high).fold(f64::MIN, f64::max);
        assert!((lo - expect_lo).abs() < 1e-9);
        assert!((range - (expect_hi - expect_lo)).abs() < 1e-9);
        assert!(range > 0.0);
    }

    #[test]
    fn test_price_scale_empty_series_is_safe() {
        let (lo, range) = price_scale(&[]);
        assert!(lo.is_finite());
        assert!(range > 0.0);
    }

    #[test]
    fn test_candle_body_bullish_is_close_on_top() {
        let c = Candle::new(1.0, 100.0, 110.0, 95.0, 105.0, 1_000.0);
        let (top, bottom) = candle_body(&c, 0.0);
        assert!((top - 105.0).abs() < 1e-9);
        assert!((bottom - 100.0).abs() < 1e-9);
    }

    #[test]
    fn test_candle_body_bearish_is_open_on_top() {
        let c = Candle::new(1.0, 105.0, 110.0, 95.0, 100.0, 1_000.0);
        let (top, bottom) = candle_body(&c, 0.0);
        assert!((top - 105.0).abs() < 1e-9);
        assert!((bottom - 100.0).abs() < 1e-9);
    }

    #[test]
    fn test_candle_body_doji_gets_visible_height() {
        let c = Candle::new(1.0, 100.0, 110.0, 95.0, 100.0, 1_000.0);
        let min_body = 2.0;
        let (top, bottom) = candle_body(&c, min_body);
        assert!(top > bottom, "doji body must remain visible");
        assert!(((top - bottom) - min_body).abs() < 1e-9);
        let mid = 0.5 * (top + bottom);
        assert!((mid - 100.0).abs() < 1e-9, "expansion stays centred on the price");
    }

    #[test]
    fn test_candle_body_never_collapses_for_tiny_ranges() {
        let series = synthetic_ohlcv("TINY", 50, 3, 10.0);
        let (_, range) = price_scale(&series.candles);
        let min_body = range * MIN_BODY_FRAC;
        for c in &series.candles {
            let (top, bottom) = candle_body(c, min_body);
            assert!(top > bottom, "every candle must have a positive body height");
            assert!(top.is_finite() && bottom.is_finite());
        }
    }

    #[test]
    fn test_arrow_and_body_fractions_are_sane() {
        assert!(MIN_BODY_FRAC > 0.0 && MIN_BODY_FRAC < 0.1);
        assert!(ARROW_FRAC > MIN_BODY_FRAC, "arrows must be taller than a body");
        assert!(ARROW_FRAC < 0.2, "arrows must stay small relative to the range");
    }

    #[test]
    fn test_heikin_ashi_seeds_first_open_from_raw_candle() {
        let c = Candle::new(1.0, 100.0, 110.0, 95.0, 105.0, 1_000.0);
        let ha = heikin_ashi(&[c]);
        assert_eq!(ha.len(), 1);
        assert!((ha[0].open - 102.5).abs() < 1e-9, "seed open is (o + c) / 2");
        assert!((ha[0].close - 102.5).abs() < 1e-9, "close is the 4-way average");
    }

    #[test]
    fn test_heikin_ashi_is_recursive() {
        let candles = vec![
            Candle::new(1.0, 100.0, 110.0, 95.0, 105.0, 1_000.0),
            Candle::new(2.0, 105.0, 115.0, 100.0, 110.0, 1_000.0),
        ];
        let ha = heikin_ashi(&candles);
        let first_close = (100.0 + 110.0 + 95.0 + 105.0) / 4.0;
        let first_open = (100.0 + 105.0) / 2.0;
        let expect_open = 0.5 * (first_open + first_close);
        assert!(
            (ha[1].open - expect_open).abs() < 1e-9,
            "second open must average the previous HA open and close"
        );
    }

    #[test]
    fn test_heikin_ashi_range_contains_open_and_close() {
        let series = synthetic_ohlcv("HA", 60, 11, 250.0);
        for (src, ha) in series.candles.iter().zip(heikin_ashi(&series.candles)) {
            assert!(ha.high >= ha.open.max(ha.close), "HA high must contain body");
            assert!(ha.low <= ha.open.min(ha.close), "HA low must contain body");
            assert!(ha.high >= src.high, "HA high must not drop below raw high");
            assert!(ha.low <= src.low, "HA low must not rise above raw low");
            assert_eq!(ha.t, src.t);
            assert_eq!(ha.volume, src.volume);
        }
    }

    #[test]
    fn test_heikin_ashi_empty_series() {
        assert!(heikin_ashi(&[]).is_empty());
    }
}