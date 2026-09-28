// crates/bt-viz/src/3d_vol_surface.rs
// Author: Sourish Dey

//! 3D implied-volatility surface — strike × expiry × IV as a mesh surface.
//! Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct VolSurface3DConfig {
    pub title: String,
    pub theme: Theme,
    pub skew: f64,
}

impl Default for VolSurface3DConfig {
    fn default() -> Self {
        Self {
            title: "3D Vol Surface".to_string(),
            theme: Theme::Dark,
            skew: 0.3,
        }
    }
}

impl VolSurface3DConfig {
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
    pub fn skew(mut self, v: f64) -> Self {
        self.skew = v.clamp(-1.0, 1.0);
        self
    }
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

fn iv_at(strike: f64, expiry: f64, base: f64, skew: f64) -> f64 {
    let m = strike - 1.0;
    base + skew * m * m * 0.05 + 0.02 * (1.0 - (-expiry * 0.3).exp())
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &VolSurface3DConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
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
    let base_vol = if returns.is_empty() {
        0.2
    } else {
        let n = returns.len() as f64;
        let mean = returns.iter().sum::<f64>() / n;
        returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n
    }
    .sqrt()
        * 100.0;

    let n_k = 14;
    let n_t = 10;
    let x_span = 360.0;
    let z_span = 220.0;
    let y_span = 160.0;

    let mut grid: Vec<Vec<(i32, i32)>> = Vec::with_capacity(n_t);
    let mut vals: Vec<Vec<f64>> = Vec::with_capacity(n_t);

    for j in 0..n_t {
        let expiry = (j as f64 + 1.0) / n_t as f64 * 2.0;
        let mut row = Vec::with_capacity(n_k);
        let mut vrow = Vec::with_capacity(n_k);
        for i in 0..n_k {
            let strike = 0.7 + (i as f64 / (n_k - 1) as f64) * 0.6;
            let iv = iv_at(strike, expiry, base_vol / 100.0, cfg.skew);
            let x = (i as f64 / (n_k - 1) as f64 - 0.5) * x_span;
            let z = (j as f64 / (n_t - 1) as f64 - 0.5) * z_span;
            let y = (iv - base_vol / 100.0) * y_span * 8.0;
            row.push(project(x, y, z, cx, cy));
            vrow.push(iv);
        }
        grid.push(row);
        vals.push(vrow);
    }

    for j in (0..n_t.saturating_sub(1)).rev() {
        for i in (0..n_k.saturating_sub(1)).rev() {
            let iv_avg = (vals[j][i] + vals[j][i + 1] + vals[j + 1][i] + vals[j + 1][i + 1]) / 4.0;
            let t = ((iv_avg - base_vol / 100.0) * 4.0).clamp(0.0, 1.0);
            let color = if t > 0.5 {
                cfg.theme.loss().mix(0.7)
            } else {
                cfg.theme.info().mix(0.7)
            };
            let depth_f = 0.4 + 0.6 * (j as f64 / n_t as f64);
            let pts = vec![
                grid[j][i],
                grid[j][i + 1],
                grid[j + 1][i + 1],
                grid[j + 1][i],
            ];
            root.draw(&Polygon::new(pts, shade(color, depth_f).filled()))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    for j in 0..n_t {
        let pts: Vec<(i32, i32)> = (0..n_k).map(|i| grid[j][i]).collect();
        root.draw(&PathElement::new(pts, cfg.theme.border().stroke_width(1)))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }
    for i in 0..n_k {
        let pts: Vec<(i32, i32)> = (0..n_t).map(|j| grid[j][i]).collect();
        root.draw(&PathElement::new(pts, cfg.theme.border().stroke_width(1)))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    root.draw(&Text::new(
        "X: strike  Y: IV  Z: expiry",
        (w as i32 / 2 - 90, h as i32 - 40),
        (LABEL_FONT, 11)
            .into_font()
            .color(&cfg.theme.text().mix(0.6)),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &VolSurface3DConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &VolSurface3DConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = VolSurface3DConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_3d_vol_surface.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
