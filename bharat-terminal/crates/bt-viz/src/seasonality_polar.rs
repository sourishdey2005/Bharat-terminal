//! Tier 5 #35 — Seasonality Polar Heatmap (month x day-of-week radial).
//! Made by Sourish Dey.
//!
//! plotters doesn't ship a polar coordinate system, so this draws true
//! annular-sector wedges directly in pixel space: each wedge is
//! approximated by a small fan of triangles along its arc, which is
//! exact in the limit and visually indistinguishable from a true arc at
//! the segment counts used here.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

pub const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
pub const WEEKDAYS: [&str; 5] = ["Mon", "Tue", "Wed", "Thu", "Fri"];

/// `data[month][weekday]` = average return (%) for that month/weekday cell.
#[derive(Debug, Clone)]
pub struct SeasonalityGrid {
    pub data: Vec<Vec<f64>>, // 12 x 5
}

impl SeasonalityGrid {
    pub fn new(data: Vec<Vec<f64>>) -> Result<Self> {
        if data.len() != 12 || data.iter().any(|row| row.len() != WEEKDAYS.len()) {
            return Err(BtError::InvalidInput(
                "seasonality grid must be 12 months x 5 weekdays".into(),
            ));
        }
        Ok(Self { data })
    }
}

/// Deterministic synthetic seasonality grid for demos/tests.
pub fn synthetic_seasonality(seed: u64) -> SeasonalityGrid {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    let mut rng = StdRng::seed_from_u64(seed);
    let data = (0..12)
        .map(|m| {
            (0..5)
                .map(|d| {
                    let seasonal = ((m as f64 / 12.0) * std::f64::consts::TAU).sin() * 1.2;
                    let weekday_effect = if d == 4 { 0.3 } else { -0.05 };
                    seasonal + weekday_effect + rng.gen_range(-0.4..0.4)
                })
                .collect()
        })
        .collect();
    SeasonalityGrid { data }
}

#[derive(Debug, Clone)]
pub struct SeasonalityConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for SeasonalityConfig {
    fn default() -> Self {
        Self {
            title: "Seasonality — Month x Weekday".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl SeasonalityConfig {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

fn wedge_points(
    cx: f64,
    cy: f64,
    r_inner: f64,
    r_outer: f64,
    theta0: f64,
    theta1: f64,
    segments: usize,
) -> Vec<(i32, i32)> {
    let mut pts = Vec::with_capacity(segments * 2 + 2);
    for s in 0..=segments {
        let t = theta0 + (theta1 - theta0) * (s as f64 / segments as f64);
        pts.push(((cx + r_outer * t.cos()) as i32, (cy + r_outer * t.sin()) as i32));
    }
    for s in (0..=segments).rev() {
        let t = theta0 + (theta1 - theta0) * (s as f64 / segments as f64);
        pts.push(((cx + r_inner * t.cos()) as i32, (cy + r_inner * t.sin()) as i32));
    }
    pts
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    grid: &SeasonalityGrid,
    cfg: &SeasonalityConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    fill_background(&root, cfg.theme)?;

    root.draw(&Text::new(
        cfg.title.clone(),
        (15, 8),
        (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let (w, h) = root.dim_in_pixel();
    let cx = w as f64 / 2.0;
    let cy = (h as f64 - 40.0) / 2.0 + 50.0;
    let r_max = (w.min(h) as f64) * 0.38;
    let r_min = r_max * 0.22;
    let ring_step = (r_max - r_min) / WEEKDAYS.len() as f64;

    let max_abs = grid
        .data
        .iter()
        .flatten()
        .cloned()
        .fold(0.0_f64, |a, v| a.max(v.abs()))
        .max(1e-6);

    for (m, row) in grid.data.iter().enumerate() {
        let theta0 = -std::f64::consts::FRAC_PI_2 + (m as f64) * std::f64::consts::TAU / 12.0;
        let theta1 = theta0 + std::f64::consts::TAU / 12.0;

        for (d, &v) in row.iter().enumerate() {
            let r_inner = r_min + ring_step * d as f64;
            let r_outer = r_inner + ring_step * 0.92;
            let t = (v / max_abs).clamp(-1.0, 1.0);
            let target = if t >= 0.0 { cfg.theme.profit() } else { cfg.theme.loss() };
            let bg = cfg.theme.background();
            let alpha = t.abs();
            let lerp = |a: u8, b: u8| (a as f64 + (b as f64 - a as f64) * alpha).round() as u8;
            let color = RGBColor(lerp(bg.0, target.0), lerp(bg.1, target.1), lerp(bg.2, target.2));

            let pts = wedge_points(cx, cy, r_inner, r_outer, theta0, theta1, 10);
            root.draw(&Polygon::new(pts, color.filled()))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }

        let label_theta = (theta0 + theta1) / 2.0;
        let label_r = r_max + 18.0;
        root.draw(&Text::new(
            MONTHS[m].to_string(),
            (
                (cx + label_r * label_theta.cos()) as i32 - 10,
                (cy + label_r * label_theta.sin()) as i32 - 6,
            ),
            (LABEL_FONT, 13).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    // Weekday ring legend on the left.
    for (d, wd) in WEEKDAYS.iter().enumerate() {
        root.draw(&Text::new(
            format!("Ring {}: {}", d + 1, wd),
            (15, 40 + d as i32 * 18),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(grid: &SeasonalityGrid, cfg: &SeasonalityConfig, path: &str) -> Result<()> {
    render(png_root(path)?, grid, cfg)
}

pub fn render_svg(grid: &SeasonalityGrid, cfg: &SeasonalityConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, grid, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_constructs() {
        let cfg = SeasonalityConfig::new();
        let _ = cfg;
    }
}
