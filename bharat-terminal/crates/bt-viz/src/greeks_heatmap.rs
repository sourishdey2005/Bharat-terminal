// crates/bt-viz/src/greeks_heatmap.rs
// Author: Sourish Dey

//! Greeks heatmap grid. Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct GreeksHeatmapConfig {
    pub title: String,
    pub theme: Theme,
    pub strikes: Vec<f64>,
    pub expiries: Vec<f64>,
    pub greek_name: String,
    pub values: Vec<Vec<f64>>,
}

impl Default for GreeksHeatmapConfig {
    fn default() -> Self {
        Self {
            title: "Greeks Heatmap".to_string(),
            theme: Theme::Dark,
            strikes: vec![],
            expiries: vec![],
            greek_name: "Delta".to_string(),
            values: vec![],
        }
    }
}

impl GreeksHeatmapConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn strikes(mut self, s: Vec<f64>) -> Self { self.strikes = s; self }
    pub fn expiries(mut self, e: Vec<f64>) -> Self { self.expiries = e; self }
    pub fn greek_name(mut self, n: impl Into<String>) -> Self { self.greek_name = n.into(); self }
    pub fn values(mut self, v: Vec<Vec<f64>>) -> Self { self.values = v; self }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    cfg: &GreeksHeatmapConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    if cfg.values.is_empty() || cfg.strikes.is_empty() || cfg.expiries.is_empty() {
        return Err(BtError::EmptySeries("greeks heatmap data".into()));
    }
    fill_background(&root, cfg.theme)?;

    let n_strikes = cfg.strikes.len();
    let n_expiries = cfg.expiries.len();
    let v_min = cfg
        .values
        .iter()
        .flat_map(|r| r.iter().cloned())
        .fold(f64::MAX, f64::min);
    let v_max = cfg
        .values
        .iter()
        .flat_map(|r| r.iter().cloned())
        .fold(f64::MIN, f64::max);

    let mut chart = ChartBuilder::on(&root)
        .caption(&cfg.title, (TITLE_FONT, 22).into_font().color(&cfg.theme.text()))
        .margin(15)
        .x_label_area_size(35)
        .y_label_area_size(60)
        .build_cartesian_2d(0..n_strikes, 0..n_expiries)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .x_labels(n_strikes)
        .y_labels(n_expiries)
        .x_label_formatter(&|idx| {
            cfg.strikes.get(*idx).map(|v| format!("{:.0}", v)).unwrap_or_default()
        })
        .y_label_formatter(&|idx| {
            cfg.expiries.get(*idx).map(|v| format!("{:.2}", v)).unwrap_or_default()
        })
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    for (j, row) in cfg.values.iter().enumerate() {
        for (i, &v) in row.iter().enumerate() {
            let t = if v_max > v_min {
                (v - v_min) / (v_max - v_min)
            } else {
                0.5
            };
            let color = blend_colors(cfg.theme.loss(), cfg.theme.profit(), t);
            chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(i, n_expiries - 1 - j), (i + 1, n_expiries - j)],
                    color.filled(),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;

            chart
                .draw_series(std::iter::once(Text::new(
                    format!("{:.3}", v),
                    (i, n_expiries - 1 - j),
                    (LABEL_FONT, 11).into_font().color(&cfg.theme.text()),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    chart
        .draw_series(std::iter::once(Text::new(
            cfg.greek_name.clone(),
            (n_strikes + 2, n_expiries / 2),
            (LABEL_FONT, 14).into_font().color(&cfg.theme.accent()),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

fn blend_colors(a: RGBColor, b: RGBColor, t: f64) -> RGBColor {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: u8, y: u8| (x as f64 + (y as f64 - x as f64) * t).round() as u8;
    RGBColor(lerp(a.0, b.0), lerp(a.1, b.1), lerp(a.2, b.2))
}

pub fn render_png(cfg: &GreeksHeatmapConfig, path: &str) -> Result<()> {
    render(png_root(path)?, cfg)
}

pub fn render_svg(cfg: &GreeksHeatmapConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let cfg = GreeksHeatmapConfig::new()
            .theme(Theme::Dark)
            .strikes(vec![90.0, 100.0, 110.0])
            .expiries(vec![0.25, 0.5, 1.0])
            .greek_name("Delta")
            .values(vec![
                vec![0.8, 0.7, 0.6],
                vec![0.5, 0.5, 0.5],
                vec![0.2, 0.3, 0.4],
            ]);
        let path = std::env::temp_dir().join("bt_test_greeks_heatmap.png").to_str().unwrap().to_string();
        render_png(&cfg, &path).unwrap();
    }
}
