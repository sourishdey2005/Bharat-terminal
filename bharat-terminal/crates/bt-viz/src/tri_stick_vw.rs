// crates/bt-viz/src/tri_stick_vw.rs
// Author: Sourish Dey

//! Triangle width scales with volume. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct TriStickVwConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for TriStickVwConfig {
    fn default() -> Self {
        Self {
            title: "Volume-Weighted Triangles".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl TriStickVwConfig {
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
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &TriStickVwConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

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
    let max_vol = series
        .candles
        .iter()
        .map(|c| c.volume)
        .fold(0.0_f64, f64::max);
    let pad = (high - low) * 0.05;

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

    let slot = ((t_max - t_min) / series.candles.len() as f64).max(0.3);

    for c in &series.candles {
        let color = if c.is_bullish() {
            cfg.theme.profit()
        } else {
            cfg.theme.loss()
        };

        let vol_frac = if max_vol > 0.0 {
            c.volume / max_vol
        } else {
            0.5
        };
        let half_w = slot * 0.15 + slot * 0.35 * vol_frac;

        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![
                    (c.t - half_w, c.open),
                    (c.t + half_w, c.close),
                    (c.t - half_w, c.close),
                ],
                color.filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;

        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![(c.t, c.high), (c.t, c.low)],
                color.stroke_width(1),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &TriStickVwConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &TriStickVwConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = TriStickVwConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_tri_stick_vw.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
