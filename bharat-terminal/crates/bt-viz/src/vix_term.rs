// crates/bt-viz/src/vix_term.rs
// Author: Sourish Dey

//! VIX term structure. Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct VixTermConfig {
    pub title: String,
    pub theme: Theme,
    pub tenors: Vec<f64>,
    pub vix_values: Vec<f64>,
    pub spot_vix: f64,
}

impl Default for VixTermConfig {
    fn default() -> Self {
        Self {
            title: "VIX Term Structure".to_string(),
            theme: Theme::Dark,
            tenors: vec![],
            vix_values: vec![],
            spot_vix: 15.0,
        }
    }
}

impl VixTermConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn tenors(mut self, t: Vec<f64>) -> Self { self.tenors = t; self }
    pub fn vix_values(mut self, v: Vec<f64>) -> Self { self.vix_values = v; self }
    pub fn spot_vix(mut self, s: f64) -> Self { self.spot_vix = s; self }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    cfg: &VixTermConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    if cfg.tenors.is_empty() || cfg.vix_values.is_empty() {
        return Err(BtError::EmptySeries("VIX term data".into()));
    }
    fill_background(&root, cfg.theme)?;

    let x_min = cfg.tenors.iter().cloned().fold(f64::MAX, f64::min);
    let x_max = cfg.tenors.iter().cloned().fold(f64::MIN, f64::max);
    let y_min = cfg.vix_values.iter().cloned().fold(f64::MAX, f64::min);
    let y_max = cfg.vix_values.iter().cloned().fold(f64::MIN, f64::max);
    let pad = (y_max - y_min).max(0.5) * 0.15;

    let mut chart = ChartBuilder::on(&root)
        .caption(&cfg.title, (TITLE_FONT, 22).into_font().color(&cfg.theme.text()))
        .margin(15)
        .x_label_area_size(35)
        .y_label_area_size(60)
        .build_cartesian_2d(x_min..x_max, (y_min - pad)..(y_max + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .x_desc("Tenor (days)")
        .y_desc("VIX")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let points: Vec<(f64, f64)> = cfg
        .tenors
        .iter()
        .cloned()
        .zip(cfg.vix_values.iter().cloned())
        .collect();

    chart
        .draw_series(LineSeries::new(points.clone(), cfg.theme.accent().stroke_width(3)))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(points.iter().map(|&(x, y)| Circle::new((x, y), 5, cfg.theme.accent().filled())))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(x_min, cfg.spot_vix), (x_max, cfg.spot_vix)],
            cfg.theme.info().stroke_width(2),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label(format!("Spot VIX: {:.1}", cfg.spot_vix))
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.info().stroke_width(2)));

    chart
        .configure_series_labels()
        .background_style(cfg.theme.background().mix(0.8))
        .border_style(cfg.theme.border())
        .label_font((LABEL_FONT, 13).into_font().color(&cfg.theme.text()))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(cfg: &VixTermConfig, path: &str) -> Result<()> {
    render(png_root(path)?, cfg)
}

pub fn render_svg(cfg: &VixTermConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let cfg = VixTermConfig::new()
            .theme(Theme::Dark)
            .tenors(vec![7.0, 14.0, 30.0, 60.0, 90.0, 180.0, 365.0])
            .vix_values(vec![18.0, 17.5, 16.8, 16.2, 15.8, 15.5, 15.2])
            .spot_vix(15.0);
        let path = std::env::temp_dir().join("bt_test_vix_term.png").to_str().unwrap().to_string();
        render_png(&cfg, &path).unwrap();
    }
}
