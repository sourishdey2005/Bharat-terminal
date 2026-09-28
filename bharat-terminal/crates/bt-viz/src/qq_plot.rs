// crates/bt-viz/src/qq_plot.rs
// Author: Sourish Dey

//! Q-Q plot vs normal distribution. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

/// Inverse normal CDF (probit) via rational approximation.
fn norm_ppf(p: f64) -> f64 {
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    // Beasley-Springer-Moro approximation
    let a = [
        2.50662823884,
        -18.61500062529,
        41.39119773534,
        -25.44106049637,
    ];
    let b = [
        -8.47351093090,
        23.08336743743,
        -21.06224101826,
        3.13082909833,
    ];
    let c = [
        0.3374754822726147,
        0.9761690190917186,
        0.1607979714918209,
        0.0276438810333863,
        0.0038405729373609,
        0.0003951896511919,
        0.0000321767881768,
        0.0000002888167364,
        0.0000003960315187,
    ];

    let p = p.clamp(1e-10, 1.0 - 1e-10);
    let y = if p < 0.5 { p } else { 1.0 - p };
    let r = (-2.0 * y.ln()).sqrt();
    let num = (((a[3] * r + a[2]) * r + a[1]) * r + a[0]) * r;
    let den = (((b[3] * r + b[2]) * r + b[1]) * r + b[0]) * r + 1.0;
    let x = num / den;

    if p < 0.5 {
        -x
    } else {
        x
    }
}

/// Computes Q-Q plot data: (theoretical_quantile, sorted_sample).
pub fn qq_data(returns: &[f64]) -> Vec<(f64, f64)> {
    let mut sorted = returns.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len();
    (0..n)
        .map(|i| {
            let p = (i as f64 + 0.5) / n as f64;
            (norm_ppf(p), sorted[i])
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct QQPlotConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for QQPlotConfig {
    fn default() -> Self {
        Self {
            title: "Q-Q Plot".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl QQPlotConfig {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = t.into();
        self
    }
    pub fn theme(mut self, t: Theme) -> Self {
        self.theme = t;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &QQPlotConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    let returns = series.returns();
    if returns.len() < 5 {
        return Err(BtError::InvalidInput("series too short for QQ".into()));
    }
    fill_background(&root, cfg.theme)?;

    let qq = qq_data(&returns);
    let x_min = qq
        .iter()
        .map(|(t, _)| *t)
        .fold(f64::INFINITY, f64::min);
    let x_max = qq
        .iter()
        .map(|(t, _)| *t)
        .fold(f64::NEG_INFINITY, f64::max);
    let y_min = qq
        .iter()
        .map(|(_, s)| *s)
        .fold(f64::INFINITY, f64::min);
    let y_max = qq
        .iter()
        .map(|(_, s)| *s)
        .fold(f64::NEG_INFINITY, f64::max);

    let x_pad = (x_max - x_min) * 0.05 + 0.01;
    let y_pad = (y_max - y_min).abs() * 0.1 + 0.001;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} - Normal Q-Q", cfg.title),
            (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
        )
        .margin(15)
        .x_label_area_size(35)
        .y_label_area_size(55)
        .build_cartesian_2d(
            x_min - x_pad..x_max + x_pad,
            y_min - y_pad..y_max + y_pad,
        )
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .x_desc("Theoretical Quantiles")
        .y_desc("Sample Quantiles")
        .label_style((LABEL_FONT, 11).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    // 45-degree reference line
    let ref_min = x_min.min(y_min);
    let ref_max = x_max.max(y_max);
    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(ref_min, ref_min), (ref_max, ref_max)],
            cfg.theme.info().mix(0.6).stroke_width(1),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Normal fit")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 15, y)],
                cfg.theme.info().mix(0.6).stroke_width(1),
            )
        });

    // QQ points
    chart
        .draw_series(
            qq.iter()
                .map(|&(t, s)| Circle::new((t, s), 4, cfg.theme.accent().mix(0.8).filled())),
        )
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_series_labels()
        .background_style(cfg.theme.background())
        .label_font((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &QQPlotConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &QQPlotConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = QQPlotConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_qq.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
