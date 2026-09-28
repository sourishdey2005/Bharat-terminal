// crates/bt-viz/src/pattern_recognition.rs
// Author: Sourish Dey

//! Candlestick pattern recognition.
//! Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub index: usize,
    pub name: String,
    pub signal: String,
}

impl PatternMatch {
    pub fn new(index: usize, name: impl Into<String>, signal: impl Into<String>) -> Self {
        Self { index, name: name.into(), signal: signal.into() }
    }
}

#[derive(Debug, Clone)]
pub struct PatternRecognitionConfig {
    pub title: String,
    pub theme: Theme,
    pub patterns: Vec<String>,
}

impl Default for PatternRecognitionConfig {
    fn default() -> Self {
        Self {
            title: "Candlestick Pattern Recognition".to_string(),
            theme: Theme::Dark,
            patterns: vec!["Doji".into(), "Hammer".into(), "Engulfing".into()],
        }
    }
}

impl PatternRecognitionConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn patterns(mut self, p: Vec<String>) -> Self { self.patterns = p; self }
}

fn detect_patterns(series: &OhlcvSeries) -> Vec<PatternMatch> {
    let mut matches = Vec::new();
    let candles = &series.candles;

    for i in 1..candles.len() {
        let c = candles[i];
        let prev = candles[i - 1];

        let body = (c.close - c.open).abs();
        let range = c.high - c.low;
        let upper_wick = c.high - c.open.max(c.close);
        let lower_wick = c.open.min(c.close) - c.low;

        if range > 0.0 && body / range < 0.1 {
            matches.push(PatternMatch::new(i, "Doji", "Neutral"));
        }

        if lower_wick > body * 2.0 && upper_wick < body * 0.5 {
            matches.push(PatternMatch::new(i, "Hammer", "Bullish"));
        }

        if prev.is_bullish() && !c.is_bullish() && c.close < prev.open && c.open > prev.close {
            matches.push(PatternMatch::new(i, "Bearish Engulfing", "Bearish"));
        }

        if !prev.is_bullish() && c.is_bullish() && c.close > prev.high && c.open < prev.low {
            matches.push(PatternMatch::new(i, "Bullish Engulfing", "Bullish"));
        }
    }

    matches
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &PatternRecognitionConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let patterns = detect_patterns(series);

    let t_min = series.candles.first().map(|c| c.t).unwrap_or(0.0);
    let t_max = series.candles.last().map(|c| c.t).unwrap_or(1.0);
    let low = series.candles.iter().map(|c| c.low).fold(f64::MAX, f64::min);
    let high = series.candles.iter().map(|c| c.high).fold(f64::MIN, f64::max);
    let pad = (high - low).max(1.0) * 0.05;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — {} patterns found", cfg.title, patterns.len()),
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
            let color = if c.is_bullish() { cfg.theme.profit() } else { cfg.theme.loss() };
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

    for pattern in &patterns {
        if pattern.index < series.candles.len() {
            let c = &series.candles[pattern.index];
            let color = if pattern.signal == "Bullish" {
                cfg.theme.profit()
            } else if pattern.signal == "Bearish" {
                cfg.theme.loss()
            } else {
                cfg.theme.accent()
            };

            chart
                .draw_series(std::iter::once(Text::new(
                    pattern.name.clone(),
                    (c.t - 0.5, c.high + pad * 0.3),
                    (LABEL_FONT, 10).into_font().color(&color),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &PatternRecognitionConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &PatternRecognitionConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = PatternRecognitionConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_pattern_recognition.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
