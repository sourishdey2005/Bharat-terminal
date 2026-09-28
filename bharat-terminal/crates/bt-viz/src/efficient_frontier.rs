//! Tier 3 #18 â€” Efficient Frontier (return x risk x Sharpe). Made by Sourish Dey.
//!
//! Renders a 2D projection (annualized return vs. annualized volatility,
//! points colored/sized by Sharpe ratio) of the classic Markowitz
//! mean-variance frontier â€” the standard way this is shown even in
//! "3D" Bloomberg-style tooling, since the third dimension (Sharpe) is
//! encoded visually rather than needing true 3D projection.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone, Copy)]
pub struct Portfolio {
    pub risk: f64,
    pub ret: f64,
    pub sharpe: f64,
}

/// Generates `n_samples` random long-only portfolios from a set of asset expected
/// returns and a covariance matrix, computing annualized return, risk and
/// Sharpe ratio for each - the standard Monte-Carlo approach to sketching
/// an efficient frontier without a full QP solver.
pub fn simulate_portfolios(
    expected_returns: &[f64],
    cov: &[Vec<f64>],
    risk_free: f64,
    n: usize,
    seed: u64,
) -> Vec<Portfolio> {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    let mut rng = StdRng::seed_from_u64(seed);
    let k = expected_returns.len();
    let mut out = Vec::with_capacity(n);

    for _ in 0..n {
        let raw: Vec<f64> = (0..k).map(|_| rng.gen_range(0.0001..1.0)).collect();
        let sum: f64 = raw.iter().sum();
        let w: Vec<f64> = raw.iter().map(|v| v / sum).collect();

        let ret: f64 = w.iter().zip(expected_returns).map(|(wi, ri)| wi * ri).sum();
        let mut var = 0.0;
        for i in 0..k {
            for j in 0..k {
                var += w[i] * w[j] * cov[i][j];
            }
        }
        let risk = var.max(0.0).sqrt();
        let sharpe = if risk > 1e-9 { (ret - risk_free) / risk } else { 0.0 };
        out.push(Portfolio { risk, ret, sharpe });
    }
    out
}

#[derive(Debug, Clone)]
pub struct EfficientFrontierConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for EfficientFrontierConfig {
    fn default() -> Self {
        Self {
            title: "Efficient Frontier".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl EfficientFrontierConfig {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    portfolios: &[Portfolio],
    cfg: &EfficientFrontierConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if portfolios.is_empty() {
        return Err(BtError::EmptySeries("portfolios".into()));
    }
    fill_background(&root, cfg.theme)?;

    let risk_max = portfolios.iter().map(|p| p.risk).fold(0.0_f64, f64::max);
    let ret_min = portfolios.iter().map(|p| p.ret).fold(f64::MAX, f64::min);
    let ret_max = portfolios.iter().map(|p| p.ret).fold(f64::MIN, f64::max);
    let sharpe_min = portfolios.iter().map(|p| p.sharpe).fold(f64::MAX, f64::min);
    let sharpe_max = portfolios.iter().map(|p| p.sharpe).fold(f64::MIN, f64::max);
    let sharpe_range = (sharpe_max - sharpe_min).max(1e-9);
    let pad = (ret_max - ret_min).max(0.01) * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(&cfg.title, (TITLE_FONT, 22).into_font().color(&cfg.theme.text()))
        .margin(15)
        .x_label_area_size(35)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..risk_max * 1.1, (ret_min - pad)..(ret_max + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .x_desc("Risk (annualized volatility)")
        .y_desc("Expected Return")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(portfolios.iter().map(|p| {
            let t = (p.sharpe - sharpe_min) / sharpe_range;
            let color = blend(cfg.theme.loss(), cfg.theme.profit(), t);
            Circle::new((p.risk, p.ret), 3, color.filled())
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Highlight the max-Sharpe portfolio (the tangency point).
    if let Some(best) = portfolios.iter().max_by(|a, b| a.sharpe.partial_cmp(&b.sharpe).unwrap()) {
        chart
            .draw_series(std::iter::once(Circle::new(
                (best.risk, best.ret),
                7,
                cfg.theme.accent().stroke_width(2),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
        chart
            .draw_series(std::iter::once(Text::new(
                format!("Max Sharpe: {:.2}", best.sharpe),
                (best.risk + risk_max * 0.02, best.ret),
                (LABEL_FONT, 13).into_font().color(&cfg.theme.accent()),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

fn blend(a: RGBColor, b: RGBColor, t: f64) -> RGBColor {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: u8, y: u8| (x as f64 + (y as f64 - x as f64) * t).round() as u8;
    RGBColor(lerp(a.0, b.0), lerp(a.1, b.1), lerp(a.2, b.2))
}

pub fn render_png(portfolios: &[Portfolio], cfg: &EfficientFrontierConfig, path: &str) -> Result<()> {
    render(png_root(path)?, portfolios, cfg)
}

pub fn render_svg(portfolios: &[Portfolio], cfg: &EfficientFrontierConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, portfolios, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_constructs() {
        let cfg = EfficientFrontierConfig::new();
        let _ = cfg;
    }
}
