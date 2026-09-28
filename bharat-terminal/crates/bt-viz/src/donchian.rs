// crates/bt-viz/src/donchian.rs
// Author: Sourish Dey

//! Donchian channels. Made by Sourish Dey.

use bt_analytics::donchian;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct DonchianConfig {
    pub title: String,
    pub theme: Theme,
    pub period: usize,
}

impl Default for DonchianConfig {
    fn default() -> Self {
        Self {
            title: "Donchian Channels".to_string(),
            theme: Theme::Dark,
            period: 20,
        }
    }
}

impl DonchianConfig {
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
    cfg: &DonchianConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let (upper, middle, lower) = donchian(series, cfg.period);
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
            format!("{} — {} (Period: {})", cfg.title, series.symbol, cfg.period),
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
        if !upper[i].is_nan() && !upper[i + 1].is_nan() {
            chart
                .draw_series(std::iter::once(Polygon::new(
                    vec![
                        (series.candles[i].t, upper[i]),
                        (series.candles[i + 1].t, upper[i + 1]),
                        (series.candles[i + 1].t, lower[i + 1]),
                        (series.candles[i].t, lower[i]),
                    ],
                    cfg.theme.accent().mix(0.08).filled(),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !upper[i].is_nan() {
                    Some((c.t, upper[i]))
                } else {
                    None
                }
            }),
            cfg.theme.profit().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Upper")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                cfg.theme.profit().stroke_width(2),
            )
        });

    chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !lower[i].is_nan() {
                    Some((c.t, lower[i]))
                } else {
                    None
                }
            }),
            cfg.theme.loss().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Lower")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                cfg.theme.loss().stroke_width(2),
            )
        });

    chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !middle[i].is_nan() {
                    Some((c.t, middle[i]))
                } else {
                    None
                }
            }),
            cfg.theme.accent().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Middle")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                cfg.theme.accent().mix(0.5).stroke_width(1),
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

pub fn render_png(series: &OhlcvSeries, cfg: &DonchianConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &DonchianConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = DonchianConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_donchian.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
