// crates/bt-viz/src/candlestick_gann.rs
// Author: Sourish Dey

//! Candlestick + Gann fan lines. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CandlestickGannConfig {
    pub title: String,
    pub theme: Theme,
    pub fan_lines: usize,
}

impl Default for CandlestickGannConfig {
    fn default() -> Self {
        Self {
            title: "Candlestick + Gann Fan".to_string(),
            theme: Theme::Dark,
            fan_lines: 5,
        }
    }
}

impl CandlestickGannConfig {
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

    pub fn fan_lines(mut self, n: usize) -> Self {
        self.fan_lines = n.clamp(3, 9);
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &CandlestickGannConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;
    let low = series.candles.iter().map(|c| c.low).fold(f64::MAX, f64::min);
    let high = series.candles.iter().map(|c| c.high).fold(f64::MIN, f64::max);
    let pad = (high - low) * 0.15;

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

    let candle_width = ((t_max - t_min) / series.candles.len() as f64).max(0.3) * 0.4;

    chart
        .draw_series(series.candles.iter().map(|c| {
            let color = if c.is_bullish() {
                cfg.theme.profit()
            } else {
                cfg.theme.loss()
            };
            CandleStick::new(
                c.t, c.open, c.high, c.low, c.close,
                color.filled(), color.filled(), (candle_width * 10.0) as u32,
            )
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    let anchor_price = (low + high) / 2.0;
    let anchor_t = (t_min + t_max) / 2.0;
    let t_range = t_max - t_min;
    let price_range = high - low + 2.0 * pad;

    for i in 0..cfg.fan_lines {
        let slope = (i + 1) as f64;
        let end_price = anchor_price + slope * price_range * 0.5;
        let end_t = anchor_t + t_range * 0.5;

        let color = if i % 2 == 0 {
            cfg.theme.info()
        } else {
            cfg.theme.accent()
        };

        chart
            .draw_series(LineSeries::new(
                vec![(anchor_t, anchor_price), (end_t, end_price)],
                color.mix(0.7).stroke_width(1),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &CandlestickGannConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &CandlestickGannConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = CandlestickGannConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_candlestick_gann.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
