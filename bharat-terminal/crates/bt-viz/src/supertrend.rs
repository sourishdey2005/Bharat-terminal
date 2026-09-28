// crates/bt-viz/src/supertrend.rs
// Author: Sourish Dey

//! Supertrend indicator chart. Made by Sourish Dey.

use bt_analytics::atr;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct SupertrendConfig {
    pub title: String,
    pub theme: Theme,
    pub period: usize,
    pub multiplier: f64,
}

impl Default for SupertrendConfig {
    fn default() -> Self {
        Self { title: "Supertrend".to_string(), theme: Theme::Dark, period: 10, multiplier: 3.0 }
    }
}

impl SupertrendConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn period(mut self, p: usize) -> Self { self.period = p.max(2); self }
    pub fn multiplier(mut self, m: f64) -> Self { self.multiplier = m.max(0.5); self }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &SupertrendConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let atr_vals = atr(series, cfg.period);
    let n = series.candles.len();
    let mut supertrend = vec![f64::NAN; n];
    let mut is_uptrend = true;

    for i in 0..n {
        if atr_vals[i].is_nan() { continue; }
        let hl2 = (series.candles[i].high + series.candles[i].low) / 2.0;
        let upper = hl2 + cfg.multiplier * atr_vals[i];
        let lower = hl2 - cfg.multiplier * atr_vals[i];

        if i == 0 || supertrend[i - 1].is_nan() {
            supertrend[i] = lower;
            is_uptrend = true;
        } else if series.candles[i].close > supertrend[i - 1] {
            supertrend[i] = lower.max(supertrend[i - 1]);
            is_uptrend = true;
        } else {
            supertrend[i] = upper.min(supertrend[i - 1]);
            is_uptrend = false;
        }
    }

    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;
    let low = series.candles.iter().map(|c| c.low).fold(f64::MAX, f64::min);
    let high = series.candles.iter().map(|c| c.high).fold(f64::MIN, f64::max);
    let pad = (high - low) * 0.05;

    let mut chart = ChartBuilder::on(&root)
        .caption(&cfg.title, (TITLE_FONT, 22).into_font().color(&cfg.theme.text()))
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, (low - pad)..(high + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart.configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    for c in &series.candles {
        let color = if c.is_bullish() { cfg.theme.profit() } else { cfg.theme.loss() };
        chart.draw_series(std::iter::once(CandleStick::new(
            c.t, c.open, c.high, c.low, c.close,
            color.filled(), color.filled(), 5,
        )))
    .map_err(|e| BtError::Render(e.to_string()))?;
    }

    chart.draw_series(LineSeries::new(
        series.candles.iter().enumerate().filter_map(|(i, c)| if !supertrend[i].is_nan() { Some((c.t, supertrend[i])) } else { None }),
        cfg.theme.info().stroke_width(2),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &SupertrendConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &SupertrendConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = SupertrendConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_supertrend.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}