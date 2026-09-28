// crates/bt-viz/src/tri_stick_ha.rs
// Author: Sourish Dey

//! Heikin-Ashi smoothed values rendered as triangles. Made by Sourish Dey.

use bt_analytics::heikin_ashi;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct TriStickHaConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for TriStickHaConfig {
    fn default() -> Self {
        Self {
            title: "Heikin-Ashi Triangles".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl TriStickHaConfig {
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
    cfg: &TriStickHaConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;

    let (ha_open, ha_high, ha_low, ha_close) = heikin_ashi(series);

    let low_min = ha_low.iter().fold(f64::MAX, |a, &b| a.min(b));
    let high_max = ha_high.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let pad = (high_max - low_min) * 0.05;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — {}", cfg.title, series.symbol),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, (low_min - pad)..(high_max + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let half_w = ((t_max - t_min) / series.candles.len() as f64).max(0.3) * 0.35;

    for i in 0..series.candles.len() {
        let c = &series.candles[i];
        let color = if ha_close[i] >= ha_open[i] {
            cfg.theme.profit()
        } else {
            cfg.theme.loss()
        };

        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![
                    (c.t - half_w, ha_open[i]),
                    (c.t + half_w, ha_close[i]),
                    (c.t - half_w, ha_close[i]),
                ],
                color.filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;

        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![(c.t, ha_high[i]), (c.t, ha_low[i])],
                color.stroke_width(1),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &TriStickHaConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &TriStickHaConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = TriStickHaConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_tri_stick_ha.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
