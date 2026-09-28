// crates/bt-viz/src/macd_histogram.rs
// Author: Sourish Dey

//! MACD histogram. Made by Sourish Dey.

use bt_analytics::macd;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct MacdHistogramConfig {
    pub title: String,
    pub theme: Theme,
    pub show_signal: bool,
}

impl Default for MacdHistogramConfig {
    fn default() -> Self {
        Self {
            title: "MACD Histogram".to_string(),
            theme: Theme::Dark,
            show_signal: true,
        }
    }
}

impl MacdHistogramConfig {
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

    pub fn show_signal(mut self, show: bool) -> Self {
        self.show_signal = show;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &MacdHistogramConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let (macd_line, signal_line, histogram) = macd(series);
    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;

    let hist_min = histogram
        .iter()
        .filter(|v| !v.is_nan())
        .fold(f64::INFINITY, |a, &b| a.min(b));
    let hist_max = histogram
        .iter()
        .filter(|v| !v.is_nan())
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let range = (hist_max - hist_min).max(1e-6);
    let y_min = hist_min - range * 0.2;
    let y_max = hist_max + range * 0.2;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — {}", cfg.title, series.symbol),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, y_min..y_max)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(LineSeries::new(
            vec![(t_min, 0.0), (t_max, 0.0)],
            cfg.theme.border().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

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

    if cfg.show_signal {
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
            .map_err(|e| BtError::Render(e.to_string()))?
            .label("MACD")
            .legend(|(x, y)| {
                PathElement::new(
                    vec![(x, y), (x + 20, y)],
                    cfg.theme.info().stroke_width(2),
                )
            });

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
            .map_err(|e| BtError::Render(e.to_string()))?
            .label("Signal")
            .legend(|(x, y)| {
                PathElement::new(
                    vec![(x, y), (x + 20, y)],
                    cfg.theme.accent().stroke_width(2),
                )
            });

        chart
            .configure_series_labels()
            .background_style(cfg.theme.background().mix(0.9))
            .border_style(cfg.theme.border())
            .label_font((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
            .draw()
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &MacdHistogramConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &MacdHistogramConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = MacdHistogramConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_macd_histogram.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
