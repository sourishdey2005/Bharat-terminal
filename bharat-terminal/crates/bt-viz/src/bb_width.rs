// crates/bt-viz/src/bb_width.rs
// Author: Sourish Dey

//! Tier 3 â€” Bollinger Band Width (band squeeze/expansion).
//! Made by Sourish Dey.

use bt_analytics::bollinger;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct BBWidthConfig {
    pub title: String,
    pub theme: Theme,
    pub period: usize,
    pub std_dev: f64,
}

impl Default for BBWidthConfig {
    fn default() -> Self {
        Self {
            title: "Bollinger Band Width".to_string(),
            theme: Theme::Dark,
            period: 20,
            std_dev: 2.0,
        }
    }
}

impl BBWidthConfig {
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

    pub fn std_dev(mut self, k: f64) -> Self {
        self.std_dev = k.max(0.5);
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &BBWidthConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;

    let (middle, upper, lower) = bollinger(series, cfg.period, cfg.std_dev);

    // Calculate band width: (Upper - Lower) / Middle * 100
    let mut bb_width = vec![f64::NAN; series.candles.len()];
    for i in 0..series.candles.len() {
        if !middle[i].is_nan() && !upper[i].is_nan() && !lower[i].is_nan() && middle[i] != 0.0 {
            bb_width[i] = (upper[i] - lower[i]) / middle[i] * 100.0;
        }
    }

    // Layout: Price with bands (top 60%), BB Width (bottom 40%)
    let (price_area, width_area) = root.split_vertically((60).percent());

    // Price chart with Bollinger Bands
    let low = series.candles.iter().map(|c| c.low).fold(f64::MAX, f64::min);
    let high = series.candles.iter().map(|c| c.high).fold(f64::MIN, f64::max);
    let band_high = upper.iter().filter(|v| !v.is_nan()).fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let band_low = lower.iter().filter(|v| !v.is_nan()).fold(f64::INFINITY, |a, &b| a.min(b));
    let y_min = low.min(band_low) - (high - low) * 0.05;
    let y_max = high.max(band_high) + (high - low) * 0.05;

    let mut price_chart = ChartBuilder::on(&price_area)
        .caption(
            format!("{} â€” {}", cfg.title, series.symbol),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, y_min..y_max)
        .map_err(|e| BtError::Render(e.to_string()))?;

    price_chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Shaded bands
    price_chart
        .draw_series(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !upper[i].is_nan() && !lower[i].is_nan() {
                    Some(Polygon::new(
                        vec![(c.t, upper[i]), (c.t, lower[i])],
                        cfg.theme.info().mix(0.15).filled(),
                    ))
                } else { None }
            }),
        )
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Band lines
    price_chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !upper[i].is_nan() { Some((c.t, upper[i])) } else { None }
            }),
            cfg.theme.info().mix(0.7).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    price_chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !middle[i].is_nan() { Some((c.t, middle[i])) } else { None }
            }),
            cfg.theme.accent().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    price_chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !lower[i].is_nan() { Some((c.t, lower[i])) } else { None }
            }),
            cfg.theme.info().mix(0.7).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    let candle_width = ((t_max - t_min) / series.candles.len() as f64).max(0.3) * 0.4;
    price_chart
        .draw_series(series.candles.iter().map(|c| {
            let color = if c.is_bullish() { cfg.theme.profit() } else { cfg.theme.loss() };
            CandleStick::new(c.t, c.open, c.high, c.low, c.close, color.filled(), color.filled(), (candle_width * 10.0) as u32)
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    // BB Width chart
    let width_vals: Vec<f64> = bb_width.iter().filter(|v| !v.is_nan()).cloned().collect();
    let width_min = width_vals.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let width_max = width_vals.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let range = (width_max - width_min).max(1e-6);

    let mut width_chart = ChartBuilder::on(&width_area)
        .caption(
            format!("BB Width ({}, {})", cfg.period, cfg.std_dev),
            (TITLE_FONT, 16).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, (width_min - range * 0.1)..(width_max + range * 0.1))
        .map_err(|e| BtError::Render(e.to_string()))?;

    width_chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    width_chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !bb_width[i].is_nan() { Some((c.t, bb_width[i])) } else { None }
            }),
            cfg.theme.accent().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Fill area under curve
    width_chart
        .draw_series(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !bb_width[i].is_nan() && i > 0 && !bb_width[i-1].is_nan() {
                    Some(Polygon::new(
                        vec![
                            (c.t, 0.0),
                            (c.t, bb_width[i]),
                            (series.candles[i-1].t, bb_width[i-1]),
                            (series.candles[i-1].t, 0.0),
                        ],
                        cfg.theme.accent().mix(0.1).filled(),
                    ))
                } else { None }
            }),
        )
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &BBWidthConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &BBWidthConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_constructs() {
        let cfg = BBWidthConfig::new();
        let _ = cfg;
    }
}
