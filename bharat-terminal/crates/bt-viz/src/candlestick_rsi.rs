// crates/bt-viz/src/candlestick_rsi.rs
// Author: Sourish Dey

//! Tier 1 #4 â€” Candlestick + RSI Panel (2-panel layout).
//! Made by Sourish Dey.

use bt_analytics::rsi;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CandlestickRSIConfig {
    pub title: String,
    pub theme: Theme,
    pub show_volume: bool,
    pub rsi_period: usize,
    pub overbought: f64,
    pub oversold: f64,
}

impl Default for CandlestickRSIConfig {
    fn default() -> Self {
        Self {
            title: "Candlestick + RSI".to_string(),
            theme: Theme::Dark,
            show_volume: true,
            rsi_period: 14,
            overbought: 70.0,
            oversold: 30.0,
        }
    }
}

impl CandlestickRSIConfig {
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

    pub fn rsi_period(mut self, period: usize) -> Self {
        self.rsi_period = period.max(2);
        self
    }

    pub fn overbought(mut self, level: f64) -> Self {
        self.overbought = level;
        self
    }

    pub fn oversold(mut self, level: f64) -> Self {
        self.oversold = level;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &CandlestickRSIConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    // Layout: candles (60%) + volume (15%) + RSI (25%)
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

    let rsi_area = bottom;

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

    let rsi_vals = rsi(series, cfg.rsi_period);

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

    // RSI chart
    let mut rsi_chart = ChartBuilder::on(&rsi_area)
        .caption(
            format!("RSI({})", cfg.rsi_period),
            (TITLE_FONT, 16).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(50)
        .build_cartesian_2d(t_min..t_max, 0.0..100.0)
        .map_err(|e| BtError::Render(e.to_string()))?;

    rsi_chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Overbought/oversold lines
    rsi_chart
        .draw_series(LineSeries::new(
            vec![(t_min, cfg.overbought), (t_max, cfg.overbought)],
            cfg.theme.loss().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    rsi_chart
        .draw_series(LineSeries::new(
            vec![(t_min, cfg.oversold), (t_max, cfg.oversold)],
            cfg.theme.profit().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    rsi_chart
        .draw_series(LineSeries::new(
            vec![(t_min, 50.0), (t_max, 50.0)],
            cfg.theme.border().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    // RSI line
    rsi_chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !rsi_vals[i].is_nan() {
                    Some((c.t, rsi_vals[i]))
                } else {
                    None
                }
            }),
            cfg.theme.info().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Fill overbought/oversold regions
    for i in 0..series.candles.len() - 1 {
        if !rsi_vals[i].is_nan() && !rsi_vals[i + 1].is_nan() {
            let c1 = &series.candles[i];
            let c2 = &series.candles[i + 1];
            if rsi_vals[i] >= cfg.overbought && rsi_vals[i + 1] >= cfg.overbought {
                rsi_chart
                    .draw_series(std::iter::once(Polygon::new(
                        vec![
                            (c1.t, cfg.overbought),
                            (c2.t, cfg.overbought),
                            (c2.t, rsi_vals[i + 1].min(100.0)),
                            (c1.t, rsi_vals[i].min(100.0)),
                        ],
                        cfg.theme.loss().mix(0.1).filled(),
                    )))
                    .map_err(|e| BtError::Render(e.to_string()))?;
            } else if rsi_vals[i] <= cfg.oversold && rsi_vals[i + 1] <= cfg.oversold {
                rsi_chart
                    .draw_series(std::iter::once(Polygon::new(
                        vec![
                            (c1.t, cfg.oversold),
                            (c2.t, cfg.oversold),
                            (c2.t, rsi_vals[i + 1].max(0.0)),
                            (c1.t, rsi_vals[i].max(0.0)),
                        ],
                        cfg.theme.profit().mix(0.1).filled(),
                    )))
                    .map_err(|e| BtError::Render(e.to_string()))?;
            }
        }
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &CandlestickRSIConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &CandlestickRSIConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_constructs() {
        let cfg = CandlestickRSIConfig::new();
        let _ = cfg;
    }
}
