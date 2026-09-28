// crates/bt-data/src/india.rs
// Author: Sourish Dey

//! India-specific market data types and sample data generators.
//!
//! Provides realistic-looking sample data for Indian markets:
//! NSE F&O options chains, IV surfaces, OI heatmaps, G-Sec yields,
//! money market rates, RBI policy, macro indicators, MCX/NCDEX
//! commodities, USD/INR forwards, yield curves, mutual funds,
//! FPI/FII flows, credit ratings, banking indicators, corporate
//! actions, IPOs, market breadth, sector research, news, SEBI/RBI
//! regulatory updates, GST/Budget analytics, portfolios with Indian
//! tax treatment, algo feeds, AI research and dashboard widgets.

// ==================== F&O OPTIONS CHAIN ====================

/// One strike row of an NSE F&O options chain.
pub struct OptionsChainEntry {
    pub strike: f64,
    pub call_oi: u64,
    pub call_chg_oi: i64,
    pub call_iv: f64,
    pub call_delta: f64,
    pub call_gamma: f64,
    pub call_theta: f64,
    pub call_vega: f64,
    pub put_oi: u64,
    pub put_chg_oi: i64,
    pub put_iv: f64,
    pub put_delta: f64,
    pub put_gamma: f64,
    pub put_theta: f64,
    pub put_vega: f64,
}

/// Generate a realistic NIFTY options chain around the given spot.
pub fn sample_options_chain(spot: f64) -> Vec<OptionsChainEntry> {
    let atm = (spot / 100.0).round() * 100.0;
    let mut entries = Vec::new();
    for i in -7..=7 {
        let strike = atm + i as f64 * 100.0;
        let moneyness = (strike - spot) / spot;
        let call_oi = (2_500_000.0 * (1.0 - (i as f64 / 8.0).abs()) + 50_000.0) as u64;
        let put_oi = (2_200_000.0 * (1.0 - (i as f64 / 8.0).abs()) + 45_000.0) as u64;
        let call_iv = 0.135 + 0.09 * moneyness * moneyness * 8.0 + if i == 0 { 0.004 } else { 0.0 };
        let put_iv = call_iv + 0.012;
        let gamma = 0.006 * (1.0 - moneyness.abs() * 6.0).max(0.15);
        entries.push(OptionsChainEntry {
            strike,
            call_oi,
            call_chg_oi: (i as i64 * 12_500) * if i % 2 == 0 { 1 } else { -1 },
            call_iv,
            call_delta: (0.5 + moneyness * 4.0).clamp(0.02, 0.98),
            call_gamma: gamma,
            call_theta: -0.35 * (1.0 - moneyness.abs() * 5.0).max(0.2),
            call_vega: 0.18 * (1.0 - moneyness.abs() * 4.0).max(0.25),
            put_oi,
            put_chg_oi: (i as i64 * 9_800) * if i % 2 == 0 { -1 } else { 1 },
            put_iv,
            put_delta: -((0.5 - moneyness * 4.0).clamp(0.02, 0.98)),
            put_gamma: gamma,
            put_theta: -0.28 * (1.0 - moneyness.abs() * 5.0).max(0.2),
            put_vega: 0.17 * (1.0 - moneyness.abs() * 4.0).max(0.25),
        });
    }
    entries
}

// ==================== IV SURFACE ====================

/// Implied volatility surface: tenors x strikes grid.
pub struct IVSurface {
    pub tenors: Vec<f64>,
    pub strikes: Vec<f64>,
    /// ivs[tenor_idx][strike_idx]
    pub ivs: Vec<Vec<f64>>,
}

pub fn sample_iv_surface() -> IVSurface {
    let tenors = vec![7.0_f64, 14.0, 30.0, 60.0, 90.0, 180.0];
    let strikes = vec![0.80_f64, 0.90, 0.95, 1.00, 1.05, 1.10, 1.20];
    let ivs = tenors
        .iter()
        .map(|&t| {
            strikes
                .iter()
                .map(|&k| {
                    let m = k - 1.0;
                    0.125 + 0.075 * m * m * 10.0 + 0.018 / (t / 30.0).sqrt()
                })
                .collect()
        })
        .collect();
    IVSurface { tenors, strikes, ivs }
}

// ==================== OI HEATMAP ====================

/// Open interest heatmap across underlyings and strikes.
pub struct OIHeatmap {
    pub underlyings: Vec<String>,
    pub strikes: Vec<f64>,
    /// oi[underlying_idx][strike_idx] in lakh contracts
    pub oi: Vec<Vec<f64>>,
}

