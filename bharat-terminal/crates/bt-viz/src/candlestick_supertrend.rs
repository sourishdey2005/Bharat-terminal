// crates/bt-viz/src/candlestick_supertrend.rs
// Author: Sourish Dey

//! Candlestick + Supertrend. Made by Sourish Dey.

use bt_analytics::atr;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CandlestickSupertrendConfig {
    pub title: String,
    pub theme: Theme,
    pub period: usize,
    pub multiplier: f64,
}

impl Default for CandlestickSupertrendConfig {
    fn default() -> Self {
        Self {
            title: "Candlestick + Supertrend".to_string(),
            theme: Theme::Dark,
            period: 10,
            multiplier: 3.0,
        }
    }
}

impl CandlestickSupertrendConfig {
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
    pub fn period(mut self, period: usize) -> Self {
        self.period = period.max(2);
        self
    }
    pub fn multiplier(mut self, v: f64) -> Self {
        self.multiplier = v.max(0.1);
        self
    }
}

fn supertrend(series: &OhlcvSeries, period: usize, multiplier: f64) -> (Vec<f64>, Vec<bool>) {
    let atr_vals = atr(series, period);
    let n = series.candles.len();
    let mut result = vec![f64::NAN; n];
    let mut is_uptrend = vec![true; n];

    for i in period..n {
        if atr_vals[i].is_nan() {
            continue;
        }
        let hl2 = (series.candles[i].high + series.candles[i].low) / 2.0;
        let upper = hl2 + multiplier * atr_vals[i];
        let lower = hl2 - multiplier * atr_vals[i];

        if i == period {
            result[i] = lower;
            is_uptrend[i] = true;
        } else {
            let prev = result[i - 1];
            if series.candles[i - 1].close > prev {
                result[i] = lower.max(prev);
                is_uptrend[i] = true;
            } else {
                result[i] = upper.min(prev);
                is_uptrend[i] = false;
            }
        }
    }

    (result, is_uptrend)
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &CandlestickSupertrendConfig,
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

    let (st_vals, is_uptrend) = supertrend(series, cfg.period, cfg.multiplier);

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

    for i in 0..series.candles.len().saturating_sub(1) {
        if !st_vals[i].is_nan() && !st_vals[i + 1].is_nan() {
            let c1 = &series.candles[i];
            let c2 = &series.candles[i + 1];
            let color = if is_uptrend[i] {
                cfg.theme.profit()
            } else {
                cfg.theme.loss()
            };
            chart
                .draw_series(LineSeries::new(
                    vec![(c1.t, st_vals[i]), (c2.t, st_vals[i + 1])],
                    color.stroke_width(2),
                ))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(
    series: &OhlcvSeries,
    cfg: &CandlestickSupertrendConfig,
    path: &str,
) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(
    series: &OhlcvSeries,
    cfg: &CandlestickSupertrendConfig,
    path: &str,
) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = CandlestickSupertrendConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_candlestick_supertrend.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
