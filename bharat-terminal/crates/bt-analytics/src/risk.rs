// crates/bt-analytics/src/risk.rs
// Author: Sourish Dey

//! Risk and portfolio analytics over return series.

use bt_core::OhlcvSeries;

/// Maximum drawdown (peak-to-trough) as a positive percentage.
pub fn max_drawdown(series: &OhlcvSeries) -> f64 {
    let closes: Vec<f64> = series.candles.iter().map(|c| c.close).collect();
    if closes.is_empty() {
        return 0.0;
    }

    let mut peak = closes[0];
    let mut max_dd = 0.0;

    for &price in &closes {
        if price > peak {
            peak = price;
        }
        let dd = (peak - price) / peak;
        if dd > max_dd {
            max_dd = dd;
        }
    }

    max_dd * 100.0
}

/// Drawdown series (running drawdown at each point).
pub fn drawdown_series(series: &OhlcvSeries) -> Vec<f64> {
    let closes: Vec<f64> = series.candles.iter().map(|c| c.close).collect();
    let mut result = Vec::with_capacity(closes.len());

    if closes.is_empty() {
        return result;
    }

    let mut peak = closes[0];
    for &price in &closes {
        if price > peak {
            peak = price;
        }
        let dd = if peak > 0.0 { (peak - price) / peak * 100.0 } else { 0.0 };
        result.push(dd);
    }

    result
}

/// Sharpe ratio (annualized).
/// `returns` should be period returns (e.g., daily returns).
/// `risk_free` is annualized risk-free rate (e.g., 0.05 for 5%).
pub fn sharpe(returns: &[f64], risk_free: f64, periods_per_year: usize) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }

    let n = returns.len() as f64;
    let mean = returns.iter().sum::<f64>() / n;
    let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n;
    let std_dev = variance.sqrt();

    if std_dev == 0.0 {
        return 0.0;
    }

    let rf_per_period = risk_free / periods_per_year as f64;
    let excess = mean - rf_per_period;

    (excess / std_dev) * (periods_per_year as f64).sqrt()
}

/// Sortino ratio (annualized, downside deviation).
pub fn sortino(returns: &[f64], risk_free: f64, periods_per_year: usize) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }

    let n = returns.len() as f64;
    let rf_per_period = risk_free / periods_per_year as f64;

    let downside_returns: Vec<f64> = returns
        .iter()
        .map(|r| (r - rf_per_period).min(0.0))
        .collect();

    let downside_variance = downside_returns.iter().map(|r| r.powi(2)).sum::<f64>() / n;
    let downside_dev = downside_variance.sqrt();

    if downside_dev == 0.0 {
        return 0.0;
    }

    let mean = returns.iter().sum::<f64>() / n;
    let excess = mean - rf_per_period;

    (excess / downside_dev) * (periods_per_year as f64).sqrt()
}

/// Historical Value at Risk (VaR) at given confidence level.
/// Returns positive number representing loss at confidence.
pub fn var_historical(returns: &[f64], confidence: f64) -> f64 {
    if returns.is_empty() || confidence <= 0.0 || confidence >= 1.0 {
        return 0.0;
    }

    let mut sorted = returns.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let index = ((1.0 - confidence) * sorted.len() as f64).floor() as usize;
    let index = index.min(sorted.len().saturating_sub(1));

    -sorted[index] * 100.0 // Return as positive percentage
}

/// Conditional Value at Risk (Expected Shortfall).
/// Average of returns beyond VaR threshold.
pub fn cvar(returns: &[f64], confidence: f64) -> f64 {
    if returns.is_empty() || confidence <= 0.0 || confidence >= 1.0 {
        return 0.0;
    }

    let var = var_historical(returns, confidence) / 100.0; // Convert back to decimal

    let tail_losses: Vec<f64> = returns
        .iter()
        .filter(|&&r| r <= -var)
        .map(|&r| -r)
        .collect();

    if tail_losses.is_empty() {
        return var * 100.0;
    }

    tail_losses.iter().sum::<f64>() / tail_losses.len() as f64 * 100.0
}

/// Rolling Sharpe ratio over a window.
pub fn rolling_sharpe(
    returns: &[f64],
    window: usize,
    risk_free: f64,
    periods_per_year: usize,
) -> Vec<f64> {
    if window == 0 || returns.len() < window {
        return vec![f64::NAN; returns.len()];
    }

    let mut result = vec![f64::NAN; returns.len()];

    for i in (window - 1)..returns.len() {
        let window_returns = &returns[i + 1 - window..=i];
        result[i] = sharpe(window_returns, risk_free, periods_per_year);
    }

    result
}

