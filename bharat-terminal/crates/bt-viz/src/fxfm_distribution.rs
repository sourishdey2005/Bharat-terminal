// crates/bt-viz/src/fxfm_distribution.rs
// Author: Sourish Dey

//! FXFM: FX forecast distribution.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct FxfmDistributionConfig {
    pub title: String,
    pub theme: Theme,
    pub current_rate: f64,
    pub forecast_mean: f64,
    pub forecast_std: f64,
    pub horizon_days: usize,
    pub num_buckets: usize,
}

impl Default for FxfmDistributionConfig {
    fn default() -> Self {
        Self {
            title: "FX Forecast Distribution".to_string(),
            theme: Theme::Dark,
            current_rate: 83.5,
            forecast_mean: 84.0,
            forecast_std: 0.8,
            horizon_days: 30,
            num_buckets: 15,
        }
    }
}

impl FxfmDistributionConfig {
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
    pub fn current_rate(mut self, r: f64) -> Self {
        self.current_rate = r;
        self
    }
    pub fn forecast_mean(mut self, m: f64) -> Self {
        self.forecast_mean = m;
        self
    }
    pub fn forecast_std(mut self, s: f64) -> Self {
        self.forecast_std = s.max(0.01);
        self
    }
    pub fn horizon_days(mut self, d: usize) -> Self {
        self.horizon_days = d.max(7).min(365);
        self
    }
    pub fn num_buckets(mut self, b: usize) -> Self {
        self.num_buckets = b.max(5).min(50);
        self
    }
}

fn normal_pdf(x: f64, mean: f64, std: f64) -> f64 {
    let coeff = 1.0 / (std * (2.0 * std::f64::consts::PI).sqrt());
    let exp = -((x - mean).powi(2)) / (2.0 * std * std);
    coeff * exp.exp()
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    cfg: &FxfmDistributionConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    fill_background(&root, cfg.theme)?;

    let mut buckets = vec![0.0_f64; cfg.num_buckets];
    let range = cfg.forecast_std * 4.0;
    let min_rate = cfg.forecast_mean - range;
    let max_rate = cfg.forecast_mean + range;
    let bucket_width = (max_rate - min_rate) / cfg.num_buckets as f64;

    let steps = 1000;
    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let rate = min_rate + t * (max_rate - min_rate);
        let prob = normal_pdf(rate, cfg.forecast_mean, cfg.forecast_std);
        let bucket_idx = ((rate - min_rate) / bucket_width).floor() as usize;
        if bucket_idx < cfg.num_buckets {
            buckets[bucket_idx] += prob;
        }
    }

    let max_prob = buckets.iter().cloned().fold(0.0_f64, f64::max);
    let prob_sum: f64 = buckets.iter().sum();

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — {}D Horizon", cfg.title, cfg.horizon_days),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0..cfg.num_buckets, 0.0..max_prob * 1.2)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .x_desc("Exchange Rate Bucket")
        .y_desc("Probability Density")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(buckets.iter().enumerate().map(|(i, &p)| {
            let norm_p = if prob_sum > 0.0 { p / prob_sum } else { 0.0 };
            Rectangle::new(
                [(i, 0.0), (i + 1, norm_p)],
                cfg.theme.info().mix(0.6).filled(),
            )
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    let current_bucket = ((cfg.current_rate - min_rate) / bucket_width).floor() as usize;
    if current_bucket < cfg.num_buckets {
        chart
            .draw_series(std::iter::once(Rectangle::new(
                [(current_bucket, 0.0), (current_bucket + 1, max_prob * 1.2)],
                cfg.theme.accent().mix(0.2).filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    let (w, h) = root.dim_in_pixel();
    root.draw(&Text::new(
        format!(
            "Current: {:.2} | Forecast: {:.2}",
            cfg.current_rate, cfg.forecast_mean
        ),
        (w as i32 / 2 - 120, 60),
        (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(cfg: &FxfmDistributionConfig, path: &str) -> Result<()> {
    render(png_root(path)?, cfg)
}

pub fn render_svg(cfg: &FxfmDistributionConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let cfg = FxfmDistributionConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_fxfm_distribution.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&cfg, &path).unwrap();
    }
}