pub fn sample_oi_heatmap() -> OIHeatmap {
    let underlyings: Vec<String> = vec![
        "NIFTY 50", "BANKNIFTY", "FINNIFTY", "MIDCPNIFTY", "SENSEX",
        "RELIANCE", "HDFCBANK", "INFY", "TCS", "ICICIBANK",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect();
    let strikes = vec![0.85_f64, 0.90, 0.95, 1.00, 1.05, 1.10, 1.15];
    let oi = underlyings
        .iter()
        .enumerate()
        .map(|(u, _)| {
            strikes
                .iter()
                .enumerate()
                .map(|(s, &k)| {
                    let base = 4.0 + 3.0 * ((u + s) as f64 * 0.7).sin().abs();
                    let atm_boost = if (k - 1.0).abs() < 0.01 { 6.0 } else { 0.0 };
                    base + atm_boost + (s as f64 * 0.3)
                })
                .collect()
        })
        .collect();
    OIHeatmap { underlyings, strikes, oi }
    // underlyings converted to Vec<String> above
}

// ==================== G-SEC YIELD CURVE ====================

/// Government of India securities yield curve.
pub struct GSecData {
    pub tenors: Vec<f64>,
    pub yields: Vec<f64>,
}

pub fn sample_gsec_curve() -> GSecData {
    let tenors = vec![0.25_f64, 0.5, 1.0, 2.0, 3.0, 5.0, 7.0, 10.0, 15.0, 20.0, 30.0];
    let yields = vec![5.62_f64, 5.78, 6.05, 6.28, 6.42, 6.55, 6.62, 6.71, 6.85, 6.92, 7.05];
    GSecData { tenors, yields }
}

// ==================== MONEY MARKET ====================

/// Indian money market rates snapshot.
pub struct MoneyMarketRate {
    pub mibor_1d: f64,
    pub mibor_1w: f64,
    pub mibor_1m: f64,
    pub mibor_3m: f64,
    pub treps: f64,
    pub cblo: f64,
    pub t_bill_91: f64,
    pub t_bill_182: f64,
    pub t_bill_364: f64,
    pub cp_3m: f64,
    pub cd_3m: f64,
    pub reverse_repo: f64,
    pub msf: f64,
}

pub fn sample_money_market() -> MoneyMarketRate {
    MoneyMarketRate {
        mibor_1d: 5.42,
        mibor_1w: 5.45,
        mibor_1m: 5.52,
        mibor_3m: 5.68,
        treps: 5.38,
        cblo: 5.35,
        t_bill_91: 5.31,
        t_bill_182: 5.47,
        t_bill_364: 5.63,
        cp_3m: 5.85,
        cd_3m: 5.72,
        reverse_repo: 5.25,
        msf: 5.75,
    }
}

// ==================== RBI MONETARY POLICY ====================

/// RBI monetary policy dashboard snapshot.
pub struct RBIPolicy {
    pub repo_rate: f64,
    pub reverse_repo: f64,
    pub msf: f64,
    pub crr: f64,
    pub slr: f64,
    pub bank_rate: f64,
    pub stance: &'static str,
    pub last_meeting: &'static str,
    pub next_meeting: &'static str,
    pub inflation_target: f64,
    pub inflation_tolerance: f64,
    pub real_rate: f64,
    pub liquidity_modality: &'static str,
}

pub fn sample_rbi_policy() -> RBIPolicy {
    RBIPolicy {
        repo_rate: 5.50,
        reverse_repo: 5.25,
        msf: 5.75,
        crr: 4.50,
        slr: 18.00,
        bank_rate: 5.75,
        stance: "Accommodative",
        last_meeting: "2026-08-06",
        next_meeting: "2026-10-01",
        inflation_target: 4.0,
        inflation_tolerance: 2.0,
        real_rate: 1.05,
        liquidity_modality: "Neutral",
    }
}

// ==================== MACRO INDICATORS ====================

/// One Indian macroeconomic indicator.
pub struct MacroIndicator {
    pub name: &'static str,
    pub value: f64,
    pub yoy: f64,
    pub prev: f64,
    pub unit: &'static str,
    pub frequency: &'static str,
}

pub fn sample_macro_indicators() -> Vec<MacroIndicator> {
    vec![
        MacroIndicator { name: "CPI (Combined)", value: 4.8, yoy: 4.8, prev: 4.3, unit: "% yoy", frequency: "Monthly" },
        MacroIndicator { name: "WPI", value: 2.1, yoy: 2.1, prev: 1.6, unit: "% yoy", frequency: "Monthly" },
        MacroIndicator { name: "IIP", value: 3.9, yoy: 3.9, prev: 3.2, unit: "% yoy", frequency: "Monthly" },
        MacroIndicator { name: "GDP (Q)", value: 6.8, yoy: 6.8, prev: 6.2, unit: "% yoy", frequency: "Quarterly" },
        MacroIndicator { name: "Manufacturing PMI", value: 57.4, yoy: 1.2, prev: 56.8, unit: "idx", frequency: "Monthly" },
        MacroIndicator { name: "Services PMI", value: 59.1, yoy: 0.8, prev: 58.4, unit: "idx", frequency: "Monthly" },
        MacroIndicator { name: "Core Sector", value: 4.5, yoy: 4.5, prev: 3.8, unit: "% yoy", frequency: "Monthly" },
        MacroIndicator { name: "Trade Deficit", value: -21.4, yoy: -12.0, prev: -19.8, unit: "USD bn", frequency: "Monthly" },
        MacroIndicator { name: "Fiscal Deficit", value: 4.4, yoy: -0.5, prev: 4.9, unit: "% GDP", frequency: "FY" },
        MacroIndicator { name: "CAD", value: -0.9, yoy: 0.3, prev: -1.1, unit: "% GDP", frequency: "Quarterly" },
        MacroIndicator { name: "Unemployment", value: 7.2, yoy: -0.4, prev: 7.6, unit: "%", frequency: "Monthly" },
        MacroIndicator { name: "Bank Credit", value: 11.8, yoy: 11.8, prev: 11.2, unit: "% yoy", frequency: "Fortnightly" },
    ]
}

// ==================== MCX / NCDEX COMMODITIES ====================

/// One commodity quote from MCX or NCDEX.
pub struct CommodityData {
    pub name: &'static str,
    pub exchange: &'static str,
    pub price: f64,
    pub unit: &'static str,
    pub change_pct: f64,
    pub lot_size: u32,
}

pub fn sample_commodities() -> Vec<CommodityData> {
    vec![
        CommodityData { name: "Gold", exchange: "MCX", price: 104_850.0, unit: "10g", change_pct: 0.85, lot_size: 100 },
        CommodityData { name: "Silver", exchange: "MCX", price: 121_400.0, unit: "1kg", change_pct: 1.42, lot_size: 30 },
        CommodityData { name: "Crude Oil (WTI)", exchange: "MCX", price: 5_845.0, unit: "bbl", change_pct: -0.92, lot_size: 100 },
        CommodityData { name: "Natural Gas", exchange: "MCX", price: 218.4, unit: "mmBtu", change_pct: 2.15, lot_size: 1_250 },
        CommodityData { name: "Copper", exchange: "MCX", price: 862.3, unit: "kg", change_pct: -0.35, lot_size: 2_500 },
        CommodityData { name: "Zinc", exchange: "MCX", price: 268.9, unit: "kg", change_pct: 0.48, lot_size: 5_000 },
        CommodityData { name: "Aluminium", exchange: "MCX", price: 242.6, unit: "kg", change_pct: -0.18, lot_size: 5_000 },
        CommodityData { name: "Lead", exchange: "MCX", price: 188.2, unit: "kg", change_pct: 0.22, lot_size: 5_000 },
        CommodityData { name: "Nickel", exchange: "MCX", price: 1_425.0, unit: "kg", change_pct: -1.10, lot_size: 100 },
        CommodityData { name: "Mentha Oil", exchange: "MCX", price: 985.0, unit: "kg", change_pct: 0.65, lot_size: 360 },
        CommodityData { name: "Cardamom", exchange: "NCDEX", price: 2_450.0, unit: "kg", change_pct: -0.85, lot_size: 100 },
        CommodityData { name: "Cotton", exchange: "NCDEX", price: 56_800.0, unit: "candy", change_pct: 0.35, lot_size: 200 },
        CommodityData { name: "Cumin (Jeera)", exchange: "NCDEX", price: 28_950.0, unit: "kg", change_pct: 1.20, lot_size: 100 },
        CommodityData { name: "Turmeric", exchange: "NCDEX", price: 14_200.0, unit: "kg", change_pct: -0.45, lot_size: 100 },
        CommodityData { name: "Guar Seed", exchange: "NCDEX", price: 5_850.0, unit: "kg", change_pct: 0.90, lot_size: 100 },
    ]
}

// ==================== USD/INR FORWARD CURVE ====================

/// USD/INR spot and forward outright curve.
pub struct USDRate {
    /// tenor in days
    pub tenors: Vec<f64>,
    pub outright: Vec<f64>,
    pub forward_points: Vec<f64>,
}

pub fn sample_usdinr_forward() -> USDRate {
    let tenors = vec![0.0_f64, 7.0, 15.0, 30.0, 60.0, 90.0, 180.0, 270.0, 365.0];
    let forward_points = vec![0.0_f64, 12.0, 21.0, 42.0, 85.0, 128.0, 255.0, 382.0, 510.0];
    let spot = 86.42;
    let outright = forward_points.iter().map(|&p| spot + p / 10_000.0).collect();
    USDRate { tenors, outright, forward_points }
}

// ==================== INDIA YIELD CURVE ====================

/// Multi-segment Indian yield curve: sovereign, SDL, corporate.
pub struct YieldCurvePoint {
    pub tenors: Vec<f64>,
    pub sovereign: Vec<f64>,
    pub sdl: Vec<f64>,
    pub corporate_aaa: Vec<f64>,
    pub corporate_aa: Vec<f64>,
}

pub fn sample_yield_india() -> YieldCurvePoint {
    let tenors = vec![0.25_f64, 0.5, 1.0, 2.0, 3.0, 5.0, 7.0, 10.0, 15.0, 20.0, 30.0];
    let sovereign = vec![5.62_f64, 5.78, 6.05, 6.28, 6.42, 6.55, 6.62, 6.71, 6.85, 6.92, 7.05];
    let sdl = sovereign.iter().map(|&y| y + 0.08).collect();
    let corporate_aaa = sovereign.iter().map(|&y| y + 0.35).collect();
    let corporate_aa = sovereign.iter().map(|&y| y + 0.85).collect();
    YieldCurvePoint { tenors, sovereign, sdl, corporate_aaa, corporate_aa }
}

// ==================== MUTUAL FUND ANALYTICS ====================

/// One mutual fund scheme with analytics.
pub struct MutualFundData {
    pub name: &'static str,
    pub category: &'static str,
    pub nav: f64,
    pub aum_cr: f64,
    pub ret_1y: f64,
    pub ret_3y: f64,
    pub ret_5y: f64,
    pub expense: f64,
    pub sharpe: f64,
    pub beta: f64,
    pub alpha: f64,
    pub std_dev: f64,
}

pub fn sample_mf_schemes() -> Vec<MutualFundData> {
    vec![
        MutualFundData { name: "Nippon India Small Cap Fund", category: "Small Cap", nav: 168.42, aum_cr: 58_240.0, ret_1y: 18.4, ret_3y: 21.2, ret_5y: 24.8, expense: 0.72, sharpe: 1.12, beta: 0.94, alpha: 3.8, std_dev: 14.2 },
        MutualFundData { name: "HDFC Mid-Cap Opportunities", category: "Mid Cap", nav: 172.15, aum_cr: 71_580.0, ret_1y: 14.2, ret_3y: 18.6, ret_5y: 21.4, expense: 0.78, sharpe: 0.94, beta: 0.91, alpha: 2.4, std_dev: 13.1 },
        MutualFundData { name: "SBI Bluechip Fund", category: "Large Cap", nav: 92.68, aum_cr: 52_310.0, ret_1y: 11.8, ret_3y: 14.2, ret_5y: 16.9, expense: 0.82, sharpe: 0.81, beta: 0.88, alpha: 1.2, std_dev: 11.8 },
        MutualFundData { name: "ICICI Pru Technology Fund", category: "Sectoral - IT", nav: 142.30, aum_cr: 12_840.0, ret_1y: 8.4, ret_3y: 12.8, ret_5y: 18.2, expense: 0.95, sharpe: 0.52, beta: 1.05, alpha: -0.8, std_dev: 16.4 },
        MutualFundData { name: "Kotak Equity Opportunities", category: "Large & Mid Cap", nav: 312.55, aum_cr: 24_120.0, ret_1y: 13.6, ret_3y: 16.4, ret_5y: 19.1, expense: 0.68, sharpe: 0.88, beta: 0.86, alpha: 1.9, std_dev: 12.2 },
        MutualFundData { name: "Axis Small Cap Fund", category: "Small Cap", nav: 78.92, aum_cr: 22_460.0, ret_1y: 16.8, ret_3y: 19.8, ret_5y: 22.6, expense: 0.74, sharpe: 1.02, beta: 0.92, alpha: 3.1, std_dev: 13.8 },
        MutualFundData { name: "Mirae Asset Emerging Bluechip", category: "Large & Mid Cap", nav: 118.40, aum_cr: 38_920.0, ret_1y: 12.4, ret_3y: 15.8, ret_5y: 18.4, expense: 0.62, sharpe: 0.85, beta: 0.84, alpha: 1.6, std_dev: 12.0 },
        MutualFundData { name: "Tata Digital India Fund", category: "Sectoral - IT", nav: 52.18, aum_cr: 8_140.0, ret_1y: 6.2, ret_3y: 10.4, ret_5y: 15.8, expense: 1.02, sharpe: 0.41, beta: 1.08, alpha: -1.4, std_dev: 17.2 },
        MutualFundData { name: "Parag Parikh Flexi Cap Fund", category: "Flexi Cap", nav: 72.85, aum_cr: 78_640.0, ret_1y: 15.2, ret_3y: 17.2, ret_5y: 19.8, expense: 0.63, sharpe: 0.96, beta: 0.82, alpha: 2.8, std_dev: 11.4 },
        MutualFundData { name: "HDFC Hybrid Equity Fund", category: "Hybrid", nav: 88.30, aum_cr: 24_800.0, ret_1y: 10.8, ret_3y: 12.6, ret_5y: 14.2, expense: 0.88, sharpe: 0.72, beta: 0.68, alpha: 1.4, std_dev: 9.8 },
    ]
}

// ==================== FPI / FII FLOWS ====================

/// Daily FPI/FII/DII flow entry (INR crore).
pub struct FPIFlow {
    pub date: &'static str,
    pub fpi_equity: f64,
    pub fpi_debt: f64,
    pub fii_equity: f64,
    pub dii: f64,
}

pub fn sample_fpi_fii_flows() -> Vec<FPIFlow> {
    vec![
        FPIFlow { date: "2026-09-01", fpi_equity: -1_842.0, fpi_debt: 620.0, fii_equity: -1_842.0, dii: 2_140.0 },
        FPIFlow { date: "2026-09-02", fpi_equity: -2_415.0, fpi_debt: 480.0, fii_equity: -2_415.0, dii: 2_680.0 },
        FPIFlow { date: "2026-09-03", fpi_equity: -1_208.0, fpi_debt: 350.0, fii_equity: -1_208.0, dii: 1_920.0 },
        FPIFlow { date: "2026-09-04", fpi_equity: 864.0, fpi_debt: 210.0, fii_equity: 864.0, dii: 1_450.0 },
        FPIFlow { date: "2026-09-05", fpi_equity: -3_120.0, fpi_debt: 540.0, fii_equity: -3_120.0, dii: 2_980.0 },
        FPIFlow { date: "2026-09-08", fpi_equity: -1_640.0, fpi_debt: 390.0, fii_equity: -1_640.0, dii: 2_210.0 },
        FPIFlow { date: "2026-09-09", fpi_equity: 1_240.0, fpi_debt: 180.0, fii_equity: 1_240.0, dii: 1_680.0 },
        FPIFlow { date: "2026-09-10", fpi_equity: -980.0, fpi_debt: 420.0, fii_equity: -980.0, dii: 1_890.0 },
        FPIFlow { date: "2026-09-11", fpi_equity: -2_760.0, fpi_debt: 610.0, fii_equity: -2_760.0, dii: 2_740.0 },
        FPIFlow { date: "2026-09-12", fpi_equity: 540.0, fpi_debt: 290.0, fii_equity: 540.0, dii: 1_520.0 },
        FPIFlow { date: "2026-09-15", fpi_equity: -1_420.0, fpi_debt: 470.0, fii_equity: -1_420.0, dii: 2_080.0 },
        FPIFlow { date: "2026-09-16", fpi_equity: -890.0, fpi_debt: 330.0, fii_equity: -890.0, dii: 1_760.0 },
        FPIFlow { date: "2026-09-17", fpi_equity: 1_680.0, fpi_debt: 240.0, fii_equity: 1_680.0, dii: 1_340.0 },
        FPIFlow { date: "2026-09-18", fpi_equity: -2_140.0, fpi_debt: 520.0, fii_equity: -2_140.0, dii: 2_420.0 },
        FPIFlow { date: "2026-09-19", fpi_equity: -1_120.0, fpi_debt: 380.0, fii_equity: -1_120.0, dii: 1_980.0 },
        FPIFlow { date: "2026-09-22", fpi_equity: 920.0, fpi_debt: 260.0, fii_equity: 920.0, dii: 1_610.0 },
        FPIFlow { date: "2026-09-23", fpi_equity: -1_580.0, fpi_debt: 440.0, fii_equity: -1_580.0, dii: 2_290.0 },
        FPIFlow { date: "2026-09-24", fpi_equity: -760.0, fpi_debt: 310.0, fii_equity: -760.0, dii: 1_830.0 },
        FPIFlow { date: "2026-09-25", fpi_equity: 1_140.0, fpi_debt: 190.0, fii_equity: 1_140.0, dii: 1_470.0 },
    ]
}

// ==================== CREDIT RATINGS ====================

/// One credit rating entry.
pub struct CreditRating {
    pub issuer: &'static str,
    pub instrument: &'static str,
    pub rating: &'static str,
    pub outlook: &'static str,
    pub agency: &'static str,
    pub amount_cr: f64,
    pub action: &'static str,
}

pub fn sample_credit_ratings() -> Vec<CreditRating> {
    vec![
        CreditRating { issuer: "HDFC Bank", instrument: "NCD", rating: "CRISIL AAA", outlook: "Stable", agency: "CRISIL", amount_cr: 5_000.0, action: "Reaffirmed" },
        CreditRating { issuer: "L&T Finance", instrument: "NCD", rating: "ICRA AA+", outlook: "Positive", agency: "ICRA", amount_cr: 2_500.0, action: "Upgrade" },
        CreditRating { issuer: "Tata Motors", instrument: "NCD", rating: "CARE AA", outlook: "Stable", agency: "CARE", amount_cr: 1_800.0, action: "Reaffirmed" },
        CreditRating { issuer: "Adani Ports", instrument: "CP", rating: "CRISIL A1+", outlook: "Stable", agency: "CRISIL", amount_cr: 4_000.0, action: "Reaffirmed" },
        CreditRating { issuer: "Bajaj Finance", instrument: "NCD", rating: "IND AAA", outlook: "Stable", agency: "India Ratings", amount_cr: 3_200.0, action: "Reaffirmed" },
        CreditRating { issuer: "Shriram Finance", instrument: "NCD", rating: "ICRA AA", outlook: "Negative", agency: "ICRA", amount_cr: 1_500.0, action: "Downgrade" },
        CreditRating { issuer: "NTPC", instrument: "Bond", rating: "CRISIL AAA", outlook: "Stable", agency: "CRISIL", amount_cr: 6_000.0, action: "Reaffirmed" },
        CreditRating { issuer: "Manappuram Finance", instrument: "NCD", rating: "CARE A+", outlook: "Stable", agency: "CARE", amount_cr: 800.0, action: "Reaffirmed" },
        CreditRating { issuer: "Tata Capital", instrument: "CP", rating: "CRISIL A1+", outlook: "Stable", agency: "CRISIL", amount_cr: 2_200.0, action: "Reaffirmed" },
        CreditRating { issuer: "JSW Steel", instrument: "NCD", rating: "IND AA-", outlook: "Positive", agency: "India Ratings", amount_cr: 1_200.0, action: "Upgrade" },
        CreditRating { issuer: "Cholamandalam", instrument: "NCD", rating: "ICRA AA", outlook: "Stable", agency: "ICRA", amount_cr: 950.0, action: "Reaffirmed" },
        CreditRating { issuer: "SBI Cards", instrument: "NCD", rating: "CRISIL AAA", outlook: "Stable", agency: "CRISIL", amount_cr: 1_600.0, action: "Reaffirmed" },
    ]
}

// ==================== BANKING SYSTEM ====================

/// One banking system indicator.
pub struct BankingIndicator {
    pub name: &'static str,
    pub value: f64,
    pub prev: f64,
    pub unit: &'static str,
}

pub fn sample_banking_indicators() -> Vec<BankingIndicator> {
    vec![
        BankingIndicator { name: "Gross NPA Ratio", value: 2.1, prev: 2.3, unit: "%" },
        BankingIndicator { name: "Net NPA Ratio", value: 0.5, prev: 0.6, unit: "%" },
        BankingIndicator { name: "CRAR", value: 16.4, prev: 16.1, unit: "%" },
        BankingIndicator { name: "Credit Growth", value: 11.8, prev: 11.2, unit: "% yoy" },
        BankingIndicator { name: "Deposit Growth", value: 10.6, prev: 10.9, unit: "% yoy" },
        BankingIndicator { name: "CD Ratio", value: 76.8, prev: 77.4, unit: "%" },
        BankingIndicator { name: "SLR Investment", value: 24.2, prev: 24.0, unit: "% NDTL" },
        BankingIndicator { name: "Provision Coverage", value: 78.5, prev: 77.2, unit: "%" },
        BankingIndicator { name: "Return on Assets", value: 1.1, prev: 1.0, unit: "%" },
        BankingIndicator { name: "Net Interest Margin", value: 3.4, prev: 3.5, unit: "%" },
        BankingIndicator { name: "Cost of Funds", value: 5.1, prev: 5.2, unit: "%" },
        BankingIndicator { name: "Yield on Advances", value: 8.9, prev: 8.8, unit: "%" },
    ]
}

// ==================== CORPORATE ACTIONS ====================

/// One corporate action calendar entry.
pub struct CorporateAction {
    pub symbol: &'static str,
    pub company: &'static str,
    pub action: &'static str,
    pub ex_date: &'static str,
    pub record_date: &'static str,
    pub detail: &'static str,
}

pub fn sample_corp_actions() -> Vec<CorporateAction> {
    vec![
        CorporateAction { symbol: "RELIANCE", company: "Reliance Industries", action: "Dividend", ex_date: "2026-09-29", record_date: "2026-09-30", detail: "Rs 13.50 per share" },
        CorporateAction { symbol: "TCS", company: "Tata Consultancy Svcs", action: "Dividend", ex_date: "2026-10-02", record_date: "2026-10-03", detail: "Rs 120 per share (final)" },
        CorporateAction { symbol: "HDFCBANK", company: "HDFC Bank", action: "Split", ex_date: "2026-10-06", record_date: "2026-10-07", detail: "1:5 split" },
        CorporateAction { symbol: "INFY", company: "Infosys", action: "Dividend", ex_date: "2026-10-09", record_date: "2026-10-10", detail: "Rs 23 per share" },
        CorporateAction { symbol: "TATAMOTORS", company: "Tata Motors", action: "Bonus", ex_date: "2026-10-13", record_date: "2026-10-14", detail: "1:2 bonus issue" },
        CorporateAction { symbol: "SBIN", company: "State Bank of India", action: "Dividend", ex_date: "2026-10-16", record_date: "2026-10-17", detail: "Rs 14.20 per share" },
        CorporateAction { symbol: "ITC", company: "ITC", action: "AGM", ex_date: "2026-10-20", record_date: "—", detail: "55th Annual General Meeting" },
        CorporateAction { symbol: "WIPRO", company: "Wipro", action: "Buyback", ex_date: "2026-10-23", record_date: "2026-10-24", detail: "Rs 11,000 cr buyback @ Rs 460" },
        CorporateAction { symbol: "ICICIBANK", company: "ICICI Bank", action: "Dividend", ex_date: "2026-10-27", record_date: "2026-10-28", detail: "Rs 11 per share" },
        CorporateAction { symbol: "TATASTEEL", company: "Tata Steel", action: "Rights", ex_date: "2026-10-30", record_date: "2026-10-31", detail: "1:5 rights @ Rs 555" },
        CorporateAction { symbol: "BAJFINANCE", company: "Bajaj Finance", action: "Dividend", ex_date: "2026-11-03", record_date: "2026-11-04", detail: "Rs 50 per share" },
        CorporateAction { symbol: "LT", company: "Larsen & Toubro", action: "Split", ex_date: "2026-11-06", record_date: "2026-11-07", detail: "1:2 split" },
    ]
}

// ==================== IPO PIPELINE ====================

/// One IPO pipeline entry.
pub struct IPOData {
    pub company: &'static str,
    pub sector: &'static str,
    pub issue_size_cr: f64,
    pub price_band: &'static str,
    pub open_date: &'static str,
    pub close_date: &'static str,
    pub status: &'static str,
    pub gmp: f64,
}

pub fn sample_ipo_pipeline() -> Vec<IPOData> {
    vec![
        IPOData { company: "NovaPay Technologies", sector: "Fintech", issue_size_cr: 2_400.0, price_band: "Rs 420-445", open_date: "2026-09-29", close_date: "2026-10-01", status: "Live", gmp: 18.4 },
        IPOData { company: "GreenVolt Energy", sector: "Renewables", issue_size_cr: 5_800.0, price_band: "Rs 310-325", open_date: "2026-09-29", close_date: "2026-10-02", status: "Live", gmp: 9.2 },
        IPOData { company: "MediChain Hospitals", sector: "Healthcare", issue_size_cr: 1_650.0, price_band: "Rs 540-565", open_date: "2026-10-06", close_date: "2026-10-08", status: "Upcoming", gmp: 12.6 },
        IPOData { company: "AutoNxt Components", sector: "Auto Ancillary", issue_size_cr: 980.0, price_band: "Rs 210-222", open_date: "2026-10-13", close_date: "2026-10-15", status: "Upcoming", gmp: -3.4 },
        IPOData { company: "DataKart Cloud", sector: "IT Services", issue_size_cr: 3_200.0, price_band: "Rs 1,150-1,210", open_date: "2026-10-20", close_date: "2026-10-22", status: "Upcoming", gmp: 22.8 },
        IPOData { company: "SpiceRoute Foods", sector: "FMCG", issue_size_cr: 1_250.0, price_band: "Rs 380-400", open_date: "2026-10-27", close_date: "2026-10-29", status: "Upcoming", gmp: 6.1 },
        IPOData { company: "UrbanNest Realty", sector: "Realty", issue_size_cr: 2_100.0, price_band: "Rs 620-650", open_date: "2026-11-03", close_date: "2026-11-05", status: "Upcoming", gmp: 4.5 },
        IPOData { company: "PayBridge Solutions", sector: "Payments", issue_size_cr: 4_500.0, price_band: "Rs 890-930", open_date: "2026-09-24", close_date: "2026-09-26", status: "Listed", gmp: 31.2 },
        IPOData { company: "Solaris Textiles", sector: "Textiles", issue_size_cr: 720.0, price_band: "Rs 145-152", open_date: "2026-09-17", close_date: "2026-09-19", status: "Listed", gmp: -8.6 },
        IPOData { company: "LogiMove Freight", sector: "Logistics", issue_size_cr: 1_890.0, price_band: "Rs 265-278", open_date: "2026-09-10", close_date: "2026-09-12", status: "Listed", gmp: 14.9 },
    ]
}

// ==================== MARKET BREADTH ====================

/// India market breadth snapshot.
pub struct BreadthData {
    pub advances: u32,
    pub declines: u32,
    pub unchanged: u32,
    pub new_52w_highs: u32,
    pub new_52w_lows: u32,
    pub above_50dma: u32,
    pub above_200dma: u32,
    pub total: u32,
}

pub fn sample_breadth() -> BreadthData {
    BreadthData {
        advances: 1_842,
        declines: 1_216,
        unchanged: 94,
        new_52w_highs: 86,
        new_52w_lows: 24,
        above_50dma: 1_468,
        above_200dma: 1_102,
        total: 3_152,
    }
}

// ==================== SECTOR RESEARCH ====================

/// One sector research summary.
pub struct SectorResearch {
    pub sector: &'static str,
    pub pe: f64,
    pub pb: f64,
    pub div_yield: f64,
    pub eps_growth: f64,
    pub roe: f64,
    pub debt_equity: f64,
    pub mom_1m: f64,
    pub mom_3m: f64,
    pub mom_1y: f64,
    pub outlook: &'static str,
}

pub fn sample_sector_research() -> Vec<SectorResearch> {
    vec![
        SectorResearch { sector: "IT Services", pe: 24.8, pb: 6.2, div_yield: 1.8, eps_growth: 12.4, roe: 28.5, debt_equity: 0.12, mom_1m: 2.4, mom_3m: 6.8, mom_1y: 14.2, outlook: "Overweight" },
        SectorResearch { sector: "Banking", pe: 11.2, pb: 1.8, div_yield: 1.4, eps_growth: 14.8, roe: 15.2, debt_equity: 6.8, mom_1m: 1.8, mom_3m: 4.2, mom_1y: 18.6, outlook: "Overweight" },
        SectorResearch { sector: "Pharma", pe: 32.4, pb: 5.8, div_yield: 0.9, eps_growth: 16.2, roe: 18.4, debt_equity: 0.28, mom_1m: 3.2, mom_3m: 8.4, mom_1y: 22.4, outlook: "Overweight" },
        SectorResearch { sector: "Auto", pe: 18.6, pb: 3.4, div_yield: 1.6, eps_growth: 18.4, roe: 16.8, debt_equity: 0.42, mom_1m: -1.2, mom_3m: 2.4, mom_1y: 12.8, outlook: "Neutral" },
        SectorResearch { sector: "FMCG", pe: 52.8, pb: 12.4, div_yield: 1.2, eps_growth: 9.8, roe: 24.2, debt_equity: 0.08, mom_1m: 0.8, mom_3m: 3.2, mom_1y: 8.4, outlook: "Neutral" },
        SectorResearch { sector: "Metals", pe: 14.2, pb: 2.6, div_yield: 2.4, eps_growth: 22.4, roe: 22.8, debt_equity: 0.68, mom_1m: 4.2, mom_3m: 12.4, mom_1y: 28.6, outlook: "Overweight" },
        SectorResearch { sector: "Realty", pe: 38.4, pb: 5.2, div_yield: 0.6, eps_growth: 24.8, roe: 12.4, debt_equity: 1.24, mom_1m: 5.4, mom_3m: 14.2, mom_1y: 32.4, outlook: "Overweight" },
        SectorResearch { sector: "Oil & Gas", pe: 9.8, pb: 1.4, div_yield: 3.8, eps_growth: 8.4, roe: 14.2, debt_equity: 0.52, mom_1m: -2.4, mom_3m: -4.8, mom_1y: -8.2, outlook: "Underweight" },
        SectorResearch { sector: "PSU Banks", pe: 8.4, pb: 1.2, div_yield: 2.8, eps_growth: 12.8, roe: 12.8, debt_equity: 7.4, mom_1m: 1.2, mom_3m: 3.8, mom_1y: 16.4, outlook: "Neutral" },
        SectorResearch { sector: "Media", pe: 28.4, pb: 4.2, div_yield: 0.8, eps_growth: 20.4, roe: 14.6, debt_equity: 0.34, mom_1m: 6.8, mom_3m: 16.2, mom_1y: 38.4, outlook: "Overweight" },
        SectorResearch { sector: "Infrastructure", pe: 22.4, pb: 3.8, div_yield: 1.0, eps_growth: 16.8, roe: 11.2, debt_equity: 1.86, mom_1m: 2.8, mom_3m: 7.4, mom_1y: 19.8, outlook: "Overweight" },
        SectorResearch { sector: "Telecom", pe: 16.8, pb: 2.8, div_yield: 2.2, eps_growth: 10.4, roe: 9.8, debt_equity: 2.4, mom_1m: 1.4, mom_3m: 4.8, mom_1y: 10.2, outlook: "Neutral" },
    ]
}

// ==================== INDIA NEWS ====================

/// One India market news item.
pub struct NewsItem {
    pub time: &'static str,
    pub source: &'static str,
    pub headline: &'static str,
    pub category: &'static str,
    pub sentiment: f64,
}

pub fn sample_india_news() -> Vec<NewsItem> {
    vec![
        NewsItem { time: "09:42", source: "ET Markets", headline: "RBI keeps repo unchanged at 5.50%, maintains accommodative stance", category: "Policy", sentiment: 0.6 },
        NewsItem { time: "09:35", source: "Reuters", headline: "Nifty 50 opens 0.4% higher, Bank Nifty hits record high", category: "Markets", sentiment: 0.7 },
        NewsItem { time: "09:28", source: "Moneycontrol", headline: "FIIs net sellers for 8th straight session; DIIs absorb supply", category: "Flows", sentiment: -0.3 },
        NewsItem { time: "09:15", source: "BS", headline: "GST collections for August at Rs 1.92 lakh crore, up 9.4% yoy", category: "Macro", sentiment: 0.5 },
        NewsItem { time: "09:02", source: "ET Tech", headline: "IT majors guide for H2 recovery as deal pipelines swell", category: "Sectors", sentiment: 0.4 },
        NewsItem { time: "08:50", source: "Livemint", headline: "Sebi proposes tighter disclosure norms for FPIs", category: "Regulatory", sentiment: -0.2 },
        NewsItem { time: "08:36", source: "CNBC-TV18", headline: "Crude slips 1.2% as OPEC+ weighs output hike; ONGC, Oil India in focus", category: "Commodities", sentiment: -0.4 },
        NewsItem { time: "08:20", source: "ET Markets", headline: "Sensex crosses 80,000 for first time; breadth strong", category: "Markets", sentiment: 0.8 },
        NewsItem { time: "08:05", source: "BS", headline: "RBI governor: inflation trajectory within target band, growth resilient", category: "Policy", sentiment: 0.5 },
        NewsItem { time: "07:48", source: "Moneycontrol", headline: "IPO market heats up: 4 issues to raise Rs 12,000 cr this week", category: "IPO", sentiment: 0.6 },
        NewsItem { time: "07:30", source: "Reuters", headline: "Rupee weakens to 86.42/USD as oil importers step up dollar bids", category: "FX", sentiment: -0.3 },
        NewsItem { time: "07:12", source: "ET Auto", headline: "August PV dispatches up 11% yoy; Maruti, M&M gain share", category: "Sectors", sentiment: 0.4 },
    ]
}

// ==================== REGULATORY UPDATES ====================

/// One SEBI/RBI regulatory update.
pub struct RegulatoryUpdate {
    pub date: &'static str,
    pub regulator: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub impact: &'static str,
}

pub fn sample_regulatory_updates() -> Vec<RegulatoryUpdate> {
    vec![
        RegulatoryUpdate { date: "2026-09-25", regulator: "SEBI", title: "Tighter P-Note disclosure norms", summary: "Overseas derivative instrument issuers must report client-level detail fortnightly.", impact: "Medium" },
        RegulatoryUpdate { date: "2026-09-22", regulator: "RBI", title: "LTV norms for gold loans tightened", summary: "Loan-to-value ratio capped at 75% for gold jewellery loans above Rs 2 lakh.", impact: "High" },
        RegulatoryUpdate { date: "2026-09-18", regulator: "SEBI", title: "New algo trading framework", summary: "All algorithmic orders must carry unique identifier; exchanges to provide kill switch.", impact: "High" },
        RegulatoryUpdate { date: "2026-09-15", regulator: "RBI", title: "UPI transaction limit raised", summary: "Per-transaction limit for UPI payments raised to Rs 5 lakh for specific categories.", impact: "Medium" },
        RegulatoryUpdate { date: "2026-09-10", regulator: "SEBI", title: "Mutual fund stress test disclosure", summary: "AMCs to disclose portfolio stress test results for debt schemes monthly.", impact: "Low" },
        RegulatoryUpdate { date: "2026-09-05", regulator: "RBI", title: "Digital lending guidelines update", summary: "Cooling-off period for digital loans extended to 7 days with free exit.", impact: "Medium" },
        RegulatoryUpdate { date: "2026-08-28", regulator: "SEBI", title: "Short-selling norms revised", summary: "All investors allowed to short sell; stock borrowing framework simplified.", impact: "High" },
        RegulatoryUpdate { date: "2026-08-20", regulator: "RBI", title: "Co-lending priority sector push", summary: "Risk-sharing ratio for bank-NBFC co-lending revised to 20:80.", impact: "Medium" },
        RegulatoryUpdate { date: "2026-08-12", regulator: "SEBI", title: "REIT/InvIT disclosure overhaul", summary: "Quarterly valuation reports to be published within 30 days of quarter end.", impact: "Low" },
        RegulatoryUpdate { date: "2026-08-05", regulator: "RBI", title: "Inflation target reaffirmed", summary: "CPI target of 4% with +/-2% tolerance extended for next 5 years.", impact: "High" },
    ]
}

// ==================== GST & BUDGET ====================

/// Monthly GST collection entry.
pub struct GSTCollection {
    pub month: &'static str,
    pub gst_cr: f64,
    pub yoy: f64,
}

pub fn sample_gst_collections() -> Vec<GSTCollection> {
    vec![
        GSTCollection { month: "Oct 25", gst_cr: 17_850.0, yoy: 8.4 },
        GSTCollection { month: "Nov 25", gst_cr: 18_240.0, yoy: 9.1 },
        GSTCollection { month: "Dec 25", gst_cr: 19_480.0, yoy: 10.2 },
        GSTCollection { month: "Jan 26", gst_cr: 19_120.0, yoy: 7.8 },
        GSTCollection { month: "Feb 26", gst_cr: 18_640.0, yoy: 8.9 },
        GSTCollection { month: "Mar 26", gst_cr: 21_480.0, yoy: 11.4 },
        GSTCollection { month: "Apr 26", gst_cr: 18_920.0, yoy: 8.2 },
        GSTCollection { month: "May 26", gst_cr: 19_360.0, yoy: 9.6 },
        GSTCollection { month: "Jun 26", gst_cr: 19_840.0, yoy: 10.1 },
        GSTCollection { month: "Jul 26", gst_cr: 20_120.0, yoy: 9.8 },
        GSTCollection { month: "Aug 26", gst_cr: 19_240.0, yoy: 9.4 },
    ]
}

/// Union budget metric.
pub struct BudgetMetric {
    pub name: &'static str,
    pub fy25: f64,
    pub fy26: f64,
    pub unit: &'static str,
}

pub fn sample_budget_metrics() -> Vec<BudgetMetric> {
    vec![
        BudgetMetric { name: "Fiscal Deficit", fy25: 4.9, fy26: 4.4, unit: "% GDP" },
        BudgetMetric { name: "Total Receipts", fy25: 31.4, fy26: 34.2, unit: "lakh cr" },
        BudgetMetric { name: "Total Expenditure", fy25: 47.2, fy26: 51.8, unit: "lakh cr" },
        BudgetMetric { name: "Capex", fy25: 11.2, fy26: 12.8, unit: "lakh cr" },
        BudgetMetric { name: "Tax Revenue (Net)", fy25: 25.8, fy26: 28.6, unit: "lakh cr" },
        BudgetMetric { name: "Disinvestment", fy25: 0.5, fy26: 1.2, unit: "lakh cr" },
        BudgetMetric { name: "Interest Outgo", fy25: 11.4, fy26: 12.1, unit: "lakh cr" },
        BudgetMetric { name: "Subsidies", fy25: 4.2, fy26: 3.8, unit: "lakh cr" },
    ]
}

// ==================== INDIA PORTFOLIO (TAX) ====================

/// One portfolio holding with Indian tax analytics.
pub struct IndiaPortfolioHolding {
    pub symbol: &'static str,
    pub name: &'static str,
    pub qty: f64,
    pub avg_price: f64,
    pub ltp: f64,
    pub stcg: f64,
    pub ltcg: f64,
    pub div_income: f64,
    pub tax_liability: f64,
}

pub fn sample_india_portfolio() -> Vec<IndiaPortfolioHolding> {
    vec![
        IndiaPortfolioHolding { symbol: "RELIANCE", name: "Reliance Industries", qty: 120.0, avg_price: 2_380.0, ltp: 2_456.75, stcg: 9_210.0, ltcg: 0.0, div_income: 1_620.0, tax_liability: 2_763.0 },
        IndiaPortfolioHolding { symbol: "HDFCBANK", name: "HDFC Bank", qty: 250.0, avg_price: 1_620.0, ltp: 1_678.90, stcg: 0.0, ltcg: 14_725.0, div_income: 3_125.0, tax_liability: 2_945.0 },
        IndiaPortfolioHolding { symbol: "INFY", name: "Infosys", qty: 180.0, avg_price: 1_540.0, ltp: 1_567.30, stcg: 4_914.0, ltcg: 0.0, div_income: 4_140.0, tax_liability: 1_474.0 },
        IndiaPortfolioHolding { symbol: "TCS", name: "Tata Consultancy", qty: 40.0, avg_price: 3_950.0, ltp: 3_890.50, stcg: -2_380.0, ltcg: 0.0, div_income: 4_800.0, tax_liability: 0.0 },
        IndiaPortfolioHolding { symbol: "ICICIBANK", name: "ICICI Bank", qty: 400.0, avg_price: 910.0, ltp: 945.60, stcg: 0.0, ltcg: 14_240.0, div_income: 4_400.0, tax_liability: 2_848.0 },
        IndiaPortfolioHolding { symbol: "TATAMOTORS", name: "Tata Motors", qty: 600.0, avg_price: 760.0, ltp: 789.40, stcg: 17_640.0, ltcg: 0.0, div_income: 1_200.0, tax_liability: 5_292.0 },
        IndiaPortfolioHolding { symbol: "SBIN", name: "State Bank of India", qty: 350.0, avg_price: 600.0, ltp: 623.45, stcg: 0.0, ltcg: 8_207.5, div_income: 4_970.0, tax_liability: 1_641.5 },
        IndiaPortfolioHolding { symbol: "ITC", name: "ITC", qty: 500.0, avg_price: 440.0, ltp: 456.70, stcg: 8_350.0, ltcg: 0.0, div_income: 3_250.0, tax_liability: 2_505.0 },
    ]
}

// ==================== ALGO FEED STATUS ====================

/// One algo feed / market data feed status.
pub struct AlgoFeedStatus {
    pub name: &'static str,
    pub status: &'static str,
    pub latency_ms: u64,
    pub msgs_per_sec: u64,
    pub uptime_pct: f64,
    pub last_error: &'static str,
}

pub fn sample_algo_feeds() -> Vec<AlgoFeedStatus> {
    vec![
        AlgoFeedStatus { name: "NSE Multicast Feed", status: "Connected", latency_ms: 12, msgs_per_sec: 48_200, uptime_pct: 99.98, last_error: "None" },
        AlgoFeedStatus { name: "BSE Multicast Feed", status: "Connected", latency_ms: 15, msgs_per_sec: 31_400, uptime_pct: 99.95, last_error: "None" },
        AlgoFeedStatus { name: "NSE F&O Feed", status: "Connected", latency_ms: 11, msgs_per_sec: 62_800, uptime_pct: 99.99, last_error: "None" },
        AlgoFeedStatus { name: "MCX Market Data", status: "Degraded", latency_ms: 180, msgs_per_sec: 8_400, uptime_pct: 98.42, last_error: "Packet loss 0.3%" },
        AlgoFeedStatus { name: "Currency Derivatives", status: "Connected", latency_ms: 14, msgs_per_sec: 12_600, uptime_pct: 99.91, last_error: "None" },
        AlgoFeedStatus { name: "Co-location Gateway", status: "Connected", latency_ms: 3, msgs_per_sec: 96_000, uptime_pct: 99.99, last_error: "None" },
        AlgoFeedStatus { name: "Ticker Plant (ODR)", status: "Connected", latency_ms: 8, msgs_per_sec: 142_000, uptime_pct: 99.97, last_error: "None" },
        AlgoFeedStatus { name: "Risk Engine", status: "Connected", latency_ms: 2, msgs_per_sec: 240_000, uptime_pct: 100.0, last_error: "None" },
        AlgoFeedStatus { name: "FIX Order Gateway", status: "Degraded", latency_ms: 240, msgs_per_sec: 1_200, uptime_pct: 97.85, last_error: "Session timeout x2" },
        AlgoFeedStatus { name: "Smart Order Router", status: "Connected", latency_ms: 5, msgs_per_sec: 3_800, uptime_pct: 99.96, last_error: "None" },
    ]
}

// ==================== AI RESEARCH ====================

/// One AI research summary item.
pub struct AIResearchItem {
    pub title: &'static str,
    pub source: &'static str,
    pub summary: &'static str,
    pub relevance: f64,
    pub tags: &'static [&'static str],
    pub date: &'static str,
}

