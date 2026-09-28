// crates/bt-viz/src/return_heatmap.rs
// Author: Sourish Dey

//! Return correlation heatmap. Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

/// Return series for heatmap.
#[derive(Debug, Clone)]
pub struct ReturnSeries {
    pub symbol: String,
    pub returns: Vec<f64>,
}

/// Sample return series data.
fn sample_returns() -> Vec<ReturnSeries> {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    let mut rng = StdRng::seed_from_u64(99);
    let n = 100;

    let factor: Vec<f64> = (0..n).map(|_| rng.gen_range(-1.0..1.0) * 0.01).collect();

    let specs = [
        ("NIFTY", 0.8),
        ("SENSEX", 0.75),
        ("BANKNIFTY", 0.7),
        ("GOLD", 0.1),
        ("CRUDE", 0.15),
        ("USDINR", 0.05),
        ("US10Y", -0.3),
        ("BTC", 0.3),
    ];

    specs
        .into_iter()
        .enumerate()
        .map(|(idx, (sym, beta))| {
            let returns: Vec<f64> = (0..n)
                .map(|i| beta * factor[i] + rng.gen_range(-1.0..1.0) * 0.006)
                .collect();
            ReturnSeries {
                symbol: sym.to_string(),
                returns,
            }
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct ReturnHeatmapConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for ReturnHeatmapConfig {
    fn default() -> Self {
        Self {
            title: "Return Correlation Heatmap".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl ReturnHeatmapConfig {
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

fn pearson(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len() as f64;
    let mean_a = a.iter().sum::<f64>() / n;
    let mean_b = b.iter().sum::<f64>() / n;
    let mut cov = 0.0;
    let mut var_a = 0.0;
    let mut var_b = 0.0;
    for i in 0..a.len() {
        let da = a[i] - mean_a;
        let db = b[i] - mean_b;
        cov += da * db;
        var_a += da * da;
        var_b += db * db;
    }
    if var_a <= 0.0 || var_b <= 0.0 {
        0.0
    } else {
        cov / (var_a.sqrt() * var_b.sqrt())
    }
}

fn lerp(a: u8, b: u8, t: f64) -> u8 {
    (a as f64 + (b as f64 - a as f64) * t.clamp(0.0, 1.0)).round() as u8
}

fn corr_color(theme: Theme, v: f64) -> RGBColor {
    let v = v.clamp(-1.0, 1.0);
    let bg = theme.background();
    let target = if v >= 0.0 { theme.profit() } else { theme.loss() };
    let t = v.abs();
    RGBColor(
        lerp(bg.0, target.0, t),
        lerp(bg.1, target.1, t),
        lerp(bg.2, target.2, t),
    )
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    data: &[ReturnSeries],
    cfg: &ReturnHeatmapConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if data.len() < 2 {
        return Err(BtError::InvalidInput("need at least 2 series".into()));
    }
    fill_background(&root, cfg.theme)?;

    let n = data.len();
    let (w, h) = root.dim_in_pixel();
    let top_pad = 50;
    let bottom_pad = 50;
    let side_pad = 100;
    let grid_w = w as f64 - 2.0 * side_pad as f64;
    let grid_h = h as f64 - top_pad as f64 - bottom_pad as f64;
    let cell_w = grid_w / n as f64;
    let cell_h = grid_h / n as f64;

    root.draw(&Text::new(
        &cfg.title,
        (w as i32 / 2, 20),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    for i in 0..n {
        for j in 0..n {
            let corr = if i == j {
                1.0
            } else {
                pearson(&data[i].returns, &data[j].returns)
            };

            let x = side_pad as f64 + j as f64 * cell_w;
            let y = top_pad as f64 + i as f64 * cell_h;

            root.draw(&Rectangle::new(
                [x as i32, y as i32, (x + cell_w) as i32, (y + cell_h) as i32],
                corr_color(cfg.theme, corr).filled(),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;

            root.draw(&Rectangle::new(
                [x as i32, y as i32, (x + cell_w) as i32, (y + cell_h) as i32],
                cfg.theme.border().stroke_width(1),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;

            root.draw(&Text::new(
                format!("{:.2}", corr),
                (x as i32 + 5, y as i32 + 10),
                (LABEL_FONT, 11).into_font().color(&cfg.theme.text()),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    // Labels
    for (i, series) in data.iter().enumerate() {
        root.draw(&Text::new(
            &series.symbol,
            (
                side_pad as i32 + i as i32 * cell_w as i32 + 5,
                top_pad as i32 - 15,
            ),
            (LABEL_FONT, 11).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        root.draw(&Text::new(
            &series.symbol,
            (
                side_pad as i32 - 50,
                top_pad as i32 + i as i32 * cell_h as i32 + 10,
            ),
            (LABEL_FONT, 11).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(data: &[ReturnSeries], cfg: &ReturnHeatmapConfig, path: &str) -> Result<()> {
    render(png_root(path)?, data, cfg)
}

pub fn render_svg(data: &[ReturnSeries], cfg: &ReturnHeatmapConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, data, cfg)
}

pub fn render_sample_png(cfg: &ReturnHeatmapConfig, path: &str) -> Result<()> {
    let data = sample_returns();
    render_png(&data, cfg, path)
}

pub fn render_sample_svg(cfg: &ReturnHeatmapConfig, path: &str) -> Result<()> {
    let data = sample_returns();
    render_svg(&data, cfg, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let data = sample_returns();
        let cfg = ReturnHeatmapConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_return_heatmap.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&data, &cfg, &path).unwrap();
    }
}
