// crates/bt-viz/src/3d_corr_sphere.rs
// Author: Sourish Dey

//! 3D correlation sphere — N assets on a sphere, closer = higher correlation.
//! Made by Sourish Dey.

use bt_analytics::correlation_matrix;
use bt_core::{BtError, OhlcvSeries, Result, synthetic_correlated_returns};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CorrSphere3DConfig {
    pub title: String,
    pub theme: Theme,
    pub rotation: f64,
}

impl Default for CorrSphere3DConfig {
    fn default() -> Self {
        Self { title: "3D Correlation Sphere".to_string(), theme: Theme::Dark, rotation: 0.6 }
    }
}

impl CorrSphere3DConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn rotation(mut self, v: f64) -> Self { self.rotation = v; self }
}

fn project(x: f64, y: f64, z: f64, cx: f64, cy: f64) -> (i32, i32, f64) {
    let focal = 420.0;
    let s = focal / (focal + z);
    ((cx + x * s) as i32, (cy - y * s) as i32, s)
}

fn shade(c: RGBColor, f: f64) -> RGBColor {
    let f = f.clamp(0.15, 1.2);
    RGBColor(
        (c.0 as f64 * f).min(255.0) as u8,
        (c.1 as f64 * f).min(255.0) as u8,
        (c.2 as f64 * f).min(255.0) as u8,
    )
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &CorrSphere3DConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let (w, h) = root.dim_in_pixel();
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0 + 30.0;

    root.draw(&Text::new(
        format!("{} — {}", cfg.title, series.symbol),
        (w as i32 / 2 - 140, 14),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let symbols = ["RELIANCE", "TCS", "INFY", "HDFC", "ICICI", "SBIN"];
    let n_a = symbols.len();
    let data = synthetic_correlated_returns(&symbols, 120, 7);
    let corr = correlation_matrix(&data);

    let radius = 150.0;
    let cos_r = cfg.rotation.cos();
    let sin_r = cfg.rotation.sin();

    let mut nodes: Vec<((i32, i32, f64), usize)> = Vec::with_capacity(n_a);
    for (idx, _) in symbols.iter().enumerate() {
        let phi = (idx as f64) * 2.399963;
        let y = 1.0 - (idx as f64 / (n_a - 1) as f64) * 2.0;
        let r = (1.0 - y * y).sqrt();
        let x = phi.cos() * r;
        let z = phi.sin() * r;
        let avg_corr = if n_a > 1 {
            corr[idx].iter().sum::<f64>() / (n_a - 1) as f64
        } else {
            0.0
        };
        let lat = avg_corr * 0.8;
        let px = x * radius * (1.0 - lat * 0.3);
        let py = (y + lat) * radius * 0.8;
        let pz = z * radius;
        let rx = px * cos_r - pz * sin_r;
        let rz = px * sin_r + pz * cos_r;
        let (sx, sy, s) = project(rx, py, rz, cx, cy);
        nodes.push(((sx, sy, s), idx));
    }

    let mut edges: Vec<((usize, usize), f64)> = Vec::new();
    for i in 0..n_a {
        for j in (i + 1)..n_a {
            let c = corr[i][j];
            if c.abs() > 0.3 {
                edges.push(((i, j), c));
            }
        }
    }
    edges.sort_by(|a, b| a.1.abs().partial_cmp(&b.1.abs()).unwrap_or(std::cmp::Ordering::Equal));

    for ((i, j), c) in &edges {
        let (p1, _) = nodes[*i];
        let (p2, _) = nodes[*j];
        let depth = (p1.2 + p2.2) / 2.0;
        let color = if *c > 0.0 { cfg.theme.profit() } else { cfg.theme.loss() };
        let alpha = (c.abs() * depth).clamp(0.15, 0.9);
        root.draw(&PathElement::new(
            vec![(p1.0, p1.1), (p2.0, p2.1)],
            shade(color, alpha).stroke_width((c.abs() * 3.0) as u32 + 1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    nodes.sort_by(|a, b| a.0 .2.partial_cmp(&b.0 .2).unwrap_or(std::cmp::Ordering::Equal));
    for ((sx, sy, s), idx) in &nodes {
        let size = (4.0 * s) as i32 + 2;
        root.draw(&Circle::new((*sx, *sy), size, cfg.theme.accent().filled()))
            .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Circle::new((*sx, *sy), size + 2, cfg.theme.accent().stroke_width(1)))
            .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            symbols[*idx],
            (sx + size + 4, sy - 5),
            (LABEL_FONT, 10).into_font().color(&cfg.theme.text().mix(0.8)),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    root.draw(&Text::new(
        "Node proximity = correlation strength",
        (w as i32 / 2 - 110, h as i32 - 40),
        (LABEL_FONT, 11).into_font().color(&cfg.theme.text().mix(0.6)),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &CorrSphere3DConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &CorrSphere3DConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = CorrSphere3DConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_3d_corr_sphere.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
