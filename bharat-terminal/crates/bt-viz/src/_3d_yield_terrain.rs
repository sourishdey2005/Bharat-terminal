// crates/bt-viz/src/3d_yield_terrain.rs
// Author: Sourish Dey

//! 3D yield terrain — maturity × time × yield as flowing terrain.
//! Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct YieldTerrain3DConfig {
    pub title: String,
    pub theme: Theme,
    pub flow: f64,
}

impl Default for YieldTerrain3DConfig {
    fn default() -> Self {
        Self { title: "3D Yield Terrain".to_string(), theme: Theme::Dark, flow: 0.5 }
    }
}

impl YieldTerrain3DConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn flow(mut self, v: f64) -> Self { self.flow = v.clamp(0.0, 1.0); self }
}

fn project(x: f64, y: f64, z: f64, cx: f64, cy: f64) -> (i32, i32) {
    let focal = 500.0;
    let s = focal / (focal + z);
    ((cx + x * s) as i32, (cy - y * s) as i32)
}

fn shade(c: RGBAColor, f: f64) -> RGBColor {
    let f = f.clamp(0.15, 1.2);
    RGBColor(
        (c.0 as f64 * f).min(255.0) as u8,
        (c.1 as f64 * f).min(255.0) as u8,
        (c.2 as f64 * f).min(255.0) as u8,
    )
}

fn yield_at(maturity: f64, time: f64, base: f64, flow: f64) -> f64 {
    let wave = (maturity * 2.0 + time * flow * 3.0).sin() * 0.3;
    base + 0.5 * (1.0 - (-maturity * 0.4).exp()) + wave * 0.1
        + 0.2 * (time * 0.5).sin() * maturity
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &YieldTerrain3DConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let (w, h) = root.dim_in_pixel();
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0 + 40.0;

    root.draw(&Text::new(
        format!("{} — {}", cfg.title, series.symbol),
        (w as i32 / 2 - 130, 14),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let returns = series.returns();
    let base = if returns.is_empty() {
        0.05
    } else {
        let n = returns.len() as f64;
        let mean = returns.iter().sum::<f64>() / n;
        (mean * 252.0).clamp(0.01, 0.15)
    };

    let n_m = 12;
    let n_t = 16;
    let x_span = 360.0;
    let z_span = 240.0;
    let y_span = 140.0;

    let mut grid: Vec<Vec<(i32, i32)>> = Vec::with_capacity(n_t);
    let mut vals: Vec<Vec<f64>> = Vec::with_capacity(n_t);

    for j in 0..n_t {
        let time = j as f64 / (n_t - 1) as f64 * 5.0;
        let mut row = Vec::with_capacity(n_m);
        let mut vrow = Vec::with_capacity(n_m);
        for i in 0..n_m {
            let maturity = (i as f64 + 1.0) / n_m as f64 * 10.0;
            let y = yield_at(maturity, time, base, cfg.flow);
            let x = (i as f64 / (n_m - 1) as f64 - 0.5) * x_span;
            let z = (j as f64 / (n_t - 1) as f64 - 0.5) * z_span;
            let py = (y - base) * y_span * 6.0;
            row.push(project(x, py, z, cx, cy));
            vrow.push(y);
        }
        grid.push(row);
        vals.push(vrow);
    }

    for j in (0..n_t.saturating_sub(1)).rev() {
        for i in (0..n_m.saturating_sub(1)).rev() {
            let y_avg = (vals[j][i] + vals[j][i + 1] + vals[j + 1][i] + vals[j + 1][i + 1]) / 4.0;
            let t = ((y_avg - base) * 8.0).clamp(0.0, 1.0);
            let color = if t > 0.5 {
                cfg.theme.profit().mix(0.7)
            } else {
                cfg.theme.info().mix(0.7)
            };
            let depth_f = 0.4 + 0.6 * (j as f64 / n_t as f64);
            let pts = vec![grid[j][i], grid[j][i + 1], grid[j + 1][i + 1], grid[j + 1][i]];
            root.draw(&Polygon::new(pts, shade(color, depth_f).filled()))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    for j in 0..n_t {
        let pts: Vec<(i32, i32)> = (0..n_m).map(|i| grid[j][i]).collect();
        root.draw(&PathElement::new(pts, cfg.theme.border().stroke_width(1)))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }
    for i in 0..n_m {
        let pts: Vec<(i32, i32)> = (0..n_t).map(|j| grid[j][i]).collect();
        root.draw(&PathElement::new(pts, cfg.theme.border().stroke_width(1)))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    root.draw(&Text::new(
        "X: maturity  Y: yield  Z: time",
        (w as i32 / 2 - 90, h as i32 - 40),
        (LABEL_FONT, 11).into_font().color(&cfg.theme.text().mix(0.6)),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &YieldTerrain3DConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &YieldTerrain3DConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = YieldTerrain3DConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_3d_yield_terrain.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
