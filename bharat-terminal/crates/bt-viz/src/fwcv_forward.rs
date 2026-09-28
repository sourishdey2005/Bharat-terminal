// crates/bt-viz/src/fwcv_forward.rs
// Author: Sourish Dey

//! FWCV: Forward rate curves.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct ForwardCurve {
    pub name: String,
    pub start_times: Vec<f64>,
    pub forward_rates: Vec<f64>,
}

impl ForwardCurve {
    pub fn new(name: impl Into<String>, start_times: Vec<f64>, forward_rates: Vec<f64>) -> Self {
        Self { name: name.into(), start_times, forward_rates }
    }
}

#[derive(Debug, Clone)]
pub struct FwcvForwardConfig {
    pub title: String,
    pub theme: Theme,
    pub show_zero_line: bool,
}

impl Default for FwcvForwardConfig {
    fn default() -> Self {
        Self {
            title: "Forward Rate Curves".to_string(),
            theme: Theme::Dark,
            show_zero_line: true,
        }
    }
}

impl FwcvForwardConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn show_zero_line(mut self, s: bool) -> Self { self.show_zero_line = s; self }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    curves: &[ForwardCurve],
    cfg: &FwcvForwardConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    if curves.is_empty() {
        return Err(BtError::EmptySeries("forward curves".into()));
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
    let mut r_min = f64::INFINITY;
    let mut r_max = f64::NEG_INFINITY;

    for curve in curves {
        for &t in &curve.start_times {
            t_min = t_min.min(t);
            t_max = t_max.max(t);
        }
        for &r in &curve.forward_rates {
            r_min = r_min.min(r);
            r_max = r_max.max(r);
        }
    }

    let t_range = (t_max - t_min).max(0.1);
    let r_range = (r_max - r_min).max(0.1);
    t_min -= t_range * 0.05;
    t_max += t_range * 0.05;
    r_min -= r_range * 0.1;
    r_max += r_range * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            cfg.title.clone(),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, r_min..r_max)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .x_desc("Start Time (years)")
        .y_desc("Forward Rate (%)")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    if cfg.show_zero_line {
        chart
            .draw_series(LineSeries::new(
                vec![(t_min, 0.0), (t_max, 0.0)],
                cfg.theme.border().mix(0.5).stroke_width(1),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    for (idx, curve) in curves.iter().enumerate() {
        let color = colors[idx % colors.len()];
        let points: Vec<(f64, f64)> = curve
            .start_times
            .iter()
            .zip(curve.forward_rates.iter())
            .map(|(&t, &r)| (t, r))
            .collect();

        chart
            .draw_series(LineSeries::new(points, color.stroke_width(2)))
            .map_err(|e| BtError::Render(e.to_string()))?
            .label(&curve.name)
            .legend(move |(x, y)| {
                PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2))
            });
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

pub fn render_png(curves: &[ForwardCurve], cfg: &FwcvForwardConfig, path: &str) -> Result<()> {
    render(png_root(path)?, curves, cfg)
}

pub fn render_svg(curves: &[ForwardCurve], cfg: &FwcvForwardConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, curves, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let curves = vec![
            ForwardCurve::new("1Y Forward", vec![0.0, 1.0, 2.0, 3.0, 5.0], vec![3.5, 3.8, 4.1, 4.3, 4.5]),
            ForwardCurve::new("2Y Forward", vec![0.0, 1.0, 2.0, 3.0, 5.0], vec![3.6, 3.9, 4.2, 4.4, 4.6]),
        ];
        let cfg = FwcvForwardConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_fwcv_forward.png").to_str().unwrap().to_string();
        render_png(&curves, &cfg, &path).unwrap();
    }
}
