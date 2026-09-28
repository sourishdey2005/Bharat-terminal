// crates/bt-viz/src/candlestick_mfi.rs
// Author: Sourish Dey

//! Candlestick + MFI panel. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CandlestickMfiConfig {
    pub title: String,
    pub theme: Theme,
    pub period: usize,
}

impl Default for CandlestickMfiConfig {
    fn default() -> Self {
        Self {
            title: "Candlestick + MFI".to_string(),
            theme: Theme::Dark,
            period: 14,
        }
    }
}

impl CandlestickMfiConfig {
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

fn compute_mfi(series: &OhlcvSeries, period: usize) -> Vec<f64> {
    let n = series.candles.len();
    let mut result = vec![f64::NAN; n];
    if n < period + 1 {
        return result;
    }

    let mut pos_flow = 0.0;
    let mut neg_flow = 0.0;

    for i in 1..=period {
        let tp = (series.candles[i].high + series.candles[i].low + series.candles[i].close) / 3.0;
        let prev_tp = (series.candles[i - 1].high + series.candles[i - 1].low + series.candles[i - 1].close) / 3.0;
        let raw = tp * series.candles[i].volume;
        if tp > prev_tp {
            pos_flow += raw;
        } else if tp < prev_tp {
            neg_flow += raw;
        }
    }

    if neg_flow > 0.0 {
        let mfr = pos_flow / neg_flow;
        result[period] = 100.0 - (100.0 / (1.0 + mfr));
    } else {
        result[period] = 100.0;
    }

    for i in (period + 1)..n {
        let tp = (series.candles[i].high + series.candles[i].low + series.candles[i].close) / 3.0;
        let prev_tp = (series.candles[i - 1].high + series.candles[i - 1].low + series.candles[i - 1].close) / 3.0;
        let raw = tp * series.candles[i].volume;
        let prev_raw = prev_tp * series.candles[i - 1].volume;

        if tp > prev_tp {
            pos_flow += raw - prev_raw;
        } else if tp < prev_tp {
            neg_flow += prev_raw - raw;
        }

        if neg_flow > 0.0 {
            let mfr = pos_flow / neg_flow;
            result[i] = 100.0 - (100.0 / (1.0 + mfr));
        } else {
            result[i] = 100.0;
        }
    }

    result
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &CandlestickMfiConfig,
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
    let low = series.candles.iter().map(|c| c.low).fold(f64::MAX, f64::min);
    let high = series.candles.iter().map(|c| c.high).fold(f64::MIN, f64::max);
    let pad = (high - low) * 0.05;

    let mfi_vals = compute_mfi(series, cfg.period);

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
                c.t, c.open, c.high, c.low, c.close,
                color.filled(), color.filled(), (candle_width * 10.0) as u32,
            )
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    let mut mfi_chart = ChartBuilder::on(&bottom)
        .caption(
            format!("MFI({})", cfg.period),
            (TITLE_FONT, 16).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(50)
        .build_cartesian_2d(t_min..t_max, 0.0..100.0)
        .map_err(|e| BtError::Render(e.to_string()))?;

    mfi_chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    mfi_chart
        .draw_series(LineSeries::new(
            vec![(t_min, 80.0), (t_max, 80.0)],
            cfg.theme.loss().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    mfi_chart
        .draw_series(LineSeries::new(
            vec![(t_min, 20.0), (t_max, 20.0)],
            cfg.theme.profit().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    mfi_chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !mfi_vals[i].is_nan() { Some((c.t, mfi_vals[i])) } else { None }
            }),
            cfg.theme.info().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &CandlestickMfiConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &CandlestickMfiConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = CandlestickMfiConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_candlestick_mfi.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