/// Rolling Sortino ratio over a window.
pub fn rolling_sortino(
    returns: &[f64],
    window: usize,
    risk_free: f64,
    periods_per_year: usize,
) -> Vec<f64> {
    if window == 0 || returns.len() < window {
        return vec![f64::NAN; returns.len()];
    }

    let mut result = vec![f64::NAN; returns.len()];

    for i in (window - 1)..returns.len() {
        let window_returns = &returns[i + 1 - window..=i];
        result[i] = sortino(window_returns, risk_free, periods_per_year);
    }

    result
}

/// Rolling maximum drawdown over a window.
pub fn rolling_max_drawdown(series: &OhlcvSeries, window: usize) -> Vec<f64> {
    if window == 0 || series.candles.len() < window {
        return vec![f64::NAN; series.candles.len()];
    }

    let n = series.candles.len();
    let mut result = vec![f64::NAN; n];

    for i in (window - 1)..n {
        let window_candles = &series.candles[i + 1 - window..=i];
        let sub_series = OhlcvSeries::new("", window_candles.to_vec());
        result[i] = max_drawdown(&sub_series);
    }

    result
}

/// Beta of asset vs benchmark (regression slope).
pub fn beta(asset_returns: &[f64], benchmark_returns: &[f64]) -> f64 {
    if asset_returns.len() != benchmark_returns.len() || asset_returns.is_empty() {
        return 0.0;
    }

    let n = asset_returns.len() as f64;
    let mean_asset = asset_returns.iter().sum::<f64>() / n;
    let mean_bench = benchmark_returns.iter().sum::<f64>() / n;

    let mut cov = 0.0;
    let mut var_bench = 0.0;

    for i in 0..asset_returns.len() {
        let da = asset_returns[i] - mean_asset;
        let db = benchmark_returns[i] - mean_bench;
        cov += da * db;
        var_bench += db * db;
    }

    if var_bench == 0.0 {
        return 0.0;
    }

    cov / var_bench
}

/// Alpha (Jensen's alpha) - excess return over CAPM.
pub fn alpha(
    asset_returns: &[f64],
    benchmark_returns: &[f64],
    risk_free: f64,
    periods_per_year: usize,
) -> f64 {
    let b = beta(asset_returns, benchmark_returns);
    let n = asset_returns.len() as f64;
    let mean_asset = asset_returns.iter().sum::<f64>() / n;
    let mean_bench = benchmark_returns.iter().sum::<f64>() / n;
    let rf_per_period = risk_free / periods_per_year as f64;

    (mean_asset - rf_per_period) - b * (mean_bench - rf_per_period)
}

/// Correlation coefficient between two return series.
pub fn correlation(returns_a: &[f64], returns_b: &[f64]) -> f64 {
    if returns_a.len() != returns_b.len() || returns_a.is_empty() {
        return 0.0;
    }

    let n = returns_a.len() as f64;
    let mean_a = returns_a.iter().sum::<f64>() / n;
    let mean_b = returns_b.iter().sum::<f64>() / n;

    let mut cov = 0.0;
    let mut var_a = 0.0;
    let mut var_b = 0.0;

    for i in 0..returns_a.len() {
        let da = returns_a[i] - mean_a;
        let db = returns_b[i] - mean_b;
        cov += da * db;
        var_a += da * da;
        var_b += db * db;
    }

    let denom = (var_a * var_b).sqrt();
    if denom == 0.0 {
        return 0.0;
    }

    cov / denom
}

/// Correlation matrix for multiple return series.
/// Input: Vec of (symbol, returns)
/// Output: Vec of Vec of correlations (square matrix)
pub fn correlation_matrix(series: &[(String, Vec<f64>)]) -> Vec<Vec<f64>> {
    let n = series.len();
    let mut matrix = vec![vec![0.0; n]; n];

    for i in 0..n {
        for j in 0..n {
            if i == j {
                matrix[i][j] = 1.0;
            } else {
                matrix[i][j] = correlation(&series[i].1, &series[j].1);
            }
        }
    }

    matrix
}

