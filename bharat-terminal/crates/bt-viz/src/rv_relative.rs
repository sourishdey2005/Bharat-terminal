// crates/bt-viz/src/rv_relative.rs
// Author: Sourish Dey

//! RV: Relative valuation scatter.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct RvPoint {
    pub symbol: String,
    pub x_value: f64,
    pub y_value: f64,
    pub size: f64,
}

impl RvPoint {
    pub fn new(symbol: impl Into<String>, x_value: f64, y_value: f64, size: f64) -> Self {
        Self {
            symbol: symbol.into(),
            x_value,
            y_value,
            size: size.max(1.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RvRelativeConfig {
    pub title: String,
    pub theme: Theme,
    pub x_label: String,
    pub y_label: String,
    pub show_regression: bool,
}

impl Default for RvRelativeConfig {
    fn default() -> Self {
        Self {
            title: "Relative Valuation Scatter".to_string(),
            theme: Theme::Dark,
            x_label: "P/E".into(),
            y_label: "ROE (%)".into(),
            show_regression: true,
        }
    }
}

impl RvRelativeConfig {
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
    pub fn x_label(mut self, l: impl Into<String>) -> Self {
        self.x_label = l.into();
        self
    }
    pub fn y_label(mut self, l: impl Into<String>) -> Self {
        self.y_label = l.into();
        self
    }
    pub fn show_regression(mut self, s: bool) -> Self {
        self.show_regression = s;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    points: &[RvPoint],
    cfg: &RvRelativeConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if points.is_empty() {
        return Err(BtError::EmptySeries("scatter points".into()));
    }
    fill_background(&root, cfg.theme)?;

    let x_min = points
        .iter()
        .map(|p| p.x_value)
        .fold(f64::INFINITY, f64::min);
    let x_max = points
        .iter()
        .map(|p| p.x_value)
        .fold(f64::NEG_INFINITY, f64::max);
    let y_min = points
        .iter()
        .map(|p| p.y_value)
        .fold(f64::INFINITY, f64::min);
    let y_max = points
        .iter()
        .map(|p| p.y_value)
        .fold(f64::NEG_INFINITY, f64::max);
    let x_pad = (x_max - x_min).max(1.0) * 0.1;
    let y_pad = (y_max - y_min).max(1.0) * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            cfg.title.clone(),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(
            (x_min - x_pad)..(x_max + x_pad),
            (y_min - y_pad)..(y_max + y_pad),
        )
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .x_desc(&cfg.x_label)
        .y_desc(&cfg.y_label)
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(points.iter().map(|p| {
            let radius = (p.size.sqrt() / 10.0).max(3.0) as i32;
            Circle::new(
                (p.x_value, p.y_value),
                radius,
                cfg.theme.info().mix(0.6).filled(),
            )
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    for p in points {
        chart
            .draw_series(std::iter::once(Text::new(
                p.symbol.clone(),
                (p.x_value, p.y_value + y_pad * 0.1),
                (LABEL_FONT, 10).into_font().color(&cfg.theme.text()),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    if cfg.show_regression && points.len() >= 2 {
        let n = points.len() as f64;
        let sum_x: f64 = points.iter().map(|p| p.x_value).sum();
        let sum_y: f64 = points.iter().map(|p| p.y_value).sum();
        let sum_xy: f64 = points.iter().map(|p| p.x_value * p.y_value).sum();
        let sum_x2: f64 = points.iter().map(|p| p.x_value * p.x_value).sum();

        let denom = n * sum_x2 - sum_x * sum_x;
        if denom.abs() > 1e-10 {
            let slope = (n * sum_xy - sum_x * sum_y) / denom;
            let intercept = (sum_y - slope * sum_x) / n;

            let x1 = x_min - x_pad;
            let x2 = x_max + x_pad;
            let y1 = slope * x1 + intercept;
            let y2 = slope * x2 + intercept;

            chart
                .draw_series(LineSeries::new(
                    vec![(x1, y1), (x2, y2)],
                    cfg.theme.accent().stroke_width(2),
                ))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(points: &[RvPoint], cfg: &RvRelativeConfig, path: &str) -> Result<()> {
    render(png_root(path)?, points, cfg)
}

pub fn render_svg(points: &[RvPoint], cfg: &RvRelativeConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, points, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let points: Vec<RvPoint> = (0..10)
            .map(|i| {
                RvPoint::new(
                    format!("S{}", i),
                    10.0 + i as f64 * 2.0,
                    5.0 + i as f64 * 1.5,
                    (i + 1) as f64 * 100.0,
                )
            })
            .collect();
        let cfg = RvRelativeConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_rv_relative.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&points, &cfg, &path).unwrap();
    }
}
