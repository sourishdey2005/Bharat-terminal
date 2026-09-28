// crates/bt-viz/src/wavelet.rs
// Author: Sourish Dey

//! Wavelet scalogram heatmap. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

/// Haar wavelet decomposition energy at each scale/position.
/// Returns rows: scale 0 = finest, scale N = coarsest.
fn haar_energy(data: &[f64], n_scales: usize) -> Vec<Vec<f64>> {
    let mut levels: Vec<Vec<f64>> = Vec::with_capacity(n_scales);
    let mut current: Vec<f64> = data.to_vec();

    for _ in 0..n_scales {
        if current.len() < 2 {
            break;
        }
        let half = current.len() / 2;
        let mut detail = Vec::with_capacity(half);
        let mut approx = Vec::with_capacity(half);
        for i in 0..half {
            let a = current[2 * i];
            let b = current[2 * i + 1];
            detail.push((a - b) / 2.0_f64.sqrt());
            approx.push((a + b) / 2.0_f64.sqrt());
        }
        levels.push(detail);
        current = approx;
    }
    levels
}

#[derive(Debug, Clone)]
pub struct WaveletConfig {
    pub title: String,
    pub theme: Theme,
    pub n_scales: usize,
}

impl Default for WaveletConfig {
    fn default() -> Self {
        Self {
            title: "Wavelet Scalogram".to_string(),
            theme: Theme::Dark,
            n_scales: 6,
        }
    }
}

impl WaveletConfig {
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
    pub fn n_scales(mut self, s: usize) -> Self {
        self.n_scales = s.clamp(2, 10);
        self
    }
}

fn lerp(a: u8, b: u8, t: f64) -> u8 {
    (a as f64 + (b as f64 - a as f64) * t.clamp(0.0, 1.0)).round() as u8
}

fn energy_color(theme: Theme, t: f64) -> RGBColor {
    let bg = theme.background();
    let accent = theme.accent();
    RGBColor(
        lerp(bg.0, accent.0, t),
        lerp(bg.1, accent.1, t),
        lerp(bg.2, accent.2, t),
    )
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &WaveletConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    let returns = series.returns();
    if returns.len() < 16 {
        return Err(BtError::InvalidInput("series too short for wavelet".into()));
    }
    fill_background(&root, cfg.theme)?;

    let levels = haar_energy(&returns, cfg.n_scales);
    let max_energy = levels
        .iter()
        .flatten()
        .map(|v| v.abs())
        .fold(0.0_f64, f64::max)
        .max(1e-9);

    let (w, h) = root.dim_in_pixel();
    let top_pad = 50;
    let bottom_pad = 50;
    let chart_h = h as f64 - top_pad as f64 - bottom_pad as f64;
    let n_rows = levels.len();
    let row_h = chart_h / n_rows as f64;

    root.draw(&Text::new(
        cfg.title.as_str(),
        (w as i32 / 2, 20),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    for (row_idx, row) in levels.iter().enumerate() {
        let y0 = top_pad as f64 + row_idx as f64 * row_h;
        let n_cells = row.len();
        let cell_w = w as f64 / n_cells as f64;
        for (col, &val) in row.iter().enumerate() {
            let intensity = (val.abs() / max_energy).min(1.0);
            let color = energy_color(cfg.theme, intensity);
            root.draw(&Rectangle::new(
                [
                    ((col as f64 * cell_w) as i32, y0 as i32),
                    (((col as f64 + 1.0) * cell_w) as i32, (y0 + row_h) as i32),
                ],
                color.filled(),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
        }

        let scale_label = format!("Scale {}", row_idx);
        root.draw(&Text::new(
            scale_label,
            (5, (y0 + row_h / 2.0) as i32),
            (LABEL_FONT, 10)
                .into_font()
                .color(&cfg.theme.text().mix(0.7)),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &WaveletConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &WaveletConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 200, 1, 100.0);
        let cfg = WaveletConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_wavelet.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