/// Rolling correlation between two series.
pub fn rolling_correlation(
    returns_a: &[f64],
    returns_b: &[f64],
    window: usize,
) -> Vec<f64> {
    if window == 0 || returns_a.len() < window || returns_b.len() < window {
        return vec![f64::NAN; returns_a.len()];
    }

    let n = returns_a.len();
    let mut result = vec![f64::NAN; n];

    for i in (window - 1)..n {
        let a = &returns_a[i + 1 - window..=i];
        let b = &returns_b[i + 1 - window..=i];
        result[i] = correlation(a, b);
    }

    result
}

/// Covariance matrix for returns (for portfolio optimization).
pub fn covariance_matrix(series: &[(String, Vec<f64>)]) -> Vec<Vec<f64>> {
    let n = series.len();
    let mut matrix = vec![vec![0.0; n]; n];

    for i in 0..n {
        for j in 0..n {
            let ri = &series[i].1;
            let rj = &series[j].1;
            if ri.len() != rj.len() || ri.is_empty() {
                matrix[i][j] = 0.0;
                continue;
            }

            let n_samples = ri.len() as f64;
            let mean_i = ri.iter().sum::<f64>() / n_samples;
            let mean_j = rj.iter().sum::<f64>() / n_samples;

            let mut cov = 0.0;
            for k in 0..ri.len() {
                cov += (ri[k] - mean_i) * (rj[k] - mean_j);
            }
            matrix[i][j] = cov / n_samples;
        }
    }

    matrix
}

/// Efficient frontier: given expected returns and covariance matrix,
/// return (risk, return) pairs for portfolios on the frontier.
/// This is a simplified implementation using minimum variance portfolios
/// for different target returns.
pub fn efficient_frontier(
    expected_returns: &[f64],
    cov_matrix: &[Vec<f64>],
    n_points: usize,
) -> Vec<(f64, f64)> {
    let n = expected_returns.len();
    if n == 0 || n_points == 0 {
        return Vec::new();
    }

    // Find min and max expected return
    let min_ret = expected_returns.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_ret = expected_returns.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    if min_ret >= max_ret {
        return Vec::new();
    }

    let mut frontier = Vec::with_capacity(n_points);

    for k in 0..n_points {
        let target = min_ret + (max_ret - min_ret) * (k as f64 / (n_points - 1) as f64);

        // Simplified: equal weight portfolio scaled to target return
        // In practice, would solve quadratic optimization
        let weight = target / expected_returns.iter().sum::<f64>() * n as f64;

        let mut port_var = 0.0;
        for i in 0..n {
            for j in 0..n {
                port_var += weight * weight * cov_matrix[i][j];
            }
        }

        frontier.push((port_var.sqrt() * 100.0, target * 100.0));
    }

    frontier
}

/// Information ratio (active return / tracking error).
pub fn information_ratio(
    asset_returns: &[f64],
    benchmark_returns: &[f64],
    periods_per_year: usize,
) -> f64 {
    if asset_returns.len() != benchmark_returns.len() || asset_returns.is_empty() {
        return 0.0;
    }

    let n = asset_returns.len();
    let mut active_returns = Vec::with_capacity(n);

    for i in 0..n {
        active_returns.push(asset_returns[i] - benchmark_returns[i]);
    }

    let mean = active_returns.iter().sum::<f64>() / n as f64;
    let var = active_returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n as f64;
    let tracking_error = var.sqrt();

    if tracking_error == 0.0 {
        return 0.0;
    }

    (mean / tracking_error) * (periods_per_year as f64).sqrt()
}

/// Treynor ratio (excess return / beta).
pub fn treynor(
    asset_returns: &[f64],
    benchmark_returns: &[f64],
    risk_free: f64,
    periods_per_year: usize,
) -> f64 {
    let b = beta(asset_returns, benchmark_returns);
    if b == 0.0 {
        return 0.0;
    }

    let n = asset_returns.len() as f64;
    let mean_asset = asset_returns.iter().sum::<f64>() / n;
    let rf_per_period = risk_free / periods_per_year as f64;

    (mean_asset - rf_per_period) / b * (periods_per_year as f64).sqrt()
}

/// Calmar ratio (annualized return / max drawdown).
pub fn calmar(series: &OhlcvSeries, periods_per_year: usize) -> f64 {
    let returns = series.returns();
    if returns.is_empty() {
        return 0.0;
    }

    let n = returns.len() as f64;
    let annual_return = (returns.iter().sum::<f64>() / n) * periods_per_year as f64;
    let max_dd = max_drawdown(series) / 100.0;

    if max_dd == 0.0 {
        return 0.0;
    }

    annual_return / max_dd
}

