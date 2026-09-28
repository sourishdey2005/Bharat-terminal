// crates/bt-viz/src/candle_vol_heatmap.rs
// Author: Sourish Dey

//! Candle bodies colored by ATR/true range. Made by Sourish Dey.

use bt_analytics::atr;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CandleVolHeatmapConfig {
    pub title: String,
    pub theme: Theme,
    pub atr_period: usize,
}

impl Default for CandleVolHeatmapConfig {
    fn default() -> Self {
        Self {
            title: "ATR Heatmap Candles".to_string(),
            theme: Theme::Dark,
            atr_period: 14,
        }
    }
}

impl CandleVolHeatmapConfig {
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

    pub fn atr_period(mut self, p: usize) -> Self {
        self.atr_period = p.max(1);
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &CandleVolHeatmapConfig,
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

    let atr_vals = atr(series, cfg.atr_period);
    let atr_max = atr_vals
        .iter()
        .filter(|v| !v.is_nan())
        .fold(0.0_f64, |a, &b| a.max(b));

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

    let candle_width = ((t_max - t_min) / series.candles.len() as f64).max(0.3) * 0.4;

    chart
        .draw_series(series.candles.iter().enumerate().map(|(i, c)| {
            let base = if c.is_bullish() {
                cfg.theme.profit()
            } else {
                cfg.theme.loss()
            };
            let intensity = if atr_max > 0.0 && !atr_vals[i].is_nan() {
                (atr_vals[i] / atr_max).min(1.0)
            } else {
                0.3
            };
            let inv = 1.0 - intensity;
            let color = RGBColor(
                (base.0 as f64 * intensity + 30.0 * inv) as u8,
                (base.1 as f64 * intensity + 30.0 * inv) as u8,
                (base.2 as f64 * intensity + 40.0 * inv) as u8,
            );
            CandleStick::new(
                c.t,
                c.open,
                c.high,
                c.low,
                c.close,
                color.filled(),
                color.filled(),
                (candle_width * 10.0) as u32,
            )
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &CandleVolHeatmapConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &CandleVolHeatmapConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = CandleVolHeatmapConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_candle_vol_heatmap.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
