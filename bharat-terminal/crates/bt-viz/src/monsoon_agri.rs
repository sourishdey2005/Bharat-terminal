// crates/bt-viz/src/monsoon_agri.rs
// Author: Sourish Dey

//! Monsoon rainfall vs agricultural commodity prices. Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

/// Monthly monsoon and agri commodity data.
#[derive(Debug, Clone)]
pub struct MonsoonAgriPoint {
    pub month: String,
    pub rainfall_mm: f64,        // actual rainfall
    pub normal_rainfall_mm: f64, // long-period average
    pub agri_index: f64,         // agricultural commodity price index
}

/// Sample data for Jun-Sep monsoon season.
fn sample_monsoon() -> Vec<MonsoonAgriPoint> {
    vec![
        MonsoonAgriPoint {
            month: "Jun".to_string(),
            rainfall_mm: 120.0,
            normal_rainfall_mm: 145.0,
            agri_index: 100.0,
        },
        MonsoonAgriPoint {
            month: "Jul".to_string(),
            rainfall_mm: 280.0,
            normal_rainfall_mm: 290.0,
            agri_index: 103.5,
        },
        MonsoonAgriPoint {
            month: "Aug".to_string(),
            rainfall_mm: 240.0,
            normal_rainfall_mm: 260.0,
            agri_index: 107.2,
        },
        MonsoonAgriPoint {
            month: "Sep".to_string(),
            rainfall_mm: 180.0,
            normal_rainfall_mm: 175.0,
            agri_index: 110.8,
        },
        MonsoonAgriPoint {
            month: "Oct".to_string(),
            rainfall_mm: 60.0,
            normal_rainfall_mm: 70.0,
            agri_index: 112.3,
        },
        MonsoonAgriPoint {
            month: "Nov".to_string(),
            rainfall_mm: 20.0,
            normal_rainfall_mm: 25.0,
            agri_index: 111.5,
        },
    ]
}

#[derive(Debug, Clone)]
pub struct MonsoonAgriConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for MonsoonAgriConfig {
    fn default() -> Self {
        Self {
            title: "Monsoon vs Agri Commodities".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl MonsoonAgriConfig {
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
    data: &[MonsoonAgriPoint],
    cfg: &MonsoonAgriConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if data.is_empty() {
        return Err(BtError::EmptySeries("monsoon data".into()));
    }
    fill_background(&root, cfg.theme)?;

    let max_rain = data
        .iter()
        .map(|p| p.rainfall_mm.max(p.normal_rainfall_mm))
        .fold(0.0_f64, f64::max)
        * 1.15;
    let idx_lo = data
        .iter()
        .map(|p| p.agri_index)
        .fold(f64::INFINITY, f64::min)
        * 0.98;
    let idx_hi = data
        .iter()
        .map(|p| p.agri_index)
        .fold(f64::NEG_INFINITY, f64::max)
        * 1.02;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            &cfg.title,
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(15)
        .x_label_area_size(35)
        .y_label_area_size(55)
        .right_y_label_area_size(55)
        .build_cartesian_2d(0f64..data.len() as f64, 0f64..max_rain)
        .map_err(|e| BtError::Render(e.to_string()))?
        .set_secondary_coord(0f64..data.len() as f64, idx_lo..idx_hi);

    chart
        .configure_mesh()
        .x_labels(data.len())
        .x_label_formatter(&|idx| {
            data.get(*idx as usize)
                .map(|p| p.month.clone())
                .unwrap_or_default()
        })
        .y_desc("Rainfall (mm)")
        .label_style((LABEL_FONT, 11).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_secondary_axes()
        .y_desc("Agri Index")
        .label_style((LABEL_FONT, 11).into_font().color(&cfg.theme.text()))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Rainfall bars
    let bar_w = 0.3;
    chart
        .draw_series(data.iter().enumerate().map(|(i, p)| {
            let x = i as f64;
            let color = if p.rainfall_mm >= p.normal_rainfall_mm {
                cfg.theme.info().mix(0.7).filled()
            } else {
                cfg.theme.loss().mix(0.5).filled()
            };
            Rectangle::new([(x - bar_w, 0.0), (x, p.rainfall_mm)], color)
        }))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Rainfall")
        .legend(|(x, y)| Rectangle::new([(x, y), (x + 10, y + 10)], cfg.theme.info().filled()));

    // Normal rainfall line
    let normal_pts: Vec<(f64, f64)> = data
        .iter()
        .enumerate()
        .map(|(i, p)| (i as f64, p.normal_rainfall_mm))
        .collect();
    chart
        .draw_series(LineSeries::new(
            normal_pts,
            cfg.theme.text().mix(0.6).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Normal")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 15, y)],
                cfg.theme.text().mix(0.6).stroke_width(1),
            )
        });

    // Agri index on secondary axis
    let idx_pts: Vec<(f64, f64)> = data
        .iter()
        .enumerate()
        .map(|(i, p)| (i as f64, p.agri_index))
        .collect();
    chart
        .draw_secondary_series(LineSeries::new(idx_pts, cfg.theme.accent().stroke_width(2)))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Agri Index")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 15, y)],
                cfg.theme.accent().stroke_width(2),
            )
        });

    chart
        .configure_series_labels()
        .background_style(cfg.theme.background())
        .label_font((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(data: &[MonsoonAgriPoint], cfg: &MonsoonAgriConfig, path: &str) -> Result<()> {
    render(png_root(path)?, data, cfg)
}

pub fn render_svg(data: &[MonsoonAgriPoint], cfg: &MonsoonAgriConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, data, cfg)
}

pub fn render_sample_png(cfg: &MonsoonAgriConfig, path: &str) -> Result<()> {
    let data = sample_monsoon();
    render_png(&data, cfg, path)
}

pub fn render_sample_svg(cfg: &MonsoonAgriConfig, path: &str) -> Result<()> {
    let data = sample_monsoon();
    render_svg(&data, cfg, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let data = sample_monsoon();
        let cfg = MonsoonAgriConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_monsoon.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&data, &cfg, &path).unwrap();
    }
}
