// crates/bt-viz/src/rolling_max_dd.rs
// Author: Sourish Dey

//! Rolling max drawdown chart. Made by Sourish Dey.

use bt_analytics::rolling_max_drawdown;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct RollingMaxDdConfig {
    pub title: String,
    pub theme: Theme,
    pub window: usize,
}

impl Default for RollingMaxDdConfig {
    fn default() -> Self {
        Self {
            title: "Rolling Max Drawdown".to_string(),
            theme: Theme::Dark,
            window: 20,
        }
    }
}

impl RollingMaxDdConfig {
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
    pub fn window(mut self, w: usize) -> Self {
        self.window = w;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &RollingMaxDdConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let rdd = rolling_max_drawdown(series, cfg.window);
    let valid: Vec<f64> = rdd.iter().filter(|v| v.is_finite()).cloned().collect();
    if valid.is_empty() {
        return Err(BtError::EmptySeries("rolling max drawdown".into()));
    }
    let y_max = valid.iter().cloned().fold(0.0_f64, f64::max);
    let n = rdd.len() as f64;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            &cfg.title,
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(15)
        .x_label_area_size(35)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..n, 0.0..(y_max * 1.15))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .x_desc("Bar")
        .y_desc("Max Drawdown (%)")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let points: Vec<(f64, f64)> = rdd
        .iter()
        .enumerate()
        .filter(|(_, v)| v.is_finite())
        .map(|(i, v)| (i as f64, *v))
        .collect();

    chart
        .draw_series(
            AreaSeries::new(points.clone(), 0.0, cfg.theme.loss().mix(0.35))
                .border_style(cfg.theme.loss().stroke_width(2)),
        )
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &RollingMaxDdConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &RollingMaxDdConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = RollingMaxDdConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_rolling_max_dd.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