pub fn sample_ai_research() -> Vec<AIResearchItem> {
    vec![
        AIResearchItem { title: "India Equity Strategy: H2 2026 Outlook", source: "Goldman Sachs", summary: "Overweight India; Nifty target 27,500. Earnings recovery broadens beyond financials.", relevance: 0.94, tags: &["Strategy", "Equity"], date: "2026-09-26" },
        AIResearchItem { title: "RBI Policy: Rate Cut Trajectory", source: "Morgan Stanley", summary: "Expect 25bp cut in October; terminal repo at 5.00% by Q1 2027.", relevance: 0.91, tags: &["Rates", "Policy"], date: "2026-09-25" },
        AIResearchItem { title: "IT Services: AI-Led Demand Inflection", source: "JP Morgan", summary: "GenAI TCV growing 3x yoy; margin headwinds offset by pricing gains.", relevance: 0.88, tags: &["Sectors", "IT"], date: "2026-09-24" },
        AIResearchItem { title: "Banking Credit Growth Sustainability", source: "Nomura", summary: "Credit growth to moderate to 10.5% by FY27; deposit repricing done.", relevance: 0.86, tags: &["Banking", "Credit"], date: "2026-09-22" },
        AIResearchItem { title: "Commodities: Gold Structural Bull Case", source: "UBS", summary: "Central bank buying + real rate decline supports Rs 1,15,000/10g target.", relevance: 0.82, tags: &["Commodities", "Gold"], date: "2026-09-20" },
        AIResearchItem { title: "FII Flows: Reversal Signals", source: "CLSA", summary: "Valuation gap vs EM peers narrows; expect FII return in Q4 2026.", relevance: 0.79, tags: &["Flows", "FII"], date: "2026-09-18" },
        AIResearchItem { title: "Auto Sector: EV Mix Shift", source: "Bernstein", summary: "PV recovery + 2W electrification drive 14% EPS CAGR over FY26-28.", relevance: 0.74, tags: &["Auto", "EV"], date: "2026-09-15" },
        AIResearchItem { title: "Pharma: US Generic Pricing Stabilises", source: "HSBC", summary: "Price erosion bottoms at -4%; complex generics drive margin expansion.", relevance: 0.71, tags: &["Pharma", "US Generics"], date: "2026-09-12" },
    ]
}

