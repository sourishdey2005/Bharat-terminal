// crates/bt-viz/src/candlestick_macd.rs
// Author: Sourish Dey

//! Tier 1 #5 â€” Candlestick + MACD Panel (2-panel layout).
//! Made by Sourish Dey.

use bt_analytics::macd;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CandlestickMACDConfig {
    pub title: String,
    pub theme: Theme,
    pub show_volume: bool,
}

impl Default for CandlestickMACDConfig {
    fn default() -> Self {
        Self {
            title: "Candlestick + MACD".to_string(),
            theme: Theme::Dark,
            show_volume: true,
        }
    }
}

impl CandlestickMACDConfig {
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

    pub fn show_volume(mut self, show: bool) -> Self {
        self.show_volume = show;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &CandlestickMACDConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    // Layout: candles (60%) + volume (15%) + MACD (25%)
    let (top, bottom) = if cfg.show_volume {
        let split = root.split_vertically((75).percent());
        (split.0, split.1)
    } else {
        let split = root.split_vertically((60).percent());
        (split.0, split.1)
    };

    let (candle_area, vol_area) = if cfg.show_volume {
        let split = top.split_vertically((80).percent());
        (split.0, Some(split.1))
    } else {
        (top, None)
    };

    let macd_area = bottom;

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

    let (macd_line, signal_line, histogram) = macd(series);

    // Candlestick chart
    let mut chart = ChartBuilder::on(&candle_area)
        .caption(
            format!("{} â€” {}", cfg.title, series.symbol),
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

    // Volume chart
    if let Some(vol_area) = vol_area {
        let max_vol = series
            .candles
            .iter()
            .map(|c| c.volume)
            .fold(0.0_f64, f64::max);
        let mut vol_chart = ChartBuilder::on(&vol_area)
            .margin(10)
            .x_label_area_size(20)
            .y_label_area_size(60)
            .build_cartesian_2d(t_min..t_max, 0.0..(max_vol * 1.1))
            .map_err(|e| BtError::Render(e.to_string()))?;

        vol_chart
            .configure_mesh()
            .label_style((LABEL_FONT, 10).into_font().color(&cfg.theme.text()))
            .axis_style(&cfg.theme.border())
            .disable_x_mesh()
            .draw()
            .map_err(|e| BtError::Render(e.to_string()))?;

        vol_chart
            .draw_series(series.candles.iter().map(|c| {
                let color = if c.is_bullish() {
                    cfg.theme.profit()
                } else {
                    cfg.theme.loss()
                };
                Rectangle::new(
                    [(c.t - candle_width, 0.0), (c.t + candle_width, c.volume)],
                    color.mix(0.6).filled(),
                )
            }))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    // MACD chart
    // Find min/max for scaling
    let macd_min = histogram
        .iter()
        .filter(|v| !v.is_nan())
        .fold(f64::INFINITY, |a, &b| a.min(b));
    let macd_max = histogram
        .iter()
        .filter(|v| !v.is_nan())
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let macd_range = (macd_max - macd_min).max(1e-6);
    let y_min = macd_min - macd_range * 0.2;
    let y_max = macd_max + macd_range * 0.2;

    let mut macd_chart = ChartBuilder::on(&macd_area)
        .caption(
            "MACD (12, 26, 9)",
            (TITLE_FONT, 16).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(50)
        .build_cartesian_2d(t_min..t_max, y_min..y_max)
        .map_err(|e| BtError::Render(e.to_string()))?;

    macd_chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Zero line
    macd_chart
        .draw_series(LineSeries::new(
            vec![(t_min, 0.0), (t_max, 0.0)],
            cfg.theme.border().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Histogram
    macd_chart
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
    macd_chart
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
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("MACD")
        .legend(|(x, y)| {
            PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.info().stroke_width(2))
        });

    // Signal line
    macd_chart
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
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Signal")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                cfg.theme.accent().stroke_width(2),
            )
        });

    macd_chart
        .configure_series_labels()
        .background_style(cfg.theme.background().mix(0.9))
        .border_style(cfg.theme.border())
        .label_font((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &CandlestickMACDConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &CandlestickMACDConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_constructs() {
        let cfg = CandlestickMACDConfig::new();
        let _ = cfg;
    }
}
