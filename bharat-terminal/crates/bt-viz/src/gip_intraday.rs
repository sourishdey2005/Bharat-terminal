// crates/bt-viz/src/gip_intraday.rs
// Author: Sourish Dey

//! GIP: High-resolution intraday chart.
//! Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct GipIntradayConfig {
    pub title: String,
    pub theme: Theme,
    pub session_start: f64,
    pub session_end: f64,
    pub show_vwap: bool,
}

impl Default for GipIntradayConfig {
    fn default() -> Self {
        Self {
            title: "Intraday Chart".to_string(),
            theme: Theme::Dark,
            session_start: 9.25,
            session_end: 15.5,
            show_vwap: true,
        }
    }
}

impl GipIntradayConfig {
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
    pub fn session_start(mut self, s: f64) -> Self {
        self.session_start = s.max(0.0);
        self
    }
    pub fn session_end(mut self, e: f64) -> Self {
        self.session_end = e.max(0.0);
        self
    }
    pub fn show_vwap(mut self, s: bool) -> Self {
        self.show_vwap = s;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &GipIntradayConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

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

    // Candle timestamps are Unix seconds, so the x-axis must be built from the
    // data range. `session_start`/`session_end` are clock hours and are only
    // applied when the series is genuinely intraday (spans under two days).
    let data_t_min = series.candles.first().map(|c| c.t).unwrap_or(0.0);
    let data_t_max = series.candles.last().map(|c| c.t).unwrap_or(0.0);
    let span = data_t_max - data_t_min;
    let (t_min, t_max) = if span > 0.0 && span < 2.0 * 86_400.0 {
        let frac = |h: f64| (h / 24.0 * 86_400.0).min(86_400.0);
        let lo = data_t_min + frac(cfg.session_start);
        let hi = data_t_min + frac(cfg.session_end);
        if hi > lo {
            (lo, hi)
        } else {
            (data_t_min, data_t_max)
        }
    } else {
        (data_t_min, data_t_max.max(data_t_min + 1.0))
    };

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — {} (Intraday)", cfg.title, series.symbol),
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
        .x_desc("Time (hours)")
        .y_desc("Price")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let slot = ((t_max - t_min) / series.candles.len().max(1) as f64).max(1.0);
    let candle_width = ((slot * 0.6) as u32).max(1);

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
                candle_width,
            )
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    if cfg.show_vwap {
        let vwap = bt_analytics::vwap(series);
        chart
            .draw_series(LineSeries::new(
                series.candles.iter().enumerate().filter_map(|(i, c)| {
                    if !vwap[i].is_nan() {
                        Some((c.t, vwap[i]))
                    } else {
                        None
                    }
                }),
                cfg.theme.accent().stroke_width(2),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?
            .label("VWAP")
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

pub fn render_png(series: &OhlcvSeries, cfg: &GipIntradayConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &GipIntradayConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = GipIntradayConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_gip_intraday.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
