// crates/bt-viz/src/candlestick_trix.rs
// Author: Sourish Dey

//! Candlestick + TRIX indicator. Made by Sourish Dey.

use bt_analytics::ema;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CandlestickTrixConfig {
    pub title: String,
    pub theme: Theme,
    pub period: usize,
}

impl Default for CandlestickTrixConfig {
    fn default() -> Self {
        Self {
            title: "Candlestick + TRIX".to_string(),
            theme: Theme::Dark,
            period: 14,
        }
    }
}

impl CandlestickTrixConfig {
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
}

fn trix(series: &OhlcvSeries, period: usize) -> Vec<f64> {
    let ema1 = ema(series, period);
    let n = ema1.len();
    let mut ema2 = vec![f64::NAN; n];
    let mut ema3 = vec![f64::NAN; n];
    let mut result = vec![f64::NAN; n];

    let alpha = 2.0 / (period as f64 + 1.0);
    let mut started = false;
    for i in 0..n {
        if !ema1[i].is_nan() {
            if !started {
                ema2[i] = ema1[i];
                started = true;
            } else {
                ema2[i] = alpha * ema1[i] + (1.0 - alpha) * ema2[i - 1];
            }
        }
    }

    started = false;
    for i in 0..n {
        if !ema2[i].is_nan() {
            if !started {
                ema3[i] = ema2[i];
                started = true;
            } else {
                ema3[i] = alpha * ema2[i] + (1.0 - alpha) * ema3[i - 1];
            }
        }
    }

    for i in 1..n {
        if !ema3[i].is_nan() && !ema3[i - 1].is_nan() && ema3[i - 1] != 0.0 {
            result[i] = 100.0 * (ema3[i] - ema3[i - 1]) / ema3[i - 1];
        }
    }

    result
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &CandlestickTrixConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let split = root.split_vertically((70).percent());
    let (top, bottom) = (split.0, split.1);

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

    let trix_vals = trix(series, cfg.period);

    let trix_min = trix_vals
        .iter()
        .filter(|v| !v.is_nan())
        .copied()
        .fold(f64::MAX, f64::min);
    let trix_max = trix_vals
        .iter()
        .filter(|v| !v.is_nan())
        .copied()
        .fold(f64::MIN, f64::max);
    let trix_range = (trix_max - trix_min).max(0.001);

    let mut chart = ChartBuilder::on(&top)
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

    let mut trix_chart = ChartBuilder::on(&bottom)
        .caption(
            format!("TRIX({})", cfg.period),
            (TITLE_FONT, 16).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(50)
        .build_cartesian_2d(
            t_min..t_max,
            (trix_min - trix_range * 0.1)..(trix_max + trix_range * 0.1),
        )
        .map_err(|e| BtError::Render(e.to_string()))?;

    trix_chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    trix_chart
        .draw_series(LineSeries::new(
            vec![(t_min, 0.0), (t_max, 0.0)],
            cfg.theme.border().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    trix_chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !trix_vals[i].is_nan() {
                    Some((c.t, trix_vals[i]))
                } else {
                    None
                }
            }),
            cfg.theme.info().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &CandlestickTrixConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &CandlestickTrixConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = CandlestickTrixConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_candlestick_trix.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
