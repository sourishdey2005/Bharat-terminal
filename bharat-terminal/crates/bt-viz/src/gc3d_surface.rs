// crates/bt-viz/src/gc3d_surface.rs
// Author: Sourish Dey

//! GC3D: 3D yield curve surface.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct Gc3dSurfaceConfig {
    pub title: String,
    pub theme: Theme,
    pub tenors: Vec<f64>,
    pub dates: Vec<f64>,
    pub surface: Vec<Vec<f64>>,
}

impl Default for Gc3dSurfaceConfig {
    fn default() -> Self {
        Self {
            title: "3D Yield Curve Surface".to_string(),
            theme: Theme::Dark,
            tenors: vec![0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0],
            dates: (0..12).map(|i| i as f64).collect(),
            surface: vec![vec![4.0; 7]; 12],
        }
    }
}

impl Gc3dSurfaceConfig {
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
    pub fn tenors(mut self, t: Vec<f64>) -> Self {
        self.tenors = t;
        self
    }
    pub fn dates(mut self, d: Vec<f64>) -> Self {
        self.dates = d;
        self
    }
    pub fn surface(mut self, s: Vec<Vec<f64>>) -> Self {
        self.surface = s;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    cfg: &Gc3dSurfaceConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if cfg.surface.is_empty() || cfg.surface[0].is_empty() {
        return Err(BtError::EmptySeries("surface data".into()));
    }
    fill_background(&root, cfg.theme)?;

    let mut z_min = f64::INFINITY;
    let mut z_max = f64::NEG_INFINITY;
    for row in &cfg.surface {
        for &v in row {
            z_min = z_min.min(v);
            z_max = z_max.max(v);
        }
    }
    let z_range = (z_max - z_min).max(0.1);

    let mut chart = ChartBuilder::on(&root)
        .caption(
            cfg.title.clone(),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_3d(
            0.0..cfg.dates.len() as f64,
            z_min..z_max,
            0.0..cfg.tenors.len() as f64,
        )
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_axes()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(cfg.surface.iter().enumerate().flat_map(|(di, row)| {
            row.iter().enumerate().map(move |(ti, &z)| {
                let color_val = (z - z_min) / z_range;
                let color = if color_val < 0.5 {
                    let t = color_val * 2.0;
                    let lerp = |a: u8, b: u8| (a as f64 + (b as f64 - a as f64) * t).round() as u8;
                    RGBColor(lerp(0x00, 0xFF), lerp(0xBF, 0x3B), lerp(0xFF, 0x3B))
                } else {
                    let t = (color_val - 0.5) * 2.0;
                    let lerp = |a: u8, b: u8| (a as f64 + (b as f64 - a as f64) * t).round() as u8;
                    RGBColor(lerp(0xFF, 0x00), lerp(0x3B, 0xFF), lerp(0x3B, 0x88))
                };
                Rectangle::new(
                    [
                        (di as f64, z, ti as f64),
                        ((di + 1) as f64, z, (ti + 1) as f64),
                    ],
                    color.filled(),
                )
            })
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(cfg: &Gc3dSurfaceConfig, path: &str) -> Result<()> {
    render(png_root(path)?, cfg)
}

pub fn render_svg(cfg: &Gc3dSurfaceConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let surface: Vec<Vec<f64>> = (0..12)
            .map(|d| {
                (0..7)
                    .map(|t| 3.0 + d as f64 * 0.1 + t as f64 * 0.3)
                    .collect()
            })
            .collect();
        let cfg = Gc3dSurfaceConfig::new().theme(Theme::Dark).surface(surface);
        let path = std::env::temp_dir()
            .join("bt_test_gc3d_surface.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&cfg, &path).unwrap();
    }
}
