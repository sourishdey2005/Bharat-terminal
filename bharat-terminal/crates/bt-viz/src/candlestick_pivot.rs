// crates/bt-viz/src/candlestick_pivot.rs
// Author: Sourish Dey

//! Candlestick + Pivot Points. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CandlestickPivotConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for CandlestickPivotConfig {
    fn default() -> Self {
        Self {
            title: "Candlestick + Pivot Points".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl CandlestickPivotConfig {
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
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &CandlestickPivotConfig,
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
    let pad = (high - low) * 0.05;

    let last = series.candles.last().unwrap();
    let pivot = (last.high + last.low + last.close) / 3.0;
    let r1 = 2.0 * pivot - last.low;
    let s1 = 2.0 * pivot - last.high;
    let r2 = pivot + (last.high - last.low);
    let s2 = pivot - (last.high - last.low);
    let r3 = last.high + 2.0 * (pivot - last.low);
    let s3 = last.low - 2.0 * (last.high - pivot);

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

    for (label, val, color) in [
        ("R3", r3, cfg.theme.loss()),
        ("R2", r2, cfg.theme.loss()),
        ("R1", r1, cfg.theme.loss()),
        ("P", pivot, cfg.theme.info()),
        ("S1", s1, cfg.theme.profit()),
        ("S2", s2, cfg.theme.profit()),
        ("S3", s3, cfg.theme.profit()),
    ] {
        chart
            .draw_series(LineSeries::new(
                vec![(t_min, val), (t_max, val)],
                color.stroke_width(1),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?
            .label(label)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(1)));
    }

    chart
        .configure_series_labels()
        .border_style(&cfg.theme.border())
        .background_style(cfg.theme.background().mix(0.8))
        .label_font((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &CandlestickPivotConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &CandlestickPivotConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = CandlestickPivotConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_candlestick_pivot.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
