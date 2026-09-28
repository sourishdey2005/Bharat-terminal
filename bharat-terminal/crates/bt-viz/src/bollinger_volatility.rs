// crates/bt-viz/src/bollinger_volatility.rs
// Author: Sourish Dey

//! Bollinger volatility analysis — visualizes band width as a volatility
//! proxy with percentile-based regime classification. Made by Sourish Dey.

use bt_analytics::bollinger;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct BollingerVolatilityConfig {
    pub title: String,
    pub theme: Theme,
    pub period: usize,
    pub num_std: f64,
}

impl Default for BollingerVolatilityConfig {
    fn default() -> Self {
        Self {
            title: "Bollinger Volatility".to_string(),
            theme: Theme::Dark,
            period: 20,
            num_std: 2.0,
        }
    }
}

impl BollingerVolatilityConfig {
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

    pub fn period(mut self, p: usize) -> Self {
        self.period = p.max(2);
        self
    }

    pub fn num_std(mut self, s: f64) -> Self {
        self.num_std = s.max(0.5);
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &BollingerVolatilityConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let (middle, upper, lower) = bollinger(series, cfg.period, cfg.num_std);

    let band_width: Vec<f64> = (0..series.candles.len())
        .map(|i| {
            if !middle[i].is_nan() && middle[i] != 0.0 {
                (upper[i] - lower[i]) / middle[i] * 100.0
            } else {
                f64::NAN
            }
        })
        .collect();

    let mut valid_bw: Vec<f64> = band_width.iter().filter(|v| !v.is_nan()).cloned().collect();
    valid_bw.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let p25 = if !valid_bw.is_empty() {
        valid_bw[valid_bw.len() / 4]
    } else {
        0.0
    };
    let p75 = if !valid_bw.is_empty() {
        valid_bw[valid_bw.len() * 3 / 4]
    } else {
        0.0
    };

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

    let (price_area, vol_area) = root.split_vertically((55).percent());

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

    for i in 0..series.candles.len() - 1 {
        if !upper[i].is_nan() && !upper[i + 1].is_nan() {
            price_chart
                .draw_series(std::iter::once(Polygon::new(
                    vec![
                        (series.candles[i].t, upper[i]),
                        (series.candles[i + 1].t, upper[i + 1]),
                        (series.candles[i + 1].t, lower[i + 1]),
                        (series.candles[i].t, lower[i]),
                    ],
                    cfg.theme.info().mix(0.08).filled(),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    price_chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !upper[i].is_nan() {
                    Some((c.t, upper[i]))
                } else {
                    None
                }
            }),
            cfg.theme.info().stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Upper")
        .legend(|(x, y)| {
            PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.info().stroke_width(1))
        });

    price_chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !lower[i].is_nan() {
                    Some((c.t, lower[i]))
                } else {
                    None
                }
            }),
            cfg.theme.info().stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Lower")
        .legend(|(x, y)| {
            PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.info().stroke_width(1))
        });

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

    let bw_min = valid_bw.first().cloned().unwrap_or(0.0);
    let bw_max = valid_bw.last().cloned().unwrap_or(1.0);
    let bw_range = (bw_max - bw_min).max(1e-6);

    let mut vol_chart = ChartBuilder::on(&vol_area)
        .caption(
            format!("Volatility (P25: {:.2}% P75: {:.2}%)", p25, p75),
            (TITLE_FONT, 16).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(
            t_min..t_max,
            (bw_min - bw_range * 0.1)..(bw_max + bw_range * 0.1),
        )
        .map_err(|e| BtError::Render(e.to_string()))?;

    vol_chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    vol_chart
        .draw_series(LineSeries::new(
            vec![(t_min, p25), (t_max, p25)],
            cfg.theme.loss().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("P25")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                cfg.theme.loss().mix(0.5).stroke_width(1),
            )
        });

    vol_chart
        .draw_series(LineSeries::new(
            vec![(t_min, p75), (t_max, p75)],
            cfg.theme.profit().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("P75")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                cfg.theme.profit().mix(0.5).stroke_width(1),
            )
        });

    vol_chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !band_width[i].is_nan() {
                    Some((c.t, band_width[i]))
                } else {
                    None
                }
            }),
            cfg.theme.accent().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Width")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                cfg.theme.accent().stroke_width(2),
            )
        });

    vol_chart
        .draw_series(series.candles.iter().enumerate().filter_map(|(i, c)| {
            if !band_width[i].is_nan() {
                let color = if band_width[i] > p75 {
                    cfg.theme.profit().mix(0.3)
                } else if band_width[i] < p25 {
                    cfg.theme.loss().mix(0.3)
                } else {
                    cfg.theme.border().mix(0.2)
                };
                Some(Rectangle::new(
                    [
                        (c.t - candle_width, bw_min),
                        (c.t + candle_width, band_width[i]),
                    ],
                    color.filled(),
                ))
            } else {
                None
            }
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    price_chart
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

pub fn render_png(series: &OhlcvSeries, cfg: &BollingerVolatilityConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &BollingerVolatilityConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = BollingerVolatilityConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_bollinger_volatility.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
