// crates/bt-viz/src/candlestick_dpo.rs
// Author: Sourish Dey

//! Candlestick + Detrended Price Oscillator. Made by Sourish Dey.

use bt_analytics::sma;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CandlestickDpoConfig {
    pub title: String,
    pub theme: Theme,
    pub period: usize,
}

impl Default for CandlestickDpoConfig {
    fn default() -> Self {
        Self {
            title: "Candlestick + DPO".to_string(),
            theme: Theme::Dark,
            period: 20,
        }
    }
}

impl CandlestickDpoConfig {
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

fn dpo(series: &OhlcvSeries, period: usize) -> Vec<f64> {
    let sma_vals = sma(series, period);
    let n = series.candles.len();
    let mut result = vec![f64::NAN; n];
    let shift = period / 2 + 1;

    for i in shift..n {
        if !sma_vals[i - shift].is_nan() {
            result[i] = series.candles[i].close - sma_vals[i - shift];
        }
    }

    result
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &CandlestickDpoConfig,
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

    let dpo_vals = dpo(series, cfg.period);

    let dpo_min = dpo_vals.iter().filter(|v| !v.is_nan()).copied().fold(f64::MAX, f64::min);
    let dpo_max = dpo_vals.iter().filter(|v| !v.is_nan()).copied().fold(f64::MIN, f64::max);
    let dpo_range = (dpo_max - dpo_min).max(0.001);

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

    let mut dpo_chart = ChartBuilder::on(&bottom)
        .caption(
            format!("DPO({})", cfg.period),
            (TITLE_FONT, 16).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(50)
        .build_cartesian_2d(t_min..t_max, (dpo_min - dpo_range * 0.1)..(dpo_max + dpo_range * 0.1))
        .map_err(|e| BtError::Render(e.to_string()))?;

    dpo_chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    dpo_chart
        .draw_series(LineSeries::new(
            vec![(t_min, 0.0), (t_max, 0.0)],
            cfg.theme.border().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    dpo_chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !dpo_vals[i].is_nan() { Some((c.t, dpo_vals[i])) } else { None }
            }),
            cfg.theme.info().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &CandlestickDpoConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &CandlestickDpoConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = CandlestickDpoConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_candlestick_dpo.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
