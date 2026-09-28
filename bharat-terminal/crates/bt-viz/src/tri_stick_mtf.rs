// crates/bt-viz/src/tri_stick_mtf.rs
// Author: Sourish Dey

//! 2×2 multi-timeframe grid of the same symbol rendered with triangle
//! sticks (15m / 1h / 4h / 1d). Made by Sourish Dey.

use bt_analytics::sma;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

const TIMEFRAMES: [(usize, &str); 4] = [(1, "15m"), (4, "1h"), (16, "4h"), (96, "1d")];

#[derive(Debug, Clone)]
pub struct TriStickMtfConfig {
    pub title: String,
    pub theme: Theme,
    pub ma_period: usize,
}

impl Default for TriStickMtfConfig {
    fn default() -> Self {
        Self {
            title: "Tri-Stick MTF".to_string(),
            theme: Theme::Dark,
            ma_period: 10,
        }
    }
}

impl TriStickMtfConfig {
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

    pub fn ma_period(mut self, p: usize) -> Self {
        self.ma_period = p.max(2);
        self
    }
}

fn resample_series(series: &OhlcvSeries, factor: usize) -> OhlcvSeries {
    if factor <= 1 {
        return series.clone();
    }

    let mut candles = Vec::new();
    let mut i = 0;
    while i < series.candles.len() {
        let end = (i + factor).min(series.candles.len());
        let chunk = &series.candles[i..end];
        let first = &chunk[0];
        let last = &chunk[chunk.len() - 1];
        let high = chunk.iter().map(|c| c.high).fold(f64::NEG_INFINITY, f64::max);
        let low = chunk.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
        let volume: f64 = chunk.iter().map(|c| c.volume).sum();

        candles.push(bt_core::Candle::new(
            first.t,
            first.open,
            high,
            low,
            last.close,
            volume,
        ));

        i += factor;
    }

    OhlcvSeries::new(&series.symbol, candles)
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &TriStickMtfConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let (top, bottom) = root.split_vertically((50).percent());
    let (tl, tr) = top.split_horizontally((50).percent());
    let (bl, br) = bottom.split_horizontally((50).percent());
    let panels = [tl, tr, bl, br];

    for (idx, (factor, label)) in TIMEFRAMES.iter().enumerate() {
        let rs = resample_series(series, *factor);
        if rs.candles.is_empty() {
            continue;
        }

        let ma = sma(&rs, cfg.ma_period);
        let t_min = rs.candles.first().unwrap().t;
        let t_max = rs.candles.last().unwrap().t;
        let low = rs
            .candles
            .iter()
            .map(|c| c.low)
            .fold(f64::MAX, f64::min);
        let high = rs
            .candles
            .iter()
            .map(|c| c.high)
            .fold(f64::MIN, f64::max);
        let pad = (high - low).max(1.0) * 0.08;

        let mut chart = ChartBuilder::on(&panels[idx])
            .caption(
                format!("{} — {} ({})", cfg.title, series.symbol, label),
                (TITLE_FONT, 16).into_font().color(&cfg.theme.text()),
            )
            .margin(6)
            .x_label_area_size(20)
            .y_label_area_size(45)
            .build_cartesian_2d(t_min..t_max, (low - pad)..(high + pad))
            .map_err(|e| BtError::Render(e.to_string()))?;

        chart
            .configure_mesh()
            .label_style((LABEL_FONT, 9).into_font().color(&cfg.theme.text()))
            .axis_style(&cfg.theme.border())
            .light_line_style(cfg.theme.border().mix(0.3))
            .draw()
            .map_err(|e| BtError::Render(e.to_string()))?;

        let bar_width = (((t_max - t_min) / rs.candles.len() as f64).max(0.3) * 0.35).min(4.0);

        for c in &rs.candles {
            let color = if c.is_bullish() {
                cfg.theme.profit()
            } else {
                cfg.theme.loss()
            };

            chart
                .draw_series(std::iter::once(PathElement::new(
                    vec![(c.t, c.high), (c.t, c.low)],
                    color.stroke_width(1),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;

            chart
                .draw_series(std::iter::once(Polygon::new(
                    vec![
                        (c.t, c.close),
                        (c.t - bar_width, c.open),
                        (c.t + bar_width, c.open),
                    ],
                    color.filled(),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }

        chart
            .draw_series(LineSeries::new(
                rs.candles.iter().enumerate().filter_map(|(i, c)| {
                    if i < ma.len() && !ma[i].is_nan() {
                        Some((c.t, ma[i]))
                    } else {
                        None
                    }
                }),
                cfg.theme.info().stroke_width(1),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &TriStickMtfConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &TriStickMtfConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = TriStickMtfConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_tri_stick_mtf.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
