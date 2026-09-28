// crates/bt-viz/src/tri_stick_renko.rs
// Author: Sourish Dey

//! Renko bricks rendered as isosceles triangles. Made by Sourish Dey.

use bt_analytics::renko;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct TriStickRenkoConfig {
    pub title: String,
    pub theme: Theme,
    pub brick_size: f64,
}

impl Default for TriStickRenkoConfig {
    fn default() -> Self {
        Self {
            title: "Renko Triangles".to_string(),
            theme: Theme::Dark,
            brick_size: 10.0,
        }
    }
}

impl TriStickRenkoConfig {
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

    pub fn brick_size(mut self, s: f64) -> Self {
        self.brick_size = s.max(0.01);
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &TriStickRenkoConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;

    let bricks = renko(series, cfg.brick_size);

    if bricks.is_empty() {
        let mut chart = ChartBuilder::on(&root)
            .caption(
                format!("{} — {} (No bricks formed)", cfg.title, series.symbol),
                (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
            )
            .margin(10)
            .build_cartesian_2d(t_min..t_max, 0.0..1.0)
            .map_err(|e| BtError::Render(e.to_string()))?;

        chart
            .configure_mesh()
            .disable_mesh()
            .draw()
            .map_err(|e| BtError::Render(e.to_string()))?;

        draw_footer(&root, cfg.theme)?;
        root.present().map_err(|e| BtError::Render(e.to_string()))?;
        return Ok(());
    }

    let brick_prices: Vec<f64> = bricks.iter().map(|b| b.0).collect();
    let low = brick_prices.iter().fold(f64::MAX, |a, &b| a.min(b));
    let high = brick_prices
        .iter()
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let pad = (high - low).max(1.0) * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!(
                "{} — {} (Brick: {:.2})",
                cfg.title, series.symbol, cfg.brick_size
            ),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, (low - pad)..(high + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let slot = (t_max - t_min) / bricks.len().max(1) as f64;
    let tri_h = cfg.brick_size * 0.6;

    for (idx, (price, direction)) in bricks.iter().enumerate() {
        let x = t_min + (idx as f64 + 0.5) * slot;
        let color = if *direction > 0 {
            cfg.theme.profit()
        } else {
            cfg.theme.loss()
        };

        let (apex_y, base_y) = if *direction > 0 {
            (price + tri_h / 2.0, price - tri_h / 2.0)
        } else {
            (price - tri_h / 2.0, price + tri_h / 2.0)
        };

        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![
                    (x - slot * 0.35, base_y),
                    (x + slot * 0.35, base_y),
                    (x, apex_y),
                ],
                color.filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &TriStickRenkoConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &TriStickRenkoConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = TriStickRenkoConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_tri_stick_renko.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
