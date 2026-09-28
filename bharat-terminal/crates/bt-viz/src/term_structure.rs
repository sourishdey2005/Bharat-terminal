// crates/bt-viz/src/term_structure.rs
// Author: Sourish Dey

//! Term structure curve family. Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct TermStructureConfig {
    pub title: String,
    pub theme: Theme,
    pub tenors: Vec<f64>,
    pub curves: Vec<(String, Vec<f64>)>,
}

impl Default for TermStructureConfig {
    fn default() -> Self {
        Self {
            title: "Term Structure".to_string(),
            theme: Theme::Dark,
            tenors: vec![],
            curves: vec![],
        }
    }
}

impl TermStructureConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn tenors(mut self, t: Vec<f64>) -> Self { self.tenors = t; self }
    pub fn curves(mut self, c: Vec<(String, Vec<f64>)>) -> Self { self.curves = c; self }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    cfg: &TermStructureConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    if cfg.curves.is_empty() || cfg.tenors.is_empty() {
        return Err(BtError::EmptySeries("term structure data".into()));
    }
    fill_background(&root, cfg.theme)?;

    let x_min = cfg.tenors.iter().cloned().fold(f64::MAX, f64::min);
    let x_max = cfg.tenors.iter().cloned().fold(f64::MIN, f64::max);
    let y_min = cfg
        .curves
        .iter()
        .flat_map(|(_, v)| v.iter().cloned())
        .fold(f64::MAX, f64::min);
    let y_max = cfg
        .curves
        .iter()
        .flat_map(|(_, v)| v.iter().cloned())
        .fold(f64::MIN, f64::max);
    let pad = (y_max - y_min).max(0.01) * 0.15;

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
        .x_desc("Tenor (years)")
        .y_desc("Rate (%)")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let palette = cfg.theme.categorical(0);
    for (idx, (label, values)) in cfg.curves.iter().enumerate() {
        let color = palette[idx % palette.len()];
        let points: Vec<(f64, f64)> = cfg
            .tenors
            .iter()
            .cloned()
            .zip(values.iter().cloned())
            .collect();
        chart
            .draw_series(LineSeries::new(points.clone(), color.stroke_width(2)))
            .map_err(|e| BtError::Render(e.to_string()))?
            .label(label.clone())
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2)));

        chart
            .draw_series(points.iter().map(|&(x, y)| Circle::new((x, y), 3, color.filled())))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

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

pub fn render_png(cfg: &TermStructureConfig, path: &str) -> Result<()> {
    render(png_root(path)?, cfg)
}

pub fn render_svg(cfg: &TermStructureConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let cfg = TermStructureConfig::new()
            .theme(Theme::Dark)
            .tenors(vec![0.25, 0.5, 1.0, 2.0, 5.0, 10.0])
            .curves(vec![
                ("Current".to_string(), vec![4.5, 4.6, 4.7, 4.8, 4.9, 5.0]),
                ("Previous".to_string(), vec![4.3, 4.4, 4.5, 4.6, 4.7, 4.8]),
            ]);
        let path = std::env::temp_dir().join("bt_test_term_structure.png").to_str().unwrap().to_string();
        render_png(&cfg, &path).unwrap();
    }
}
