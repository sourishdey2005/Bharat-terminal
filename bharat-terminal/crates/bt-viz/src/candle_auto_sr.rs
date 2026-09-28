// crates/bt-viz/src/candle_auto_sr.rs
// Author: Sourish Dey

//! Candles + auto-detected support/resistance. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CandleAutoSrConfig {
    pub title: String,
    pub theme: Theme,
    pub lookback: usize,
    pub tolerance_pct: f64,
}

impl Default for CandleAutoSrConfig {
    fn default() -> Self {
        Self {
            title: "Auto S/R".to_string(),
            theme: Theme::Dark,
            lookback: 20,
            tolerance_pct: 1.0,
        }
    }
}

impl CandleAutoSrConfig {
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

    pub fn lookback(mut self, n: usize) -> Self {
        self.lookback = n.max(5);
        self
    }

    pub fn tolerance_pct(mut self, p: f64) -> Self {
        self.tolerance_pct = p.max(0.1);
        self
    }
}

fn detect_sr(series: &OhlcvSeries, lookback: usize, tolerance_pct: f64) -> (Vec<f64>, Vec<f64>) {
    let n = series.candles.len();
    if n < lookback {
        return (Vec::new(), Vec::new());
    }

    let tol = tolerance_pct / 100.0;
    let mut supports = Vec::new();
    let mut resistances = Vec::new();

    for i in lookback..n {
        let window = &series.candles[i - lookback..i];
        let cur = &series.candles[i];

        let is_low = window
            .iter()
            .all(|c| cur.low <= c.low * (1.0 + tol));
        let is_high = window
            .iter()
            .all(|c| cur.high >= c.high * (1.0 - tol));

        if is_low {
            supports.push(cur.low);
        }
        if is_high {
            resistances.push(cur.high);
        }
    }

    (supports, resistances)
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &CandleAutoSrConfig,
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

    let (supports, resistances) = detect_sr(series, cfg.lookback, cfg.tolerance_pct);

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!(
                "{} — {} (Lookback: {})",
                cfg.title, series.symbol, cfg.lookback
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

    for s in &supports {
        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![(t_min, *s), (t_max, *s)],
                cfg.theme.profit().stroke_width(1),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    for r in &resistances {
        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![(t_min, *r), (t_max, *r)],
                cfg.theme.loss().stroke_width(1),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &CandleAutoSrConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &CandleAutoSrConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = CandleAutoSrConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_candle_auto_sr.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
