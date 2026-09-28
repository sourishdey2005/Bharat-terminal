// crates/bt-viz/src/3d_flow_river.rs
// Author: Sourish Dey

//! 3D flow river — particles flowing: X=price, Y=time, Z=buy/sell pressure.
//! Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct FlowRiver3DConfig {
    pub title: String,
    pub theme: Theme,
    pub n_particles: usize,
    pub seed: u64,
}

impl Default for FlowRiver3DConfig {
    fn default() -> Self {
        Self { title: "3D Flow River".to_string(), theme: Theme::Dark, n_particles: 800, seed: 7 }
    }
}

impl FlowRiver3DConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn n_particles(mut self, n: usize) -> Self { self.n_particles = n.max(50); self }
    pub fn seed(mut self, s: u64) -> Self { self.seed = s; self }
}

fn project(x: f64, y: f64, z: f64, cx: f64, cy: f64) -> (i32, i32, f64) {
    let focal = 480.0;
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
    cfg: &FlowRiver3DConfig,
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
        (w as i32 / 2 - 120, 14),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let min_p = series.candles.iter().map(|c| c.low).fold(f64::MAX, f64::min);
    let max_p = series.candles.iter().map(|c| c.high).fold(f64::MIN, f64::max);
    let pr = (max_p - min_p).max(1e-9);
    let n = series.candles.len();

    let mut rng = StdRng::seed_from_u64(cfg.seed);
    let x_span = 340.0;
    let y_span = 200.0;
    let z_span = 220.0;

    let mut particles: Vec<((i32, i32, f64), f64)> = Vec::with_capacity(cfg.n_particles);
    for _ in 0..cfg.n_particles {
        let t_idx = rng.gen_range(0..n.max(1));
        let c = &series.candles[t_idx];
        let price = c.close + rng.gen_range(-0.02..0.02) * pr;
        let pressure = if c.is_bullish() { 1.0 } else { -1.0 };
        let x = (t_idx as f64 / n as f64 - 0.5) * x_span;
        let y = ((price - min_p) / pr - 0.5) * y_span;
        let z = pressure * z_span * 0.5 + rng.gen_range(-20.0..20.0);
        let (sx, sy, s) = project(x, y, z, cx, cy);
        particles.push(((sx, sy, s), pressure));
    }

    particles.sort_by(|a, b| a.0 .2.partial_cmp(&b.0 .2).unwrap_or(std::cmp::Ordering::Equal));

    for ((sx, sy, s), pressure) in &particles {
        let size = (2.0 * s) as i32 + 1;
        let color = if *pressure > 0.0 { cfg.theme.profit() } else { cfg.theme.loss() };
        let alpha = (0.3 + 0.5 * s).clamp(0.2, 0.8);
        root.draw(&Circle::new((*sx, *sy), size, shade(color, alpha).filled()))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    root.draw(&Text::new(
        "X: price  Y: time  Z: buy/sell pressure",
        (w as i32 / 2 - 120, h as i32 - 40),
        (LABEL_FONT, 11).into_font().color(&cfg.theme.text().mix(0.6)),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &FlowRiver3DConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &FlowRiver3DConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = FlowRiver3DConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_3d_flow_river.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
