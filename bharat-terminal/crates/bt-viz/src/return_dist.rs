// crates/bt-viz/src/return_dist.rs
// Author: Sourish Dey

//! Return distribution histogram with normal overlay. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

/// Histogram bins for return distribution.
pub fn histogram(returns: &[f64], n_bins: usize) -> Vec<(f64, usize)> {
    if returns.is_empty() || n_bins == 0 {
        return Vec::new();
    }
    let min = returns
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min);
    let max = returns
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let width = (max - min) / n_bins as f64;
    if width <= 0.0 {
        return vec![(min, returns.len())];
    }
    let mut bins = vec![(min + width * 0.5, 0usize); n_bins];
    for &r in returns {
        let idx = ((r - min) / width).floor() as usize;
        let idx = idx.min(n_bins - 1);
        bins[idx].1 += 1;
    }
    bins
}

fn normal_pdf(x: f64, mean: f64, std: f64) -> f64 {
    if std <= 0.0 {
        return 0.0;
    }
    let z = (x - mean) / std;
    (-0.5 * z * z).exp() / (std * (2.0 * std::f64::consts::PI).sqrt())
}

#[derive(Debug, Clone)]
pub struct ReturnDistConfig {
    pub title: String,
    pub theme: Theme,
    pub n_bins: usize,
}

impl Default for ReturnDistConfig {
    fn default() -> Self {
        Self {
            title: "Return Distribution".to_string(),
            theme: Theme::Dark,
            n_bins: 30,
        }
    }
}

impl ReturnDistConfig {
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
    pub fn n_bins(mut self, n: usize) -> Self {
        self.n_bins = n.max(5);
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &ReturnDistConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    let returns = series.returns();
    if returns.len() < 5 {
        return Err(BtError::InvalidInput("series too short".into()));
    }
    fill_background(&root, cfg.theme)?;

    let bins = histogram(&returns, cfg.n_bins);
    let max_count = bins.iter().map(|(_, c)| *c).max().unwrap_or(1);
    let bin_width = if bins.len() > 1 {
        bins[1].0 - bins[0].0
    } else {
        1.0
    };

    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let std = (returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>()
        / returns.len() as f64)
        .sqrt();

    let mut chart = ChartBuilder::on(&root)
        .caption(
            &cfg.title,
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(15)
        .x_label_area_size(35)
        .y_label_area_size(55)
        .build_cartesian_2d(
            bins.first().map(|(c, _)| *c).unwrap_or(0.0)
                - bin_width
                ..bins.last().map(|(c, _)| *c).unwrap_or(0.0) + bin_width,
            0f64..max_count as f64 * 1.15,
        )
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .x_desc("Return")
        .y_desc("Frequency")
        .label_style((LABEL_FONT, 11).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Histogram bars
    chart
        .draw_series(bins.iter().map(|(center, count)| {
            let x0 = center - bin_width / 2.0;
            let x1 = center + bin_width / 2.0;
            Rectangle::new(
                [(x0, 0.0), (x1, *count as f64)],
                cfg.theme.accent().mix(0.6).filled(),
            )
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Normal overlay
    let overlay_pts: Vec<(f64, f64)> = bins
        .iter()
        .map(|(c, _)| (*c, normal_pdf(*c, mean, std) * returns.len() as f64 * bin_width))
        .collect();

    chart
        .draw_series(LineSeries::new(
            overlay_pts,
            cfg.theme.profit().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Normal fit")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 15, y)],
                cfg.theme.profit().stroke_width(2),
            )
        });

    // Stats annotation
    let stats_text = format!(
        "Mean: {:.4}%  Std: {:.4}%  Skew: {:.2}  Kurt: {:.2}",
        mean * 100.0,
        std * 100.0,
        bt_analytics::skewness(&returns),
        bt_analytics::kurtosis(&returns),
    );
    root.draw(&Text::new(
        stats_text,
        (50, 40),
        (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
    ))
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

pub fn render_png(series: &OhlcvSeries, cfg: &ReturnDistConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &ReturnDistConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 200, 1, 100.0);
        let cfg = ReturnDistConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_return_dist.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
