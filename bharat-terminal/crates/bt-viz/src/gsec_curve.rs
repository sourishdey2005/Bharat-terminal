// crates/bt-viz/src/gsec_curve.rs
// Author: Sourish Dey

//! Government securities (G-Sec) yield curve. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct GsecCurveConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for GsecCurveConfig {
    fn default() -> Self {
        Self {
            title: "G-Sec Yield Curve".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl GsecCurveConfig {
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

fn sample_gsec() -> Vec<(String, f64)> {
    vec![
        ("91D T-Bill".to_string(), 6.45),
        ("182D T-Bill".to_string(), 6.62),
        ("364D T-Bill".to_string(), 6.71),
        ("2Y G-Sec".to_string(), 6.78),
        ("3Y G-Sec".to_string(), 6.82),
        ("5Y G-Sec".to_string(), 6.91),
        ("7Y G-Sec".to_string(), 6.98),
        ("10Y G-Sec".to_string(), 7.05),
        ("15Y G-Sec".to_string(), 7.12),
        ("20Y G-Sec".to_string(), 7.18),
        ("30Y G-Sec".to_string(), 7.24),
        ("40Y G-Sec".to_string(), 7.28),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &GsecCurveConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_gsec();
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
        .build_cartesian_2d(0f64..data.len() as f64, 6.0f64..8.0f64)
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
        .y_desc("Yield (%)")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let points: Vec<(f64, f64)> = data
        .iter()
        .enumerate()
        .map(|(i, (_, y))| (i as f64, *y))
        .collect();
    chart
        .draw_series(LineSeries::new(
            points.clone(),
            cfg.theme.accent().stroke_width(3),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    chart
        .draw_series(
            points
                .iter()
                .map(|&(x, y)| Circle::new((x, y), 5, cfg.theme.profit().filled())),
        )
        .map_err(|e| BtError::Render(e.to_string()))?;

    for (i, (label, y)) in data.iter().enumerate() {
        root.draw(&Text::new(
            format!("{:.2}", y),
            (60 + i as i32 * ((w as i32 - 120) / data.len() as i32), 50),
            (LABEL_FONT, 10).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        let _ = label;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &GsecCurveConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &GsecCurveConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("GSEC", 100, 1, 100.0);
        let cfg = GsecCurveConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_gsec_curve.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
