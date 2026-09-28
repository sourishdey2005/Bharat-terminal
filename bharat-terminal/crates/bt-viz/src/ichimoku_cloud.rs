// crates/bt-viz/src/ichimoku_cloud.rs
// Author: Sourish Dey

//! Standalone Ichimoku cloud chart — full Kinko Hyo visualization with
//! cloud projection, signal markers, and trend classification.
//! Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct IchimokuCloudConfig {
    pub title: String,
    pub theme: Theme,
    pub tenkan_period: usize,
    pub kijun_period: usize,
    pub senkou_b_period: usize,
    pub show_chikou: bool,
}

impl Default for IchimokuCloudConfig {
    fn default() -> Self {
        Self {
            title: "Ichimoku Cloud".to_string(),
            theme: Theme::Dark,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            show_chikou: true,
        }
    }
}

impl IchimokuCloudConfig {
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

    pub fn tenkan_period(mut self, p: usize) -> Self {
        self.tenkan_period = p.max(2);
        self
    }

    pub fn kijun_period(mut self, p: usize) -> Self {
        self.kijun_period = p.max(2);
        self
    }

    pub fn senkou_b_period(mut self, p: usize) -> Self {
        self.senkou_b_period = p.max(2);
        self
    }

    pub fn show_chikou(mut self, show: bool) -> Self {
        self.show_chikou = show;
        self
    }
}

#[derive(Debug, Clone)]
struct IchimokuLines {
    tenkan: Vec<f64>,
    kijun: Vec<f64>,
    senkou_a: Vec<f64>,
    senkou_b: Vec<f64>,
    chikou: Vec<f64>,
}

fn compute_ichimoku(
    series: &OhlcvSeries,
    tenkan_p: usize,
    kijun_p: usize,
    senkou_b_p: usize,
) -> IchimokuLines {
    let n = series.candles.len();
    let mut tenkan = vec![f64::NAN; n];
    let mut kijun = vec![f64::NAN; n];
    let mut senkou_a = vec![f64::NAN; n];
    let mut senkou_b = vec![f64::NAN; n];
    let mut chikou = vec![f64::NAN; n];

    for i in 0..n {
        if i >= tenkan_p - 1 {
            let window = &series.candles[i + 1 - tenkan_p..=i];
            let hh = window.iter().map(|c| c.high).fold(f64::NEG_INFINITY, f64::max);
            let ll = window.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
            tenkan[i] = (hh + ll) / 2.0;
        }
        if i >= kijun_p - 1 {
            let window = &series.candles[i + 1 - kijun_p..=i];
            let hh = window.iter().map(|c| c.high).fold(f64::NEG_INFINITY, f64::max);
            let ll = window.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
            kijun[i] = (hh + ll) / 2.0;
        }
        if i >= senkou_b_p - 1 {
            let window = &series.candles[i + 1 - senkou_b_p..=i];
            let hh = window.iter().map(|c| c.high).fold(f64::NEG_INFINITY, f64::max);
            let ll = window.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
            senkou_b[i] = (hh + ll) / 2.0;
        }
        if !tenkan[i].is_nan() && !kijun[i].is_nan() {
            senkou_a[i] = (tenkan[i] + kijun[i]) / 2.0;
        }
        if i + kijun_p < n {
            chikou[i] = series.candles[i + kijun_p].close;
        }
    }

    IchimokuLines {
        tenkan,
        kijun,
        senkou_a,
        senkou_b,
        chikou,
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &IchimokuCloudConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let ichi = compute_ichimoku(series, cfg.tenkan_period, cfg.kijun_period, cfg.senkou_b_period);

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

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!(
                "{} — {} ({},{},{})",
                cfg.title, series.symbol, cfg.tenkan_period, cfg.kijun_period, cfg.senkou_b_period
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

    for i in 0..series.candles.len() - 1 {
        if !ichi.senkou_a[i].is_nan()
            && !ichi.senkou_a[i + 1].is_nan()
            && !ichi.senkou_b[i].is_nan()
            && !ichi.senkou_b[i + 1].is_nan()
        {
            let is_bull = ichi.senkou_a[i] > ichi.senkou_b[i];
            let color = if is_bull {
                cfg.theme.profit().mix(0.15)
            } else {
                cfg.theme.loss().mix(0.15)
            };
            chart
                .draw_series(std::iter::once(Polygon::new(
                    vec![
                        (series.candles[i].t, ichi.senkou_a[i]),
                        (series.candles[i + 1].t, ichi.senkou_a[i + 1]),
                        (series.candles[i + 1].t, ichi.senkou_b[i + 1]),
                        (series.candles[i].t, ichi.senkou_b[i]),
                    ],
                    color.filled(),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !ichi.tenkan[i].is_nan() {
                    Some((c.t, ichi.tenkan[i]))
                } else {
                    None
                }
            }),
            cfg.theme.profit().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Tenkan")
        .legend(|(x, y)| {
            PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.profit().stroke_width(2))
        });

    chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !ichi.kijun[i].is_nan() {
                    Some((c.t, ichi.kijun[i]))
                } else {
                    None
                }
            }),
            cfg.theme.loss().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Kijun")
        .legend(|(x, y)| {
            PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.loss().stroke_width(2))
        });

    chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !ichi.senkou_a[i].is_nan() {
                    Some((c.t, ichi.senkou_a[i]))
                } else {
                    None
                }
            }),
            cfg.theme.info().stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Senkou A")
        .legend(|(x, y)| {
            PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.info().stroke_width(1))
        });

    chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !ichi.senkou_b[i].is_nan() {
                    Some((c.t, ichi.senkou_b[i]))
                } else {
                    None
                }
            }),
            cfg.theme.accent().stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Senkou B")
        .legend(|(x, y)| {
            PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.accent().stroke_width(1))
        });

    if cfg.show_chikou {
        chart
            .draw_series(LineSeries::new(
                series.candles.iter().enumerate().filter_map(|(i, c)| {
                    if !ichi.chikou[i].is_nan() {
                        Some((c.t, ichi.chikou[i]))
                    } else {
                        None
                    }
                }),
                cfg.theme.border().mix(0.7).stroke_width(1),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?
            .label("Chikou")
            .legend(|(x, y)| {
                PathElement::new(
                    vec![(x, y), (x + 20, y)],
                    cfg.theme.border().mix(0.7).stroke_width(1),
                )
            });
    }

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

    for i in 1..series.candles.len() {
        if ichi.tenkan[i].is_nan() || ichi.kijun[i].is_nan() {
            continue;
        }
        let prev_diff = ichi.tenkan[i - 1] - ichi.kijun[i - 1];
        let curr_diff = ichi.tenkan[i] - ichi.kijun[i];

        if prev_diff <= 0.0 && curr_diff > 0.0 {
            chart
                .draw_series(std::iter::once(Circle::new(
                    (series.candles[i].t, series.candles[i].low),
                    5,
                    cfg.theme.profit().filled(),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;
        } else if prev_diff >= 0.0 && curr_diff < 0.0 {
            chart
                .draw_series(std::iter::once(Circle::new(
                    (series.candles[i].t, series.candles[i].high),
                    5,
                    cfg.theme.loss().filled(),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

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

pub fn render_png(series: &OhlcvSeries, cfg: &IchimokuCloudConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &IchimokuCloudConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = IchimokuCloudConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_ichimoku_cloud.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
