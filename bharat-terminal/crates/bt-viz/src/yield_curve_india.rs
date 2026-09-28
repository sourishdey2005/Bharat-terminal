// crates/bt-viz/src/yield_curve_india.rs
// Author: Sourish Dey

//! Indian yield curve (sovereign, SDL, corporate). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct YieldCurveIndiaConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for YieldCurveIndiaConfig {
    fn default() -> Self {
        Self {
            title: "India Yield Curve".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl YieldCurveIndiaConfig {
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

fn sample_yields() -> Vec<(String, f64, f64, f64)> {
    vec![
        ("1Y".to_string(), 6.75, 6.85, 7.35),
        ("2Y".to_string(), 6.78, 6.92, 7.45),
        ("3Y".to_string(), 6.82, 6.98, 7.55),
        ("5Y".to_string(), 6.91, 7.08, 7.68),
        ("7Y".to_string(), 6.98, 7.15, 7.75),
        ("10Y".to_string(), 7.05, 7.22, 7.85),
        ("15Y".to_string(), 7.12, 7.30, 7.95),
        ("20Y".to_string(), 7.18, 7.36, 8.02),
        ("30Y".to_string(), 7.24, 7.42, 8.10),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &YieldCurveIndiaConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_yields();
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
        .build_cartesian_2d(0f64..data.len() as f64, 6.5f64..8.5f64)
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

    let series_data = [
        ("Sovereign", cfg.theme.profit(), 1),
        ("SDL", cfg.theme.accent(), 2),
        ("Corporate", cfg.theme.info(), 3),
    ];

    for (name, color, idx) in series_data {
        let points: Vec<(f64, f64)> = data
            .iter()
            .enumerate()
            .map(|(i, row)| {
                let v = match idx {
                    1 => row.1,
                    2 => row.2,
                    _ => row.3,
                };
                (i as f64, v)
            })
            .collect();
        chart
            .draw_series(LineSeries::new(points, color.stroke_width(3)))
            .map_err(|e| BtError::Render(e.to_string()))?
            .label(name)
            .legend(move |(x, y)| {
                PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(3))
            });
    }

    chart
        .configure_series_labels()
        .border_style(&cfg.theme.border())
        .label_font((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let _ = (w, h);
    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &YieldCurveIndiaConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &YieldCurveIndiaConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("YIELD", 100, 1, 100.0);
        let cfg = YieldCurveIndiaConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_yield_curve_india.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
