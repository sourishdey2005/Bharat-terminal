// crates/bt-viz/src/3d_monte_carlo.rs
// Author: Sourish Dey

//! 3D Monte Carlo — 10,000 simulated paths in 3D perspective.
//! Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct MonteCarlo3DConfig {
    pub title: String,
    pub theme: Theme,
    pub n_paths: usize,
    pub horizon: usize,
    pub seed: u64,
}

impl Default for MonteCarlo3DConfig {
    fn default() -> Self {
        Self {
            title: "3D Monte Carlo".to_string(),
            theme: Theme::Dark,
            n_paths: 10_000,
            horizon: 25,
            seed: 42,
        }
    }
}

impl MonteCarlo3DConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn n_paths(mut self, n: usize) -> Self { self.n_paths = n.max(10); self }
    pub fn horizon(mut self, h: usize) -> Self { self.horizon = h.max(5); self }
    pub fn seed(mut self, s: u64) -> Self { self.seed = s; self }
}

fn project(x: f64, y: f64, z: f64, cx: f64, cy: f64) -> (i32, i32) {
    let focal = 500.0;
    let s = focal / (focal + z);
    ((cx + x * s) as i32, (cy - y * s) as i32)
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &MonteCarlo3DConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let (w, h) = root.dim_in_pixel();
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0 + 30.0;

    root.draw(&Text::new(
        format!("{} — {} ({} paths)", cfg.title, series.symbol, cfg.n_paths),
        (w as i32 / 2 - 160, 14),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let returns = series.returns();
    if returns.is_empty() {
        return Err(BtError::EmptySeries("returns".into()));
    }
    let n = returns.len() as f64;
    let mean = returns.iter().sum::<f64>() / n;
    let vol = (returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n).sqrt();
    let last_price = series.candles.last().map(|c| c.close).unwrap_or(100.0);

    let mut rng = StdRng::seed_from_u64(cfg.seed);
    let x_span = 340.0;
    let y_span = 180.0;
    let z_span = 260.0;

    let mut all_paths: Vec<Vec<f64>> = Vec::with_capacity(cfg.n_paths);
    for _ in 0..cfg.n_paths {
        let mut path = Vec::with_capacity(cfg.horizon);
        let mut price = last_price;
        for _ in 0..cfg.horizon {
            let shock: f64 = rng.gen_range(-1.0..1.0);
            price = (price * (1.0 + mean + vol * shock)).max(0.01);
            path.push(price);
        }
        all_paths.push(path);
    }

    let y_min = all_paths.iter().flat_map(|p| p.iter().copied()).fold(f64::MAX, f64::min);
    let y_max = all_paths.iter().flat_map(|p| p.iter().copied()).fold(f64::MIN, f64::max);
    let yr = (y_max - y_min).max(1e-9);

    for (p_idx, path) in all_paths.iter().enumerate() {
        let z = (p_idx as f64 / cfg.n_paths as f64 - 0.5) * z_span;
        let color = if p_idx % 3 == 0 {
            cfg.theme.info()
        } else if p_idx % 3 == 1 {
            cfg.theme.accent()
        } else {
            cfg.theme.text()
        };
        let alpha = 0.08 + 0.1 * (1.0 - p_idx as f64 / cfg.n_paths as f64);
        let pts: Vec<(i32, i32)> = path
            .iter()
            .enumerate()
            .map(|(t, &price)| {
                let x = (t as f64 / cfg.horizon as f64 - 0.5) * x_span;
                let y = ((price - y_min) / yr - 0.5) * y_span;
                project(x, y, z, cx, cy)
            })
            .collect();
        root.draw(&PathElement::new(pts, color.mix(alpha).stroke_width(1)))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    root.draw(&Text::new(
        "X: time  Y: price  Z: path index",
        (w as i32 / 2 - 100, h as i32 - 40),
        (LABEL_FONT, 11).into_font().color(&cfg.theme.text().mix(0.6)),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &MonteCarlo3DConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &MonteCarlo3DConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = MonteCarlo3DConfig::new().theme(Theme::Dark).n_paths(500);
        let path = std::env::temp_dir().join("bt_test_3d_monte_carlo.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
