// crates/bt-viz/src/fpi_fii_flow.rs
// Author: Sourish Dey

//! FPI/FII flow chart (daily/weekly flows into Indian markets). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct FpiFiiFlowConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for FpiFiiFlowConfig {
    fn default() -> Self {
        Self {
            title: "FPI/FII Flow".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl FpiFiiFlowConfig {
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

fn sample_flows() -> Vec<(String, f64, f64)> {
    vec![
        ("Mon".to_string(), 1250.0, -850.0),
        ("Tue".to_string(), -420.0, 680.0),
        ("Wed".to_string(), 890.0, -120.0),
        ("Thu".to_string(), -150.0, 450.0),
        ("Fri".to_string(), 2100.0, 320.0),
        ("Sat".to_string(), 0.0, 0.0),
        ("Sun".to_string(), 0.0, 0.0),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &FpiFiiFlowConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_flows();
    let (w, h) = root.dim_in_pixel();

    root.draw(&Text::new(
        format!("{} — {} (INR Cr)", cfg.title, series.symbol),
        (10, 10),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let chart = ChartBuilder::on(&root)
        .margin(15)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..data.len() as f64, -1500f64..2500f64)
        .map_err(|e| BtError::Render(e.to_string()))?;

    let mut chart = chart;
    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 11).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .x_labels(data.len())
        .x_label_formatter(&|x| {
            let idx = *x as usize;
            if idx < data.len() {
                data[idx].0.clone()
            } else {
                String::new()
            }
        })
        .y_desc("Flow (INR Cr)")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let bar_w = 0.35;
    for (i, (_, fpi, fii)) in data.iter().enumerate() {
        let fpi_color = if *fpi >= 0.0 {
            cfg.theme.profit()
        } else {
            cfg.theme.loss()
        };
        let fii_color = if *fii >= 0.0 {
            cfg.theme.info()
        } else {
            cfg.theme.accent()
        };
        chart
            .draw_series(std::iter::once(Rectangle::new(
                [(i as f64 - bar_w, 0.0f64), (i as f64, *fpi)],
                fpi_color.filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
        chart
            .draw_series(std::iter::once(Rectangle::new(
                [(i as f64, 0.0f64), (i as f64 + bar_w, *fii)],
                fii_color.filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    chart
        .draw_series(LineSeries::new(
            vec![(0.0, 0.0), (data.len() as f64, 0.0)],
            cfg.theme.text().stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    let _ = (w, h);
    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &FpiFiiFlowConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &FpiFiiFlowConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("FPI", 100, 1, 100.0);
        let cfg = FpiFiiFlowConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_fpi_fii_flow.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
