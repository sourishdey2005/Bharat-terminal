// crates/bt-viz/src/gst_budget.rs
// Author: Sourish Dey

//! GST and Union Budget analytics. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct GstBudgetConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for GstBudgetConfig {
    fn default() -> Self {
        Self { title: "GST & Union Budget".to_string(), theme: Theme::Dark }
    }
}

impl GstBudgetConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
}

fn sample_gst() -> Vec<(String, f64, f64)> {
    vec![
        ("Apr".to_string(), 1.82, 1.65),
        ("May".to_string(), 1.75, 1.68),
        ("Jun".to_string(), 1.88, 1.72),
        ("Jul".to_string(), 1.95, 1.78),
        ("Aug".to_string(), 1.85, 1.80),
        ("Sep".to_string(), 1.92, 1.82),
        ("Oct".to_string(), 2.05, 1.88),
        ("Nov".to_string(), 1.98, 1.90),
        ("Dec".to_string(), 2.10, 1.95),
        ("Jan".to_string(), 2.15, 2.00),
        ("Feb".to_string(), 2.08, 2.05),
        ("Mar".to_string(), 2.20, 2.10),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &GstBudgetConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_gst();
    let (w, h) = root.dim_in_pixel();

    root.draw(&Text::new(
        format!("{} — {} (INR L Cr)", cfg.title, series.symbol),
        (10, 10),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let chart = ChartBuilder::on(&root)
        .margin(15)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..data.len() as f64, 1.0f64..2.5f64)
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
        .y_desc("GST Collection (L Cr)")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let current: Vec<(f64, f64)> = data.iter().enumerate().map(|(i, (_, c, _))| (i as f64, *c)).collect();
    let previous: Vec<(f64, f64)> = data.iter().enumerate().map(|(i, (_, _, p))| (i as f64, *p)).collect();

    chart.draw_series(LineSeries::new(previous, cfg.theme.text().mix(0.5).stroke_width(2)))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Previous Year")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.text().mix(0.5).stroke_width(2)));

    chart.draw_series(LineSeries::new(current, cfg.theme.accent().stroke_width(3)))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Current Year")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.accent().stroke_width(3)));

    chart.configure_series_labels()
        .border_style(&cfg.theme.border())
        .label_font((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let _ = (w, h);
    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &GstBudgetConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &GstBudgetConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("GST", 100, 1, 100.0);
        let cfg = GstBudgetConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_gst_budget.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
