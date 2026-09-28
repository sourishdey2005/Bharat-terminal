// crates/bt-viz/src/iv_rank.rs
// Author: Sourish Dey

//! Implied vol rank/percentile. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct IvRankConfig {
    pub title: String,
    pub theme: Theme,
    pub window: usize,
    pub iv_series: Vec<f64>,
}

impl Default for IvRankConfig {
    fn default() -> Self {
        Self {
            title: "IV Rank / Percentile".to_string(),
            theme: Theme::Dark,
            window: 252,
            iv_series: vec![],
        }
    }
}

impl IvRankConfig {
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
    pub fn window(mut self, w: usize) -> Self {
        self.window = w;
        self
    }
    pub fn iv_series(mut self, v: Vec<f64>) -> Self {
        self.iv_series = v;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &IvRankConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let iv = if cfg.iv_series.is_empty() {
        let returns = series.returns();
        let n = returns.len() as f64;
        let mean = returns.iter().sum::<f64>() / n;
        let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n;
        vec![variance.sqrt() * (252.0_f64).sqrt() * 100.0]
    } else {
        cfg.iv_series.clone()
    };

    if iv.len() < 2 {
        return Err(BtError::InvalidInput("insufficient IV data".into()));
    }

    let iv_min = iv.iter().cloned().fold(f64::MAX, f64::min);
    let iv_max = iv.iter().cloned().fold(f64::MIN, f64::max);
    let current = iv[iv.len() - 1];
    let rank = if iv_max > iv_min {
        (current - iv_min) / (iv_max - iv_min) * 100.0
    } else {
        50.0
    };

    let n = iv.len() as f64;
    let pad = (iv_max - iv_min).max(0.1) * 0.15;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            &cfg.title,
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(15)
        .x_label_area_size(35)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..n, (iv_min - pad)..(iv_max + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .x_desc("Observation")
        .y_desc("Implied Vol (%)")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let points: Vec<(f64, f64)> = iv.iter().enumerate().map(|(i, v)| (i as f64, *v)).collect();

    chart
        .draw_series(LineSeries::new(
            points.clone(),
            cfg.theme.accent().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(0.0, current), (n, current)],
            cfg.theme.info().stroke_width(2),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(std::iter::once(Text::new(
            format!("IV Rank: {:.1}%", rank),
            (n * 0.05, iv_max + pad * 0.5),
            (LABEL_FONT, 14).into_font().color(&cfg.theme.accent()),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &IvRankConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &IvRankConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let mut cfg = IvRankConfig::new().theme(Theme::Dark);
        let iv: Vec<f64> = (1..=30).map(|i| 15.0 + (i as f64 * 0.5)).collect();
        cfg.iv_series = iv;
        let path = std::env::temp_dir()
            .join("bt_test_iv_rank.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
