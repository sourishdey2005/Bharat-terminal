// crates/bt-viz/src/nse_heatmap.rs
// Author: Sourish Dey

//! NSE sector heatmap. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct NseHeatmapConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for NseHeatmapConfig {
    fn default() -> Self {
        Self {
            title: "NSE Sector Heatmap".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl NseHeatmapConfig {
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
}

fn sample_sectors() -> Vec<(String, f64, f64)> {
    vec![
        ("Nifty IT".to_string(), -0.8, 15.2),
        ("Nifty Bank".to_string(), 1.2, 28.5),
        ("Nifty FMCG".to_string(), 0.3, 12.8),
        ("Nifty Auto".to_string(), 1.8, 10.5),
        ("Nifty Pharma".to_string(), -0.5, 8.2),
        ("Nifty Metal".to_string(), 2.5, 6.8),
        ("Nifty Energy".to_string(), 0.9, 9.5),
        ("Nifty Realty".to_string(), 3.2, 4.2),
        ("Nifty Infra".to_string(), 1.5, 5.8),
        ("Nifty Media".to_string(), -1.2, 2.5),
        ("Nifty PSU Bank".to_string(), 2.8, 4.8),
        ("Nifty Cons".to_string(), 0.1, 6.2),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &NseHeatmapConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_sectors();
    let (w, h) = root.dim_in_pixel();

    root.draw(&Text::new(
        format!("{} — {}", cfg.title, series.symbol),
        (10, 10),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let cols = 4;
    let rows = (data.len() + cols - 1) / cols;
    let cell_w = w as f64 / cols as f64;
    let cell_h = (h as f64 - 100.0) / rows as f64;

    let max_abs = data
        .iter()
        .map(|(_, c, _)| c.abs())
        .fold(0.0f64, f64::max)
        .max(0.5);

    for (i, (name, change, weight)) in data.iter().enumerate() {
        let col = i % cols;
        let row = i / cols;
        let x = col as f64 * cell_w + 5.0;
        let y = 50.0 + row as f64 * cell_h + 5.0;
        let cw = cell_w - 10.0;
        let ch = cell_h - 10.0;

        let intensity = (change / max_abs).clamp(-1.0, 1.0);
        let color = if intensity >= 0.0 {
            cfg.theme.profit().mix(0.3 + 0.7 * intensity)
        } else {
            cfg.theme.loss().mix(0.3 + 0.7 * intensity.abs())
        };

        root.draw(&Rectangle::new(
            [(x as i32, y as i32), ((x + cw) as i32, (y + ch) as i32)],
            color.filled(),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Rectangle::new(
            [(x as i32, y as i32), ((x + cw) as i32, (y + ch) as i32)],
            cfg.theme.border().stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        root.draw(&Text::new(
            name.clone(),
            (x as i32 + 8, y as i32 + 10),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            format!("{:+.2}%", change),
            (x as i32 + 8, y as i32 + ch as i32 / 2 - 4),
            (TITLE_FONT, 18).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            format!("{:.1}% wt", weight),
            (x as i32 + 8, y as i32 + ch as i32 - 18),
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

pub fn render_png(series: &OhlcvSeries, cfg: &NseHeatmapConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &NseHeatmapConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("NSE", 100, 1, 100.0);
        let cfg = NseHeatmapConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_nse_heatmap.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
