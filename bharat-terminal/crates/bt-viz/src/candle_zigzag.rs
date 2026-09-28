// crates/bt-viz/src/candle_zigzag.rs
// Author: Sourish Dey

//! Candles + Zigzag indicator. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CandleZigzagConfig {
    pub title: String,
    pub theme: Theme,
    pub deviation_pct: f64,
}

impl Default for CandleZigzagConfig {
    fn default() -> Self {
        Self {
            title: "Candles + Zigzag".to_string(),
            theme: Theme::Dark,
            deviation_pct: 5.0,
        }
    }
}

impl CandleZigzagConfig {
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

    pub fn deviation_pct(mut self, p: f64) -> Self {
        self.deviation_pct = p.max(0.1);
        self
    }
}

fn compute_zigzag(series: &OhlcvSeries, deviation_pct: f64) -> Vec<(f64, f64)> {
    if series.candles.is_empty() {
        return Vec::new();
    }

    let mut pivots = Vec::new();
    let mut trend_up: Option<bool> = None;
    let mut last_pivot_idx = 0usize;
    let mut last_pivot_price = series.candles[0].close;

    pivots.push((series.candles[0].t, series.candles[0].close));

    let dev = deviation_pct / 100.0;

    for i in 1..series.candles.len() {
        let c = &series.candles[i];
        let extreme = if trend_up.unwrap_or(true) {
            c.high
        } else {
            c.low
        };

        match trend_up {
            None => {
                if (extreme - last_pivot_price).abs() / last_pivot_price >= dev {
                    trend_up = Some(extreme > last_pivot_price);
                    last_pivot_price = extreme;
                    last_pivot_idx = i;
                    pivots.push((c.t, extreme));
                }
            }
            Some(true) => {
                if c.high > last_pivot_price {
                    last_pivot_price = c.high;
                    last_pivot_idx = i;
                    if let Some(last) = pivots.last_mut() {
                        *last = (c.t, c.high);
                    }
                } else if (last_pivot_price - c.low) / last_pivot_price >= dev {
                    trend_up = Some(false);
                    last_pivot_price = c.low;
                    last_pivot_idx = i;
                    pivots.push((c.t, c.low));
                }
            }
            Some(false) => {
                if c.low < last_pivot_price {
                    last_pivot_price = c.low;
                    last_pivot_idx = i;
                    if let Some(last) = pivots.last_mut() {
                        *last = (c.t, c.low);
                    }
                } else if (c.high - last_pivot_price) / last_pivot_price >= dev {
                    trend_up = Some(true);
                    last_pivot_price = c.high;
                    last_pivot_idx = i;
                    pivots.push((c.t, c.high));
                }
            }
        }
    }

    let _ = last_pivot_idx;
    pivots
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &CandleZigzagConfig,
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

    let zigzag = compute_zigzag(series, cfg.deviation_pct);

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!(
                "{} — {} (Zigzag: {:.1}%)",
                cfg.title, series.symbol, cfg.deviation_pct
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

    let candle_width = ((t_max - t_min) / series.candles.len() as f64).max(0.3) * 0.4;

    chart
        .draw_series(series.candles.iter().map(|c| {
            let color = if c.is_bullish() {
                cfg.theme.profit()
            } else {
                cfg.theme.loss()
            };
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

    if zigzag.len() >= 2 {
        chart
            .draw_series(LineSeries::new(zigzag, cfg.theme.info().stroke_width(2)))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &CandleZigzagConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &CandleZigzagConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = CandleZigzagConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_candle_zigzag.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
