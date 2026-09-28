// crates/bt-viz/src/macd_divergence.rs
// Author: Sourish Dey

//! MACD divergence signals. Made by Sourish Dey.

use bt_analytics::macd;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct MacdDivergenceConfig {
    pub title: String,
    pub theme: Theme,
    pub lookback: usize,
}

impl Default for MacdDivergenceConfig {
    fn default() -> Self {
        Self {
            title: "MACD Divergence".to_string(),
            theme: Theme::Dark,
            lookback: 14,
        }
    }
}

impl MacdDivergenceConfig {
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

    pub fn lookback(mut self, lb: usize) -> Self {
        self.lookback = lb.max(5);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum DivergenceType {
    Bullish,
    Bearish,
}

#[derive(Debug, Clone)]
struct Divergence {
    bar_idx: usize,
    dtype: DivergenceType,
    price: f64,
    macd_val: f64,
}

fn find_divergences(series: &OhlcvSeries, macd_line: &[f64], lookback: usize) -> Vec<Divergence> {
    let mut divergences = Vec::new();
    let n = series.candles.len();

    for i in lookback..n {
        if macd_line[i].is_nan() {
            continue;
        }

        let mut price_low = f64::MAX;
        let mut macd_low = f64::MAX;
        let mut price_low_idx = i;
        let mut macd_low_idx = i;

        let mut price_high = f64::MIN;
        let mut macd_high = f64::MIN;
        let mut price_high_idx = i;
        let mut macd_high_idx = i;

        for j in (i - lookback)..=i {
            if series.candles[j].low < price_low {
                price_low = series.candles[j].low;
                price_low_idx = j;
            }
            if !macd_line[j].is_nan() && macd_line[j] < macd_low {
                macd_low = macd_line[j];
                macd_low_idx = j;
            }
            if series.candles[j].high > price_high {
                price_high = series.candles[j].high;
                price_high_idx = j;
            }
            if !macd_line[j].is_nan() && macd_line[j] > macd_high {
                macd_high = macd_line[j];
                macd_high_idx = j;
            }
        }

        if price_low_idx != macd_low_idx
            && series.candles[i].low <= price_low
            && macd_line[i] > macd_low
        {
            divergences.push(Divergence {
                bar_idx: i,
                dtype: DivergenceType::Bullish,
                price: series.candles[i].low,
                macd_val: macd_line[i],
            });
        }

        if price_high_idx != macd_high_idx
            && series.candles[i].high >= price_high
            && macd_line[i] < macd_high
        {
            divergences.push(Divergence {
                bar_idx: i,
                dtype: DivergenceType::Bearish,
                price: series.candles[i].high,
                macd_val: macd_line[i],
            });
        }
    }

    divergences
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &MacdDivergenceConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let (macd_line, signal_line, histogram) = macd(series);
    let divergences = find_divergences(series, &macd_line, cfg.lookback);

    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;

    let (price_area, macd_area) = root.split_vertically((60).percent());

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

    let mut price_chart = ChartBuilder::on(&price_area)
        .caption(
            format!("{} — {}", cfg.title, series.symbol),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, (low - pad)..(high + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    price_chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let candle_width = ((t_max - t_min) / series.candles.len() as f64).max(0.3) * 0.4;

    price_chart
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

    for div in &divergences {
        let color = if div.dtype == DivergenceType::Bullish {
            cfg.theme.profit()
        } else {
            cfg.theme.loss()
        };
        let label = if div.dtype == DivergenceType::Bullish {
            "BULL DIV"
        } else {
            "BEAR DIV"
        };
        price_chart
            .draw_series(std::iter::once(Circle::new(
                (series.candles[div.bar_idx].t, div.price),
                6,
                color.filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
        price_chart
            .draw_series(std::iter::once(Text::new(
                label.to_string(),
                (series.candles[div.bar_idx].t, div.price),
                (LABEL_FONT, 10).into_font().color(&color),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

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

    let mut macd_chart = ChartBuilder::on(&macd_area)
        .caption(
            "MACD",
            (TITLE_FONT, 16).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, y_min..y_max)
        .map_err(|e| BtError::Render(e.to_string()))?;

    macd_chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    macd_chart
        .draw_series(LineSeries::new(
            vec![(t_min, 0.0), (t_max, 0.0)],
            cfg.theme.border().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

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
        .map_err(|e| BtError::Render(e.to_string()))?;

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
        .map_err(|e| BtError::Render(e.to_string()))?;

    for div in &divergences {
        let color = if div.dtype == DivergenceType::Bullish {
            cfg.theme.profit()
        } else {
            cfg.theme.loss()
        };
        macd_chart
            .draw_series(std::iter::once(Circle::new(
                (series.candles[div.bar_idx].t, div.macd_val),
                5,
                color.filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &MacdDivergenceConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &MacdDivergenceConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = MacdDivergenceConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_macd_divergence.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
