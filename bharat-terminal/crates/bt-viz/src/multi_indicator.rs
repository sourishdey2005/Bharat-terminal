// crates/bt-viz/src/multi_indicator.rs
// Author: Sourish Dey

//! Tier 3 — Multi-Indicator Dashboard (2x3 grid: RSI, MACD, Stochastic, ATR, OBV, VWAP).
//! Made by Sourish Dey.

use bt_analytics::{atr, macd, obv, rsi, stochastic, vwap};
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct MultiIndicatorConfig {
    pub title: String,
    pub theme: Theme,
    pub rsi_period: usize,
    pub stoch_k_period: usize,
    pub stoch_d_period: usize,
    pub atr_period: usize,
}

impl Default for MultiIndicatorConfig {
    fn default() -> Self {
        Self {
            title: "Multi-Indicator Dashboard".to_string(),
            theme: Theme::Dark,
            rsi_period: 14,
            stoch_k_period: 14,
            stoch_d_period: 3,
            atr_period: 14,
        }
    }
}

impl MultiIndicatorConfig {
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
}

fn draw_indicator_panel<DB: DrawingBackend>(
    area: &DrawingArea<DB, plotters::coord::Shift>,
    title: &str,
    series: &OhlcvSeries,
    values: &[f64],
    theme: Theme,
    y_min: f64,
    y_max: f64,
    hlines: &[(f64, RGBColor)],
    color: RGBColor,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;

    let mut chart = ChartBuilder::on(area)
        .caption(title, (TITLE_FONT, 14).into_font().color(&theme.text()))
        .margin(5)
        .x_label_area_size(25)
        .y_label_area_size(45)
        .build_cartesian_2d(t_min..t_max, y_min..y_max)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 10).into_font().color(&theme.text()))
        .axis_style(&theme.border())
        .light_line_style(theme.border().mix(0.2))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    for (y, c) in hlines {
        chart
            .draw_series(LineSeries::new(
                vec![(t_min, *y), (t_max, *y)],
                c.mix(0.4).stroke_width(1),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !values[i].is_nan() {
                    Some((c.t, values[i]))
                } else {
                    None
                }
            }),
            color.stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    Ok(())
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &MultiIndicatorConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    // Compute all indicators
    let rsi_vals = rsi(series, cfg.rsi_period);
    let (macd_line, signal_line, histogram) = macd(series);
    let (stoch_k, stoch_d) = stochastic(series, cfg.stoch_k_period, cfg.stoch_d_period);
    let atr_vals = atr(series, cfg.atr_period);
    let obv_vals = obv(series);
    let vwap_vals = vwap(series);

    // 2x3 grid layout
    let (top_row, bottom_row) = root.split_vertically((50).percent());
    let (rsi_area, macd_area, stoch_area) = {
        let (left, right) = top_row.split_horizontally((33).percent());
        let (mid, right) = right.split_horizontally((50).percent());
        (left, mid, right)
    };
    let (atr_area, obv_area, vwap_area) = {
        let (left, right) = bottom_row.split_horizontally((33).percent());
        let (mid, right) = right.split_horizontally((50).percent());
        (left, mid, right)
    };

    // RSI Panel
    draw_indicator_panel(
        &rsi_area,
        &format!("RSI({})", cfg.rsi_period),
        series,
        &rsi_vals,
        cfg.theme,
        0.0,
        100.0,
        &[
            (70.0, cfg.theme.loss()),
            (30.0, cfg.theme.profit()),
            (50.0, cfg.theme.border()),
        ],
        cfg.theme.info(),
    )?;

    // MACD Panel
    {
        let t_min = series.candles.first().unwrap().t;
        let t_max = series.candles.last().unwrap().t;
        let macd_min = histogram
            .iter()
            .filter(|v| !v.is_nan())
            .fold(f64::INFINITY, |a, &b| a.min(b));
        let macd_max = histogram
            .iter()
            .filter(|v| !v.is_nan())
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let range = (macd_max - macd_min).max(1e-6);
        let y_min = macd_min - range * 0.2;
        let y_max = macd_max + range * 0.2;

        let mut chart = ChartBuilder::on(&macd_area)
            .caption(
                "MACD",
                (TITLE_FONT, 14).into_font().color(&cfg.theme.text()),
            )
            .margin(5)
            .x_label_area_size(25)
            .y_label_area_size(45)
            .build_cartesian_2d(t_min..t_max, y_min..y_max)
            .map_err(|e| BtError::Render(e.to_string()))?;

        chart
            .configure_mesh()
            .label_style((LABEL_FONT, 10).into_font().color(&cfg.theme.text()))
            .axis_style(&cfg.theme.border())
            .light_line_style(cfg.theme.border().mix(0.2))
            .draw()
            .map_err(|e| BtError::Render(e.to_string()))?;

        // Zero line
        chart
            .draw_series(LineSeries::new(
                vec![(t_min, 0.0), (t_max, 0.0)],
                cfg.theme.border().mix(0.4).stroke_width(1),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;

        // Histogram
        let candle_width = ((t_max - t_min) / series.candles.len() as f64).max(0.3) * 0.4;
        chart
            .draw_series(series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !histogram[i].is_nan() {
                    let color = if histogram[i] >= 0.0 {
                        cfg.theme.profit()
                    } else {
                        cfg.theme.loss()
                    };
                    Some(Rectangle::new(
                        [
                            (c.t - candle_width * 0.5, 0.0),
                            (c.t + candle_width * 0.5, histogram[i]),
                        ],
                        color.mix(0.7).filled(),
                    ))
                } else {
                    None
                }
            }))
            .map_err(|e| BtError::Render(e.to_string()))?;

        // MACD line
        chart
            .draw_series(LineSeries::new(
                series.candles.iter().enumerate().filter_map(|(i, c)| {
                    if !macd_line[i].is_nan() {
                        Some((c.t, macd_line[i]))
                    } else {
                        None
                    }
                }),
                cfg.theme.info().stroke_width(2),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;

        // Signal line
        chart
            .draw_series(LineSeries::new(
                series.candles.iter().enumerate().filter_map(|(i, c)| {
                    if !signal_line[i].is_nan() {
                        Some((c.t, signal_line[i]))
                    } else {
                        None
                    }
                }),
                cfg.theme.accent().stroke_width(2),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    // Stochastic Panel
    draw_indicator_panel(
        &stoch_area,
        &format!("Stoch({}, {})", cfg.stoch_k_period, cfg.stoch_d_period),
        series,
        &stoch_k,
        cfg.theme,
        0.0,
        100.0,
        &[
            (80.0, cfg.theme.loss()),
            (20.0, cfg.theme.profit()),
            (50.0, cfg.theme.border()),
        ],
        cfg.theme.info(),
    )?;

    // ATR Panel
    let atr_max = atr_vals
        .iter()
        .filter(|v| !v.is_nan())
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    draw_indicator_panel(
        &atr_area,
        &format!("ATR({})", cfg.atr_period),
        series,
        &atr_vals,
        cfg.theme,
        0.0,
        atr_max * 1.2,
        &[],
        cfg.theme.profit(),
    )?;

    // OBV Panel
    let obv_min = obv_vals.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let obv_max = obv_vals.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    draw_indicator_panel(
        &obv_area,
        "OBV",
        series,
        &obv_vals,
        cfg.theme,
        obv_min - (obv_max - obv_min).abs() * 0.1,
        obv_max + (obv_max - obv_min).abs() * 0.1,
        &[],
        cfg.theme.accent(),
    )?;

    // VWAP Panel
    let vwap_min = vwap_vals
        .iter()
        .filter(|v| !v.is_nan())
        .fold(f64::INFINITY, |a, &b| a.min(b));
    let vwap_max = vwap_vals
        .iter()
        .filter(|v| !v.is_nan())
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    draw_indicator_panel(
        &vwap_area,
        "VWAP",
        series,
        &vwap_vals,
        cfg.theme,
        vwap_min - (vwap_max - vwap_min).abs() * 0.1,
        vwap_max + (vwap_max - vwap_min).abs() * 0.1,
        &[],
        cfg.theme.info(),
    )?;

    // Main title
    root.draw(&Text::new(
        format!("{} — {}", cfg.title, series.symbol),
        (root.dim_in_pixel().0 as i32 / 2, 25),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &MultiIndicatorConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &MultiIndicatorConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_constructs() {
        let cfg = MultiIndicatorConfig::new();
        let _ = cfg;
    }
}
