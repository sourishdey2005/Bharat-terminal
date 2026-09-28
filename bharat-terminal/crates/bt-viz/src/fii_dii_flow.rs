// crates/bt-viz/src/fii_dii_flow.rs
// Author: Sourish Dey

//! FII/DII flow chart. Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

/// Daily FII/DII flow data point.
#[derive(Debug, Clone)]
pub struct FlowPoint {
    pub date: String,
    pub fii: f64,  // crores, +ve = buying
    pub dii: f64,  // crores, +ve = buying
}

/// Sample FII/DII flow data (30 days).
fn sample_flow() -> Vec<FlowPoint> {
    let dates = [
        "01 Sep", "02 Sep", "03 Sep", "04 Sep", "05 Sep",
        "08 Sep", "09 Sep", "10 Sep", "11 Sep", "12 Sep",
        "15 Sep", "16 Sep", "17 Sep", "18 Sep", "19 Sep",
        "22 Sep", "23 Sep", "24 Sep", "25 Sep", "26 Sep",
        "29 Sep", "30 Sep", "01 Oct", "02 Oct", "03 Oct",
        "06 Oct", "07 Oct", "08 Oct", "09 Oct", "10 Oct",
    ];
    let fii_vals = [
        -1250.0, -890.0, 450.0, 1200.0, -340.0,
        -2100.0, -1500.0, 800.0, 1600.0, 950.0,
        -670.0, -1100.0, 300.0, 780.0, 1450.0,
        -450.0, -920.0, 1100.0, 1800.0, 650.0,
        -1300.0, -780.0, 500.0, 900.0, 1300.0,
        -550.0, -1000.0, 700.0, 1150.0, 850.0,
    ];
    let dii_vals = [
        800.0, 650.0, -200.0, -500.0, 300.0,
        1200.0, 900.0, -400.0, -700.0, -350.0,
        500.0, 750.0, -150.0, -400.0, -600.0,
        350.0, 600.0, -500.0, -800.0, -300.0,
        850.0, 500.0, -250.0, -450.0, -600.0,
        400.0, 650.0, -350.0, -550.0, -400.0,
    ];
    dates
        .iter()
        .enumerate()
        .map(|(i, d)| FlowPoint {
            date: d.to_string(),
            fii: fii_vals[i],
            dii: dii_vals[i],
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct FiiDiiFlowConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for FiiDiiFlowConfig {
    fn default() -> Self {
        Self {
            title: "FII / DII Flow".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl FiiDiiFlowConfig {
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
    data: &[FlowPoint],
    cfg: &FiiDiiFlowConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if data.is_empty() {
        return Err(BtError::EmptySeries("flow data".into()));
    }
    fill_background(&root, cfg.theme)?;

    let max_abs = data
        .iter()
        .flat_map(|p| [p.fii.abs(), p.dii.abs()])
        .fold(0.0_f64, f64::max)
        * 1.15;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            &cfg.title,
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(15)
        .x_label_area_size(35)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..data.len() as f64, -max_abs..max_abs)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .x_labels(10)
        .x_label_formatter(&|idx| {
            data.get(*idx as usize)
                .map(|p| p.date.clone())
                .unwrap_or_default()
        })
        .y_desc("Crores (INR)")
        .label_style((LABEL_FONT, 11).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Zero line
    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(0.0, 0.0), (data.len() as f64, 0.0)],
            cfg.theme.border().stroke_width(1),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    // FII bars
    let bar_w = 0.35;
    chart
        .draw_series(data.iter().enumerate().map(|(i, p)| {
            let x = i as f64;
            let color = if p.fii >= 0.0 {
                cfg.theme.profit().mix(0.7).filled()
            } else {
                cfg.theme.loss().mix(0.7).filled()
            };
            Rectangle::new(
                [(x - bar_w, 0.0), (x, p.fii)],
                color,
            )
        }))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("FII")
        .legend(|(x, y)| {
            Rectangle::new([(x, y), (x + 10, y + 10)], cfg.theme.profit().filled())
        });

    // DII bars
    chart
        .draw_series(data.iter().enumerate().map(|(i, p)| {
            let x = i as f64;
            let color = if p.dii >= 0.0 {
                cfg.theme.info().mix(0.7).filled()
            } else {
                cfg.theme.accent().mix(0.7).filled()
            };
            Rectangle::new(
                [(x, 0.0), (x + bar_w, p.dii)],
                color,
            )
        }))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("DII")
        .legend(|(x, y)| {
            Rectangle::new([(x, y), (x + 10, y + 10)], cfg.theme.info().filled())
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

pub fn render_png(data: &[FlowPoint], cfg: &FiiDiiFlowConfig, path: &str) -> Result<()> {
    render(png_root(path)?, data, cfg)
}

pub fn render_svg(data: &[FlowPoint], cfg: &FiiDiiFlowConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, data, cfg)
}

pub fn render_sample_png(cfg: &FiiDiiFlowConfig, path: &str) -> Result<()> {
    let data = sample_flow();
    render_png(&data, cfg, path)
}

pub fn render_sample_svg(cfg: &FiiDiiFlowConfig, path: &str) -> Result<()> {
    let data = sample_flow();
    render_svg(&data, cfg, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let data = sample_flow();
        let cfg = FiiDiiFlowConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_fii_dii.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&data, &cfg, &path).unwrap();
    }
}
