// crates/bt-viz/src/vdat_cone.rs
// Author: Sourish Dey

//! VDAT: Volatility cones.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct VdatConeConfig {
    pub title: String,
    pub theme: Theme,
    pub percentiles: Vec<f64>,
    pub tenors: Vec<f64>,
    pub cone_data: Vec<Vec<f64>>,
    pub current_vol: f64,
}

impl Default for VdatConeConfig {
    fn default() -> Self {
        Self {
            title: "Volatility Cones".to_string(),
            theme: Theme::Dark,
            percentiles: vec![0.1, 0.25, 0.5, 0.75, 0.9],
            tenors: vec![0.25, 0.5, 1.0, 2.0, 3.0, 5.0],
            cone_data: vec![vec![20.0; 6]; 5],
            current_vol: 22.0,
        }
    }
}

impl VdatConeConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn percentiles(mut self, p: Vec<f64>) -> Self { self.percentiles = p; self }
    pub fn tenors(mut self, t: Vec<f64>) -> Self { self.tenors = t; self }
    pub fn cone_data(mut self, d: Vec<Vec<f64>>) -> Self { self.cone_data = d; self }
    pub fn current_vol(mut self, v: f64) -> Self { self.current_vol = v.max(0.0); self }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    cfg: &VdatConeConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    if cfg.cone_data.is_empty() || cfg.cone_data[0].is_empty() {
        return Err(BtError::EmptySeries("cone data".into()));
    }
    fill_background(&root, cfg.theme)?;

    let mut v_min = f64::INFINITY;
    let mut v_max = f64::NEG_INFINITY;
    for row in &cfg.cone_data {
        for &v in row {
            v_min = v_min.min(v);
            v_max = v_max.max(v);
        }
    }
    v_min = v_min.min(cfg.current_vol);
    v_max = v_max.max(cfg.current_vol);
    let pad = (v_max - v_min).max(1.0) * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — Current: {:.1}%", cfg.title, cfg.current_vol),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..cfg.tenors.len() as f64, (v_min - pad)..(v_max + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .x_desc("Tenor (years)")
        .y_desc("Volatility (%)")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let colors = [
        cfg.theme.loss(),
        cfg.theme.accent(),
        cfg.theme.info(),
        cfg.theme.profit(),
        RGBColor(0xB0, 0x66, 0xFF),
    ];

    for (pi, row) in cfg.cone_data.iter().enumerate() {
        let color = colors[pi % colors.len()];
        let points: Vec<(f64, f64)> = row
            .iter()
            .enumerate()
            .map(|(ti, &v)| (ti as f64, v))
            .collect();

        chart
            .draw_series(LineSeries::new(points, color.stroke_width(2)))
            .map_err(|e| BtError::Render(e.to_string()))?
            .label(format!("P{:.0}", cfg.percentiles.get(pi).unwrap_or(&0.0) * 100.0))
            .legend(move |(x, y)| {
                PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2))
            });
    }

    chart
        .draw_series(LineSeries::new(
            vec![(0.0, cfg.current_vol), (cfg.tenors.len() as f64, cfg.current_vol)],
            cfg.theme.text().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Current")
        .legend(|(x, y)| {
            PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.text().stroke_width(2))
        });

    chart
        .configure_series_labels()
        .background_style(cfg.theme.background().mix(0.9))
        .border_style(cfg.theme.border())
        .label_font((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(cfg: &VdatConeConfig, path: &str) -> Result<()> {
    render(png_root(path)?, cfg)
}

pub fn render_svg(cfg: &VdatConeConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let cone_data: Vec<Vec<f64>> = (0..5)
            .map(|p| (0..6).map(|t| 15.0 + p as f64 * 3.0 + t as f64 * 2.0).collect())
            .collect();
        let cfg = VdatConeConfig::new().theme(Theme::Dark).cone_data(cone_data);
        let path = std::env::temp_dir().join("bt_test_vdat_cone.png").to_str().unwrap().to_string();
        render_png(&cfg, &path).unwrap();
    }
}
