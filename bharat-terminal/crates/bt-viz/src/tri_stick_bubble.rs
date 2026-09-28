// crates/bt-viz/src/tri_stick_bubble.rs
// Author: Sourish Dey

//! Triangles + circles showing RSI/MACD values. Made by Sourish Dey.

use bt_analytics::{macd, rsi};
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct TriStickBubbleConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for TriStickBubbleConfig {
    fn default() -> Self {
        Self {
            title: "RSI/MACD Bubbles".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl TriStickBubbleConfig {
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
    cfg: &TriStickBubbleConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;
    let low = series
        .candles
        .iter()
        .map(|c| c.low)
        .fold(f64::MAX, f64::min);
    let high = series
        .candles
        .iter()
        .map(|c| c.high)
        .fold(f64::MIN, f64::max);
    let pad = (high - low) * 0.05;

    let rsi_vals = rsi(series, 14);
    let (macd_line, _signal, _hist) = macd(series);

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — {}", cfg.title, series.symbol),
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

    let half_w = ((t_max - t_min) / series.candles.len() as f64).max(0.3) * 0.3;

    for i in 0..series.candles.len() {
        let c = &series.candles[i];
        let color = if c.is_bullish() {
            cfg.theme.profit()
        } else {
            cfg.theme.loss()
        };

        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![
                    (c.t - half_w, c.open),
                    (c.t + half_w, c.close),
                    (c.t - half_w, c.close),
                ],
                color.filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;

        if !rsi_vals[i].is_nan() {
            let rsi_color = if rsi_vals[i] > 70.0 {
                cfg.theme.loss()
            } else if rsi_vals[i] < 30.0 {
                cfg.theme.profit()
            } else {
                cfg.theme.info()
            };
            let r = 2.0 + (rsi_vals[i] / 100.0) * 4.0;
            chart
                .draw_series(std::iter::once(Circle::new(
                    (c.t, c.high + pad * 0.3),
                    r as i32,
                    rsi_color.mix(0.6).filled(),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }

        if !macd_line[i].is_nan() {
            let macd_color = if macd_line[i] >= 0.0 {
                cfg.theme.profit()
            } else {
                cfg.theme.loss()
            };
            chart
                .draw_series(std::iter::once(Circle::new(
                    (c.t, c.low - pad * 0.3),
                    3,
                    macd_color.mix(0.6).filled(),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &TriStickBubbleConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &TriStickBubbleConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = TriStickBubbleConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_tri_stick_bubble.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