/// Rolling volatility (annualized).
pub fn rolling_volatility(returns: &[f64], window: usize, periods_per_year: usize) -> Vec<f64> {
    if window == 0 || returns.len() < window {
        return vec![f64::NAN; returns.len()];
    }

    let mut result = vec![f64::NAN; returns.len()];

    for i in (window - 1)..returns.len() {
        let window_returns = &returns[i + 1 - window..=i];
        let n = window_returns.len() as f64;
        let mean = window_returns.iter().sum::<f64>() / n;
        let var = window_returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n;
        result[i] = var.sqrt() * (periods_per_year as f64).sqrt();
    }

    result
}

/// Kurtosis of returns (excess kurtosis).
pub fn kurtosis(returns: &[f64]) -> f64 {
    if returns.len() < 4 {
        return 0.0;
    }

    let n = returns.len() as f64;
    let mean = returns.iter().sum::<f64>() / n;
    let var = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n;
    let std = var.sqrt();

    if std == 0.0 {
        return 0.0;
    }

    let m4 = returns.iter().map(|r| ((r - mean) / std).powi(4)).sum::<f64>() / n;

    m4 - 3.0
}

/// Skewness of returns.
pub fn skewness(returns: &[f64]) -> f64 {
    if returns.len() < 3 {
        return 0.0;
    }

    let n = returns.len() as f64;
    let mean = returns.iter().sum::<f64>() / n;
    let var = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n;
    let std = var.sqrt();

    if std == 0.0 {
        return 0.0;
    }

    let m3 = returns.iter().map(|r| ((r - mean) / std).powi(3)).sum::<f64>() / n;

    m3
}

