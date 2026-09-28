// crates/bt-viz/src/bollinger_double.rs
// Author: Sourish Dey

//! Double Bollinger Bands — plots two sets of bands (1σ and 2σ) to
//! identify zones of support/resistance and volatility regime shifts.
//! Made by Sourish Dey.

use bt_analytics::bollinger;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct BollingerDoubleConfig {
    pub title: String,
    pub theme: Theme,
    pub period: usize,
}

impl Default for BollingerDoubleConfig {
    fn default() -> Self {
        Self {
            title: "Double Bollinger Bands".to_string(),
            theme: Theme::Dark,
            period: 20,
        }
    }
}

impl BollingerDoubleConfig {
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
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &BollingerDoubleConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let (mid1, up1, lo1) = bollinger(series, cfg.period, 1.0);
    let (_mid2, up2, lo2) = bollinger(series, cfg.period, 2.0);

    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;

    let all_values: Vec<f64> = series
        .candles
        .iter()
        .flat_map(|c| vec![c.low, c.high])
        .chain(up2.iter().filter(|v| !v.is_nan()).cloned())
        .chain(lo2.iter().filter(|v| !v.is_nan()).cloned())
        .collect();
    let low = all_values.iter().fold(f64::MAX, |a, &b| a.min(b));
    let high = all_values.iter().fold(f64::MIN, |a, &b| a.max(b));
    let pad = (high - low).max(1.0) * 0.05;

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

    for i in 0..series.candles.len() - 1 {
        if !up2[i].is_nan() && !up2[i + 1].is_nan() {
            chart
                .draw_series(std::iter::once(Polygon::new(
                    vec![
                        (series.candles[i].t, up2[i]),
                        (series.candles[i + 1].t, up2[i + 1]),
                        (series.candles[i + 1].t, lo2[i + 1]),
                        (series.candles[i].t, lo2[i]),
                    ],
                    cfg.theme.info().mix(0.05).filled(),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    for i in 0..series.candles.len() - 1 {
        if !up1[i].is_nan() && !up1[i + 1].is_nan() {
            chart
                .draw_series(std::iter::once(Polygon::new(
                    vec![
                        (series.candles[i].t, up1[i]),
                        (series.candles[i + 1].t, up1[i + 1]),
                        (series.candles[i + 1].t, lo1[i + 1]),
                        (series.candles[i].t, lo1[i]),
                    ],
                    cfg.theme.accent().mix(0.08).filled(),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !up2[i].is_nan() {
                    Some((c.t, up2[i]))
                } else {
                    None
                }
            }),
            cfg.theme.info().mix(0.7).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("+2σ")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                cfg.theme.info().mix(0.7).stroke_width(1),
            )
        });

    chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !lo2[i].is_nan() {
                    Some((c.t, lo2[i]))
                } else {
                    None
                }
            }),
            cfg.theme.info().mix(0.7).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("-2σ")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                cfg.theme.info().mix(0.7).stroke_width(1),
            )
        });

    chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !up1[i].is_nan() {
                    Some((c.t, up1[i]))
                } else {
                    None
                }
            }),
            cfg.theme.accent().stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("+1σ")
        .legend(|(x, y)| {
            PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.accent().stroke_width(1))
        });

    chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !lo1[i].is_nan() {
                    Some((c.t, lo1[i]))
                } else {
                    None
                }
            }),
            cfg.theme.accent().stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("-1σ")
        .legend(|(x, y)| {
            PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.accent().stroke_width(1))
        });

    chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !mid1[i].is_nan() {
                    Some((c.t, mid1[i]))
                } else {
                    None
                }
            }),
            cfg.theme.border().mix(0.7).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("SMA")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                cfg.theme.border().mix(0.7).stroke_width(1),
            )
        });

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

    chart
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

pub fn render_png(series: &OhlcvSeries, cfg: &BollingerDoubleConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &BollingerDoubleConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = BollingerDoubleConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_bollinger_double.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
