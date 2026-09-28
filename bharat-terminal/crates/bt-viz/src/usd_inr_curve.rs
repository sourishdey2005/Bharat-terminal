// crates/bt-viz/src/usd_inr_curve.rs
// Author: Sourish Dey

//! USD/INR forward premium curve. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct UsdInrCurveConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for UsdInrCurveConfig {
    fn default() -> Self {
        Self { title: "USD/INR Forward Curve".to_string(), theme: Theme::Dark }
    }
}

impl UsdInrCurveConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
}

fn sample_forwards() -> Vec<(String, f64, f64)> {
    vec![
        ("Spot".to_string(), 83.25, 0.0),
        ("1W".to_string(), 83.32, 0.08),
        ("1M".to_string(), 83.48, 0.28),
        ("2M".to_string(), 83.65, 0.47),
        ("3M".to_string(), 83.82, 0.67),
        ("6M".to_string(), 84.15, 1.08),
        ("1Y".to_string(), 84.62, 1.64),
        ("2Y".to_string(), 85.35, 2.52),
        ("3Y".to_string(), 86.05, 3.37),
        ("5Y".to_string(), 87.25, 4.80),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &UsdInrCurveConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_forwards();
    let (w, h) = root.dim_in_pixel();

    root.draw(&Text::new(
        format!("{} — {}", cfg.title, series.symbol),
        (10, 10),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let chart = ChartBuilder::on(&root)
        .margin(15)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0f64..data.len() as f64, 82.5f64..88.0f64)
        .map_err(|e| BtError::Render(e.to_string()))?;

    let mut chart = chart;
    chart.configure_mesh()
        .label_style((LABEL_FONT, 11).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .x_labels(data.len())
        .x_label_formatter(&|x| {
            let idx = *x as usize;
            if idx < data.len() { data[idx].0.clone() } else { String::new() }
        })
        .y_desc("INR per USD")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let points: Vec<(f64, f64)> = data.iter().enumerate().map(|(i, (_, px, _))| (i as f64, *px)).collect();
    chart.draw_series(LineSeries::new(points.clone(), cfg.theme.accent().stroke_width(3)))
        .map_err(|e| BtError::Render(e.to_string()))?;
    chart.draw_series(points.iter().map(|&(x, y)| {
        Circle::new((x, y), 5, cfg.theme.profit().filled())
    }))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let prem: Vec<(f64, f64)> = data.iter().enumerate().map(|(i, (_, _, p))| (i as f64, *p)).collect();
    chart.draw_series(LineSeries::new(prem, cfg.theme.info().stroke_width(2)))
        .map_err(|e| BtError::Render(e.to_string()))?;

    for (i, (label, px, prem)) in data.iter().enumerate() {
        root.draw(&Text::new(
            format!("{:.2}", px),
            (60 + i as i32 * ((w as i32 - 120) / data.len() as i32), 50),
            (LABEL_FONT, 10).into_font().color(&cfg.theme.text()),
        ))
    .map_err(|e| BtError::Render(e.to_string()))?;
        let _ = (label, prem);
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &UsdInrCurveConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &UsdInrCurveConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("USDINR", 100, 1, 100.0);
        let cfg = UsdInrCurveConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_usd_inr_curve.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