// ==================== INDIA DASHBOARD WIDGETS ====================

/// One customizable dashboard widget.
pub struct DashboardWidget {
    pub title: &'static str,
    pub value: &'static str,
    pub change_pct: f64,
    pub sparkline: &'static [f64],
}

pub fn sample_dashboard_widgets() -> Vec<DashboardWidget> {
    vec![
        DashboardWidget { title: "NIFTY 50", value: "24,842.10", change_pct: 0.62, sparkline: &[24_420.0, 24_510.0, 24_480.0, 24_620.0, 24_590.0, 24_710.0, 24_680.0, 24_760.0, 24_800.0, 24_842.1] },
        DashboardWidget { title: "SENSEX", value: "80,124.55", change_pct: 0.48, sparkline: &[79_400.0, 79_620.0, 79_540.0, 79_810.0, 79_760.0, 79_920.0, 79_880.0, 80_010.0, 80_060.0, 80_124.55] },
        DashboardWidget { title: "BANK NIFTY", value: "51,248.30", change_pct: 1.12, sparkline: &[50_400.0, 50_620.0, 50_580.0, 50_840.0, 50_790.0, 50_980.0, 51_050.0, 51_140.0, 51_190.0, 51_248.3] },
        DashboardWidget { title: "USD/INR", value: "86.42", change_pct: -0.18, sparkline: &[86.55, 86.50, 86.58, 86.48, 86.60, 86.45, 86.52, 86.40, 86.38, 86.42] },
        DashboardWidget { title: "GOLD (MCX)", value: "1,04,850", change_pct: 0.85, sparkline: &[103_200.0, 103_600.0, 103_450.0, 104_100.0, 103_900.0, 104_300.0, 104_150.0, 104_600.0, 104_700.0, 104_850.0] },
        DashboardWidget { title: "INDIA VIX", value: "13.84", change_pct: -4.20, sparkline: &[15.2, 14.8, 15.0, 14.5, 14.6, 14.2, 14.3, 14.0, 13.9, 13.84] },
        DashboardWidget { title: "10Y G-SEC", value: "6.71%", change_pct: -1.50, sparkline: &[6.82, 6.80, 6.78, 6.79, 6.76, 6.77, 6.74, 6.75, 6.72, 6.71] },
        DashboardWidget { title: "CRUDE (MCX)", value: "5,845", change_pct: -0.92, sparkline: &[5_920.0, 5_900.0, 5_940.0, 5_880.0, 5_910.0, 5_860.0, 5_890.0, 5_850.0, 5_860.0, 5_845.0] },
        DashboardWidget { title: "FII NET (Day)", value: "-1,842 cr", change_pct: -12.4, sparkline: &[-800.0, -1_200.0, -900.0, -1_600.0, -1_100.0, -1_900.0, -1_400.0, -1_700.0, -1_500.0, -1_842.0] },
        DashboardWidget { title: "DII NET (Day)", value: "+2,140 cr", change_pct: 8.6, sparkline: &[1_600.0, 1_800.0, 1_700.0, 2_000.0, 1_750.0, 2_100.0, 1_900.0, 2_050.0, 2_000.0, 2_140.0] },
        DashboardWidget { title: "ADV/DEC", value: "1.51", change_pct: 3.2, sparkline: &[1.32, 1.38, 1.35, 1.42, 1.40, 1.46, 1.44, 1.48, 1.50, 1.51] },
        DashboardWidget { title: "MIBOR 3M", value: "5.68%", change_pct: 0.4, sparkline: &[5.62, 5.64, 5.63, 5.66, 5.65, 5.67, 5.66, 5.68, 5.67, 5.68] },
    ]
}
