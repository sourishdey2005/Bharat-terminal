// crates/bt-viz/src/var_distribution.rs
// Author: Sourish Dey

//! VaR distribution.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;
use rand::Rng;
use rand::SeedableRng;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct VarDistributionConfig {
    pub title: String,
    pub theme: Theme,
    pub confidence: f64,
    pub num_buckets: usize,
    pub returns: Vec<f64>,
}

impl Default for VarDistributionConfig {
    fn default() -> Self {
        Self {
            title: "VaR Distribution".to_string(),
            theme: Theme::Dark,
            confidence: 0.95,
            num_buckets: 20,
            returns: Vec::new(),
        }
    }
}

impl VarDistributionConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn confidence(mut self, c: f64) -> Self { self.confidence = c.clamp(0.5, 0.999); self }
    pub fn num_buckets(mut self, b: usize) -> Self { self.num_buckets = b.max(5).min(50); self }
    pub fn returns(mut self, r: Vec<f64>) -> Self { self.returns = r; self }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    cfg: &VarDistributionConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    fill_background(&root, cfg.theme)?;

    let returns = if cfg.returns.is_empty() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        (0..200).map(|_| {
            let shock: f64 = rng.gen_range(-1.0..1.0);
            -0.02 + shock * 0.015
        }).collect()
    } else {
        cfg.returns.clone()
    };

    if returns.is_empty() {
        return Err(BtError::EmptySeries("returns".into()));
    }

    let mut sorted = returns.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let var_idx = ((1.0 - cfg.confidence) * sorted.len() as f64).floor() as usize;
    let var_idx = var_idx.min(sorted.len().saturating_sub(1));
    let var_value = -sorted[var_idx] * 100.0;

    let r_min = sorted.first().cloned().unwrap_or(0.0) * 100.0;
    let r_max = sorted.last().cloned().unwrap_or(0.0) * 100.0;
    let bucket_width = (r_max - r_min) / cfg.num_buckets as f64;

    let mut buckets = vec![0.0_f64; cfg.num_buckets];
    for &r in &returns {
        let pct = r * 100.0;
        let idx = ((pct - r_min) / bucket_width).floor() as usize;
        if idx < cfg.num_buckets {
            buckets[idx] += 1.0;
        }
    }

    let max_count = buckets.iter().cloned().fold(0.0_f64, f64::max);

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — {:.0}% VaR: {:.2}%", cfg.title, cfg.confidence * 100.0, var_value),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0..cfg.num_buckets, 0.0..max_count * 1.2)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .x_desc("Return Bucket (%)")
        .y_desc("Frequency")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    for (i, &count) in buckets.iter().enumerate() {
        let bucket_center = r_min + (i as f64 + 0.5) * bucket_width;
        let color = if bucket_center < -var_value {
            cfg.theme.loss().mix(1.0)
        } else {
            cfg.theme.info().mix(0.6)
        };

        chart
            .draw_series(std::iter::once(Rectangle::new(
                [(i, 0.0), (i + 1, count)],
                color.filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(cfg: &VarDistributionConfig, path: &str) -> Result<()> {
    render(png_root(path)?, cfg)
}

pub fn render_svg(cfg: &VarDistributionConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let cfg = VarDistributionConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_var_distribution.png").to_str().unwrap().to_string();
        render_png(&cfg, &path).unwrap();
    }
}
