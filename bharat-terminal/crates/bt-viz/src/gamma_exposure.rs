// crates/bt-viz/src/gamma_exposure.rs
// Author: Sourish Dey

//! Gamma exposure profile. Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct GammaExposureConfig {
    pub title: String,
    pub theme: Theme,
    pub strikes: Vec<f64>,
    pub gamma_values: Vec<f64>,
    pub spot: f64,
}

impl Default for GammaExposureConfig {
    fn default() -> Self {
        Self {
            title: "Gamma Exposure".to_string(),
            theme: Theme::Dark,
            strikes: vec![],
            gamma_values: vec![],
            spot: 100.0,
        }
    }
}

impl GammaExposureConfig {
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
    pub fn strikes(mut self, s: Vec<f64>) -> Self {
        self.strikes = s;
        self
    }
    pub fn gamma_values(mut self, g: Vec<f64>) -> Self {
        self.gamma_values = g;
        self
    }
    pub fn spot(mut self, s: f64) -> Self {
        self.spot = s;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    cfg: &GammaExposureConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if cfg.strikes.is_empty() || cfg.gamma_values.is_empty() {
        return Err(BtError::EmptySeries("gamma exposure data".into()));
    }
    fill_background(&root, cfg.theme)?;

    let x_min = cfg.strikes.iter().cloned().fold(f64::MAX, f64::min);
    let x_max = cfg.strikes.iter().cloned().fold(f64::MIN, f64::max);
    let y_max = cfg.gamma_values.iter().cloned().fold(0.0_f64, f64::max);
    let y_min = cfg.gamma_values.iter().cloned().fold(0.0_f64, f64::min);
    let pad = (y_max - y_min).max(0.001) * 0.15;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            &cfg.title,
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(15)
        .x_label_area_size(35)
        .y_label_area_size(60)
        .build_cartesian_2d(x_min..x_max, (y_min - pad)..(y_max + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .x_desc("Strike")
        .y_desc("Gamma")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let points: Vec<(f64, f64)> = cfg
        .strikes
        .iter()
        .cloned()
        .zip(cfg.gamma_values.iter().cloned())
        .collect();

    chart
        .draw_series(
            AreaSeries::new(points.clone(), 0.0, cfg.theme.profit().mix(0.35))
                .border_style(cfg.theme.profit().stroke_width(2)),
        )
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(x_min, 0.0), (x_max, 0.0)],
            cfg.theme.border().stroke_width(1),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(cfg.spot, y_min - pad), (cfg.spot, y_max + pad)],
            cfg.theme.accent().stroke_width(2),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(std::iter::once(Text::new(
            format!("Spot: {:.0}", cfg.spot),
            (cfg.spot, y_max + pad * 0.5),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.accent()),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(cfg: &GammaExposureConfig, path: &str) -> Result<()> {
    render(png_root(path)?, cfg)
}

pub fn render_svg(cfg: &GammaExposureConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let cfg = GammaExposureConfig::new()
            .theme(Theme::Dark)
            .strikes(vec![80.0, 90.0, 95.0, 100.0, 105.0, 110.0, 120.0])
            .gamma_values(vec![0.001, 0.005, 0.015, 0.025, 0.015, 0.005, 0.001])
            .spot(100.0);
        let path = std::env::temp_dir()
            .join("bt_test_gamma_exposure.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&cfg, &path).unwrap();
    }
}
