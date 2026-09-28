// crates/bt-viz/src/wirp_probability.rs
// Author: Sourish Dey

//! WIRP: Rate probability forecast.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct WirpProbabilityConfig {
    pub title: String,
    pub theme: Theme,
    pub horizon: usize,
    pub current_rate: f64,
    pub mean_forecast: f64,
    pub std_dev: f64,
}

impl Default for WirpProbabilityConfig {
    fn default() -> Self {
        Self {
            title: "Rate Probability Forecast".to_string(),
            theme: Theme::Dark,
            horizon: 12,
            current_rate: 6.5,
            mean_forecast: 6.0,
            std_dev: 0.5,
        }
    }
}

impl WirpProbabilityConfig {
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
    pub fn horizon(mut self, h: usize) -> Self {
        self.horizon = h.max(3).min(36);
        self
    }
    pub fn current_rate(mut self, r: f64) -> Self {
        self.current_rate = r;
        self
    }
    pub fn mean_forecast(mut self, m: f64) -> Self {
        self.mean_forecast = m;
        self
    }
    pub fn std_dev(mut self, s: f64) -> Self {
        self.std_dev = s.max(0.01);
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
    cfg: &WirpProbabilityConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    fill_background(&root, cfg.theme)?;

    let mut probs: Vec<f64> = Vec::with_capacity(cfg.horizon);
    for i in 0..cfg.horizon {
        let t = (i + 1) as f64;
        let mean_t = cfg.current_rate
            + (cfg.mean_forecast - cfg.current_rate) * (1.0 - (-t / cfg.horizon as f64).exp());
        let std_t = cfg.std_dev * t.sqrt();
        let p = normal_pdf(cfg.current_rate, mean_t, std_t);
        probs.push(p);
    }

    let max_prob = probs.iter().cloned().fold(0.0_f64, f64::max);
    let prob_sum: f64 = probs.iter().sum();
    let norm_probs: Vec<f64> = probs
        .iter()
        .map(|&p| if prob_sum > 0.0 { p / prob_sum } else { 0.0 })
        .collect();

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — {}M Horizon", cfg.title, cfg.horizon),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0..cfg.horizon, 0.0..max_prob * 1.2)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .x_desc("Months Ahead")
        .y_desc("Probability Density")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(norm_probs.iter().enumerate().map(|(i, &p)| {
            Rectangle::new([(i, 0.0), (i + 1, p)], cfg.theme.info().mix(0.6).filled())
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(LineSeries::new(
            norm_probs.iter().enumerate().map(|(i, &p)| (i, p)),
            cfg.theme.accent().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    let (w, h) = root.dim_in_pixel();
    root.draw(&Text::new(
        format!(
            "Current: {:.2}% | Forecast: {:.2}%",
            cfg.current_rate, cfg.mean_forecast
        ),
        (w as i32 / 2 - 120, 60),
        (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(cfg: &WirpProbabilityConfig, path: &str) -> Result<()> {
    render(png_root(path)?, cfg)
}

pub fn render_svg(cfg: &WirpProbabilityConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let cfg = WirpProbabilityConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_wirp_probability.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&cfg, &path).unwrap();
    }
}
