// crates/bt-viz/src/tri_stick_delta.rs
// Author: Sourish Dey

//! Two overlapping triangles: green (buy) and red (sell) per bar.
//! Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct TriStickDeltaConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for TriStickDeltaConfig {
    fn default() -> Self {
        Self {
            title: "Delta Triangles".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl TriStickDeltaConfig {
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
    cfg: &TriStickDeltaConfig,
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

    let half_w = ((t_max - t_min) / series.candles.len() as f64).max(0.3) * 0.35;

    for c in &series.candles {
        let body_top = c.open.max(c.close);
        let body_bot = c.open.min(c.close);
        let range = (c.high - c.low).max(1e-9);
        let buy_frac = (c.close - c.low) / range;
        let sell_frac = 1.0 - buy_frac;

        let buy_h = (body_top - body_bot) * buy_frac;
        let sell_h = (body_top - body_bot) * sell_frac;

        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![
                    (c.t - half_w, body_bot),
                    (c.t + half_w, body_bot),
                    (c.t, body_bot + buy_h),
                ],
                cfg.theme.profit().mix(0.7).filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;

        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![
                    (c.t - half_w, body_top),
                    (c.t + half_w, body_top),
                    (c.t, body_top - sell_h),
                ],
                cfg.theme.loss().mix(0.7).filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;

        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![(c.t, c.high), (c.t, c.low)],
                cfg.theme.text().stroke_width(1),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &TriStickDeltaConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &TriStickDeltaConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = TriStickDeltaConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_tri_stick_delta.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
