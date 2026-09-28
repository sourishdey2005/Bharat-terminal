// crates/bt-viz/src/gc_curves.rs
// Author: Sourish Dey

//! GC: Multi-tenor yield curves.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct YieldCurve {
    pub name: String,
    pub tenors: Vec<f64>,
    pub yields: Vec<f64>,
}

impl YieldCurve {
    pub fn new(name: impl Into<String>, tenors: Vec<f64>, yields: Vec<f64>) -> Self {
        Self {
            name: name.into(),
            tenors,
            yields,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GcCurvesConfig {
    pub title: String,
    pub theme: Theme,
    pub show_labels: bool,
}

impl Default for GcCurvesConfig {
    fn default() -> Self {
        Self {
            title: "Multi-Tenor Yield Curves".to_string(),
            theme: Theme::Dark,
            show_labels: true,
        }
    }
}

impl GcCurvesConfig {
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
    pub fn show_labels(mut self, s: bool) -> Self {
        self.show_labels = s;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    curves: &[YieldCurve],
    cfg: &GcCurvesConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if curves.is_empty() {
        return Err(BtError::EmptySeries("yield curves".into()));
    }
    fill_background(&root, cfg.theme)?;

    let colors = [
        cfg.theme.profit(),
        cfg.theme.info(),
        cfg.theme.accent(),
        cfg.theme.loss(),
        RGBColor(0xB0, 0x66, 0xFF),
        RGBColor(0xFF, 0x8A, 0x00),
    ];

    let mut t_min = f64::INFINITY;
    let mut t_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;

    for curve in curves {
        for &t in &curve.tenors {
            t_min = t_min.min(t);
            t_max = t_max.max(t);
        }
        for &y in &curve.yields {
            y_min = y_min.min(y);
            y_max = y_max.max(y);
        }
    }

    let t_range = (t_max - t_min).max(0.1);
    let y_range = (y_max - y_min).max(0.1);
    t_min -= t_range * 0.05;
    t_max += t_range * 0.05;
    y_min -= y_range * 0.1;
    y_max += y_range * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            cfg.title.clone(),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, y_min..y_max)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .x_desc("Tenor (years)")
        .y_desc("Yield (%)")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    for (idx, curve) in curves.iter().enumerate() {
        let color = colors[idx % colors.len()];
        let points: Vec<(f64, f64)> = curve
            .tenors
            .iter()
            .zip(curve.yields.iter())
            .map(|(&t, &y)| (t, y))
            .collect();

        chart
            .draw_series(LineSeries::new(points.clone(), color.stroke_width(2)))
            .map_err(|e| BtError::Render(e.to_string()))?
            .label(&curve.name)
            .legend(move |(x, y)| {
                PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2))
            });

        if cfg.show_labels {
            chart
                .draw_series(
                    points
                        .iter()
                        .map(|&(t, y)| Circle::new((t, y), 3, color.filled())),
                )
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

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

pub fn render_png(curves: &[YieldCurve], cfg: &GcCurvesConfig, path: &str) -> Result<()> {
    render(png_root(path)?, curves, cfg)
}

pub fn render_svg(curves: &[YieldCurve], cfg: &GcCurvesConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, curves, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let curves = vec![
            YieldCurve::new(
                "Curve A",
                vec![0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0],
                vec![3.5, 3.7, 4.0, 4.3, 4.8, 5.2, 5.5],
            ),
            YieldCurve::new(
                "Curve B",
                vec![0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0],
                vec![3.2, 3.4, 3.8, 4.1, 4.5, 4.9, 5.1],
            ),
        ];
        let cfg = GcCurvesConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_gc_curves.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&curves, &cfg, &path).unwrap();
    }
}