/// Rolling skewness and kurtosis.
pub fn rolling_moments(returns: &[f64], window: usize) -> (Vec<f64>, Vec<f64>) {
    if window == 0 || returns.len() < window {
        let n = returns.len();
        return (vec![f64::NAN; n], vec![f64::NAN; n]);
    }

    let mut skew = vec![f64::NAN; returns.len()];
    let mut kurt = vec![f64::NAN; returns.len()];

    for i in (window - 1)..returns.len() {
        let window_returns = &returns[i + 1 - window..=i];
        skew[i] = skewness(window_returns);
        kurt[i] = kurtosis(window_returns);
    }

    (skew, kurt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::{Candle, OhlcvSeries};

    fn test_series() -> OhlcvSeries {
        let candles = vec![
            Candle::new(0.0, 100.0, 102.0, 99.0, 101.0, 1000.0),
            Candle::new(1.0, 101.0, 103.0, 100.0, 102.0, 1100.0),
            Candle::new(2.0, 102.0, 104.0, 101.0, 103.0, 1200.0),
            Candle::new(3.0, 103.0, 105.0, 102.0, 104.0, 1300.0),
            Candle::new(4.0, 104.0, 106.0, 103.0, 105.0, 1400.0),
            Candle::new(5.0, 105.0, 107.0, 104.0, 106.0, 1500.0),
            Candle::new(6.0, 106.0, 108.0, 105.0, 107.0, 1600.0),
            Candle::new(7.0, 107.0, 109.0, 106.0, 108.0, 1700.0),
            Candle::new(8.0, 108.0, 110.0, 107.0, 109.0, 1800.0),
            Candle::new(9.0, 109.0, 111.0, 108.0, 110.0, 1900.0),
        ];
        OhlcvSeries::new("TEST", candles)
    }

    fn test_returns() -> Vec<f64> {
        vec![0.01, 0.02, -0.01, 0.015, -0.005, 0.02, 0.01, -0.01, 0.015, 0.005]
    }

    #[test]
    fn test_max_drawdown() {
        let s = test_series();
        let dd = max_drawdown(&s);
        assert!(dd >= 0.0);
    }

    #[test]
    fn test_drawdown_series() {
        let s = test_series();
        let dd = drawdown_series(&s);
        assert_eq!(dd.len(), 10);
        for v in &dd {
            assert!(*v >= 0.0);
        }
    }

    #[test]
    fn test_sharpe() {
        let r = test_returns();
        let s = sharpe(&r, 0.05, 252);
        assert!(s.is_finite());
    }

    #[test]
    fn test_sortino() {
        let r = test_returns();
        let s = sortino(&r, 0.05, 252);
        assert!(s.is_finite());
    }

    #[test]
    fn test_var_historical() {
        let r = test_returns();
        let var = var_historical(&r, 0.95);
        assert!(var >= 0.0);
    }

    #[test]
    fn test_cvar() {
        let r = test_returns();
        let cvar_val = cvar(&r, 0.95);
        let var_val = var_historical(&r, 0.95);
        assert!(cvar_val >= var_val); // CVaR >= VaR
    }

    #[test]
    fn test_rolling_sharpe() {
        let r = test_returns();
        let rs = rolling_sharpe(&r, 5, 0.05, 252);
        assert_eq!(rs.len(), 10);
    }

    #[test]
    fn test_rolling_sortino() {
        let r = test_returns();
        let rs = rolling_sortino(&r, 5, 0.05, 252);
        assert_eq!(rs.len(), 10);
    }

    #[test]
    fn test_rolling_max_dd() {
        let s = test_series();
        let rdd = rolling_max_drawdown(&s, 5);
        assert_eq!(rdd.len(), 10);
    }

    #[test]
    fn test_beta() {
        let ra = test_returns();
        let rb: Vec<f64> = ra.iter().map(|x| x * 1.2).collect();
        let b = beta(&ra, &rb);
        assert!((b - 1.0 / 1.2).abs() < 0.01); // beta of ra vs 1.2*ra = 1/1.2
    }

    #[test]
    fn test_alpha() {
        let ra = test_returns();
        let rb: Vec<f64> = ra.iter().map(|x| x * 1.2).collect();
        let a = alpha(&ra, &rb, 0.05, 252);
        assert!(a.is_finite());
    }

    #[test]
    fn test_correlation() {
        let ra = test_returns();
        let rb = ra.clone();
        let c = correlation(&ra, &rb);
        assert!((c - 1.0).abs() < 0.001);

        let rc: Vec<f64> = ra.iter().map(|x| -x).collect();
        let c2 = correlation(&ra, &rc);
        assert!((c2 + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_correlation_matrix() {
        let series = vec![
            ("A".to_string(), test_returns()),
            ("B".to_string(), test_returns()),
        ];
        let mat = correlation_matrix(&series);
        assert_eq!(mat.len(), 2);
        assert_eq!(mat[0].len(), 2);
        assert!((mat[0][0] - 1.0).abs() < 0.001);
        assert!((mat[0][1] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_rolling_correlation() {
        let ra = test_returns();
        let rb = ra.clone();
        let rc = rolling_correlation(&ra, &rb, 5);
        assert_eq!(rc.len(), 10);
        for v in &rc[4..] {
            assert!((*v - 1.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_covariance_matrix() {
        let series = vec![
            ("A".to_string(), test_returns()),
            ("B".to_string(), test_returns()),
        ];
        let mat = covariance_matrix(&series);
        assert_eq!(mat.len(), 2);
        assert_eq!(mat[0].len(), 2);
    }

    #[test]
    fn test_efficient_frontier() {
        let expected = vec![0.1, 0.15, 0.08];
        let cov = vec![
            vec![0.04, 0.01, 0.005],
            vec![0.01, 0.06, 0.01],
            vec![0.005, 0.01, 0.03],
        ];
        let frontier = efficient_frontier(&expected, &cov, 10);
        assert_eq!(frontier.len(), 10);
    }

    #[test]
    fn test_information_ratio() {
        let ra = test_returns();
        let rb: Vec<f64> = ra.iter().map(|x| x * 0.9).collect();
        let ir = information_ratio(&ra, &rb, 252);
        assert!(ir.is_finite());
    }

    #[test]
    fn test_treynor() {
        let ra = test_returns();
        let rb: Vec<f64> = ra.iter().map(|x| x * 1.2).collect();
        let t = treynor(&ra, &rb, 0.05, 252);
        assert!(t.is_finite());
    }

    #[test]
    fn test_calmar() {
        let s = test_series();
        let c = calmar(&s, 252);
        assert!(c.is_finite());
    }

    #[test]
    fn test_rolling_volatility() {
        let r = test_returns();
        let rv = rolling_volatility(&r, 5, 252);
        assert_eq!(rv.len(), 10);
    }

    #[test]
    fn test_kurtosis() {
        let r = test_returns();
        let k = kurtosis(&r);
        assert!(k.is_finite());
    }

    #[test]
    fn test_skewness() {
        let r = test_returns();
        let s = skewness(&r);
        assert!(s.is_finite());
    }

    #[test]
    fn test_rolling_moments() {
        let r = test_returns();
        let (sk, kt) = rolling_moments(&r, 5);
        assert_eq!(sk.len(), 10);
        assert_eq!(kt.len(), 10);
    }
}