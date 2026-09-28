// crates/bt-viz/src/3d_copula.rs
// Author: Sourish Dey

//! 3D copula — joint tail dependence as a 3D surface.
//! Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct Copula3DConfig {
    pub title: String,
    pub theme: Theme,
    pub rho: f64,
}

impl Default for Copula3DConfig {
    fn default() -> Self {
        Self {
            title: "3D Copula Surface".to_string(),
            theme: Theme::Dark,
            rho: 0.6,
        }
    }
}

impl Copula3DConfig {
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
    pub fn rho(mut self, v: f64) -> Self {
        self.rho = v.clamp(-0.95, 0.95);
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

fn norm_inv(p: f64) -> f64 {
    let p = p.clamp(1e-9, 1.0 - 1e-9);
    let a = [
        -3.969683028665376e+01,
        2.209460984245205e+02,
        -2.759285104469687e+02,
        1.383577518672690e+02,
        -3.066479806614716e+01,
        2.506628277459239e+00,
    ];
    let b = [
        -5.447609879822406e+01,
        1.615858368580409e+02,
        -1.556989798598866e+02,
        6.680131188771972e+01,
        -1.328068155288572e+01,
    ];
    let c = [
        -7.784894002430293e-03,
        -3.223964580411365e-01,
        -2.400758277161838e+00,
        -2.549732539343734e+00,
        4.374664141464968e+00,
        2.938163982698783e+00,
    ];
    let d = [
        7.784695709041462e-03,
        3.224671290700398e-01,
        2.445134137142996e+00,
        3.754408661907416e+00,
    ];
    let plow = 0.02425;
    let phigh = 1.0 - plow;
    if p < plow {
        let q = (-2.0 * p.ln()).sqrt();
        (((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    } else if p > phigh {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    } else {
        let q = p - 0.5;
        let r = q * q;
        (((((a[0] * r + a[1]) * r + a[2]) * r + a[3]) * r + a[4]) * r + a[5]) * q
            / (((((b[0] * r + b[1]) * r + b[2]) * r + b[3]) * r + b[4]) * r + 1.0)
    }
}

fn copula_density(u: f64, v: f64, rho: f64) -> f64 {
    let x = norm_inv(u);
    let y = norm_inv(v);
    let r2 = rho * rho;
    let denom = (1.0 - r2).sqrt();
    (-(r2 * (x * x + y * y) - 2.0 * rho * x * y) / (2.0 * (1.0 - r2))).exp() / denom
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &Copula3DConfig,
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
        format!("{} — {} (ρ={:.2})", cfg.title, series.symbol, cfg.rho),
        (w as i32 / 2 - 150, 14),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let n = 16;
    let x_span = 340.0;
    let z_span = 220.0;
    let y_span = 180.0;

    let mut grid: Vec<Vec<(i32, i32)>> = Vec::with_capacity(n);
    let mut vals: Vec<Vec<f64>> = Vec::with_capacity(n);

    for j in 0..n {
        let v = (j as f64 + 0.5) / n as f64;
        let mut row = Vec::with_capacity(n);
        let mut vrow = Vec::with_capacity(n);
        for i in 0..n {
            let u = (i as f64 + 0.5) / n as f64;
            let density = copula_density(u, v, cfg.rho);
            let x = (i as f64 / (n - 1) as f64 - 0.5) * x_span;
            let z = (j as f64 / (n - 1) as f64 - 0.5) * z_span;
            let py = density * y_span * 0.8;
            row.push(project(x, py, z, cx, cy));
            vrow.push(density);
        }
        grid.push(row);
        vals.push(vrow);
    }

    let max_d = vals
        .iter()
        .flat_map(|r| r.iter().copied())
        .fold(0.0_f64, f64::max)
        .max(1e-9);

    for j in (0..n.saturating_sub(1)).rev() {
        for i in (0..n.saturating_sub(1)).rev() {
            let d_avg = (vals[j][i] + vals[j][i + 1] + vals[j + 1][i] + vals[j + 1][i + 1]) / 4.0;
            let t = (d_avg / max_d).clamp(0.0, 1.0);
            let color = if t > 0.6 {
                cfg.theme.loss().mix(0.75)
            } else if t > 0.3 {
                cfg.theme.accent().mix(0.7)
            } else {
                cfg.theme.info().mix(0.7)
            };
            let depth_f = 0.4 + 0.6 * (j as f64 / n as f64);
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

    for j in 0..n {
        let pts: Vec<(i32, i32)> = (0..n).map(|i| grid[j][i]).collect();
        root.draw(&PathElement::new(pts, cfg.theme.border().stroke_width(1)))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }
    for i in 0..n {
        let pts: Vec<(i32, i32)> = (0..n).map(|j| grid[j][i]).collect();
        root.draw(&PathElement::new(pts, cfg.theme.border().stroke_width(1)))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    root.draw(&Text::new(
        "X: u  Y: density  Z: v — tail dependence visible at corners",
        (w as i32 / 2 - 170, h as i32 - 40),
        (LABEL_FONT, 11)
            .into_font()
            .color(&cfg.theme.text().mix(0.6)),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &Copula3DConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &Copula3DConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = Copula3DConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_3d_copula.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
