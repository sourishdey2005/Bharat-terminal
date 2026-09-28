// crates/bt-viz/src/volume_weighted_scatter.rs
// Author: Sourish Dey

//! Volume-weighted price scatter. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct VolumeWeightedScatterConfig {
    pub title: String,
    pub theme: Theme,
    pub show_trend: bool,
}

impl Default for VolumeWeightedScatterConfig {
    fn default() -> Self {
        Self {
            title: "Volume-Weighted Scatter".to_string(),
            theme: Theme::Dark,
            show_trend: true,
        }
    }
}

impl VolumeWeightedScatterConfig {
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

    pub fn show_trend(mut self, show: bool) -> Self {
        self.show_trend = show;
        self
    }
}

fn compute_trend_line(series: &OhlcvSeries) -> (f64, f64) {
    let n = series.candles.len() as f64;
    let sum_x: f64 = series.candles.iter().map(|c| c.t).sum();
    let sum_y: f64 = series.candles.iter().map(|c| c.close).sum();
    let sum_xy: f64 = series.candles.iter().map(|c| c.t * c.close).sum();
    let sum_x2: f64 = series.candles.iter().map(|c| c.t * c.t).sum();

    let denom = n * sum_x2 - sum_x * sum_x;
    if denom.abs() < 1e-10 {
        return (0.0, sum_y / n);
    }

    let slope = (n * sum_xy - sum_x * sum_y) / denom;
    let intercept = (sum_y - slope * sum_x) / n;

    (slope, intercept)
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &VolumeWeightedScatterConfig,
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
    let pad = (high - low).max(1.0) * 0.05;

    let max_vol = series
        .candles
        .iter()
        .map(|c| c.volume)
        .fold(0.0_f64, f64::max);

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

    if cfg.show_trend {
        let (slope, intercept) = compute_trend_line(series);
        chart
            .draw_series(LineSeries::new(
                vec![
                    (t_min, slope * t_min + intercept),
                    (t_max, slope * t_max + intercept),
                ],
                cfg.theme.accent().mix(0.5).stroke_width(1),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    for candle in &series.candles {
        let size = (candle.volume / max_vol * 10.0).max(1.0) as i32;
        let color = if candle.is_bullish() {
            cfg.theme.profit().mix(0.6)
        } else {
            cfg.theme.loss().mix(0.6)
        };

        chart
            .draw_series(std::iter::once(Circle::new(
                (candle.t, candle.close),
                size,
                color.filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(
    series: &OhlcvSeries,
    cfg: &VolumeWeightedScatterConfig,
    path: &str,
) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(
    series: &OhlcvSeries,
    cfg: &VolumeWeightedScatterConfig,
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
        let cfg = VolumeWeightedScatterConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_volume_weighted_scatter.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
