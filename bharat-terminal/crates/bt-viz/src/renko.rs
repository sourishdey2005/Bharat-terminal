// crates/bt-viz/src/renko.rs
// Author: Sourish Dey

//! Tier 1 #7 â€” Renko Chart (fixed-brick price chart).
//! Made by Sourish Dey.

use bt_analytics::renko;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct RenkoConfig {
    pub title: String,
    pub theme: Theme,
    pub brick_size: f64,
}

impl Default for RenkoConfig {
    fn default() -> Self {
        Self {
            title: "Renko".to_string(),
            theme: Theme::Dark,
            brick_size: 10.0,
        }
    }
}

impl RenkoConfig {
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

    pub fn brick_size(mut self, size: f64) -> Self {
        self.brick_size = size.max(0.01);
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &RenkoConfig,
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
        // Draw empty chart with message
        let mut chart = ChartBuilder::on(&root)
            .caption(
                format!("{} â€” {} (No bricks formed)", cfg.title, series.symbol),
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

    // Find price range of bricks
    let brick_prices: Vec<f64> = bricks.iter().map(|b| b.0).collect();
    let low = brick_prices.iter().fold(f64::MAX, |a, &b| a.min(b));
    let high = brick_prices
        .iter()
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let pad = (high - low).max(1.0) * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!(
                "{} â€” {} (Brick: {:.2})",
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

    // Renko uses brick index for x-axis, but we map to time
    // For simplicity, distribute bricks evenly across time range
    let brick_width = (t_max - t_min) / bricks.len().max(1) as f64;

    for (idx, (price, direction)) in bricks.iter().enumerate() {
        let x = t_min + (idx as f64 + 0.5) * brick_width;
        let color = if *direction > 0 {
            cfg.theme.profit()
        } else {
            cfg.theme.loss()
        };

        chart
            .draw_series(std::iter::once(Rectangle::new(
                [
                    (x - brick_width * 0.4, price - cfg.brick_size / 2.0),
                    (x + brick_width * 0.4, price + cfg.brick_size / 2.0),
                ],
                color.filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &RenkoConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &RenkoConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_constructs() {
        let cfg = RenkoConfig::new();
        let _ = cfg;
    }
}
