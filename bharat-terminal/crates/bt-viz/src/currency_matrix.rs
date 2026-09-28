// crates/bt-viz/src/currency_matrix.rs
// Author: Sourish Dey

//! Currency cross-rates heatmap. Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

/// Currency cross-rate data.
#[derive(Debug, Clone)]
pub struct CurrencyPair {
    pub base: String,
    pub quote: String,
    pub rate: f64,
    pub change_pct: f64,
}

/// Sample currency matrix data.
fn sample_currency() -> Vec<CurrencyPair> {
    let bases = ["USD", "EUR", "GBP", "JPY", "INR", "AUD"];
    let quotes = ["USD", "EUR", "GBP", "JPY", "INR", "AUD"];
    let rates: [[f64; 6]; 6] = [
        [1.0, 0.92, 0.79, 149.5, 83.4, 1.52],
        [1.09, 1.0, 0.86, 162.8, 90.7, 1.65],
        [1.27, 1.16, 1.0, 189.2, 105.6, 1.92],
        [0.0067, 0.0061, 0.0053, 1.0, 0.557, 0.0102],
        [0.012, 0.011, 0.0095, 1.795, 1.0, 0.0182],
        [0.66, 0.61, 0.52, 98.0, 54.9, 1.0],
    ];
    let changes: [[f64; 6]; 6] = [
        [0.0, 0.12, -0.08, 0.35, 0.15, -0.22],
        [-0.12, 0.0, 0.05, 0.28, 0.10, -0.18],
        [0.08, -0.05, 0.0, 0.42, 0.20, -0.15],
        [-0.35, -0.28, -0.42, 0.0, -0.15, 0.30],
        [-0.15, -0.10, -0.20, 0.15, 0.0, 0.12],
        [0.22, 0.18, 0.15, -0.30, -0.12, 0.0],
    ];

    let mut data = Vec::new();
    for (i, base) in bases.iter().enumerate() {
        for (j, quote) in quotes.iter().enumerate() {
            data.push(CurrencyPair {
                base: base.to_string(),
                quote: quote.to_string(),
                rate: rates[i][j],
                change_pct: changes[i][j],
            });
        }
    }
    data
}

#[derive(Debug, Clone)]
pub struct CurrencyMatrixConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for CurrencyMatrixConfig {
    fn default() -> Self {
        Self {
            title: "Currency Cross Rates".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl CurrencyMatrixConfig {
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

fn lerp(a: u8, b: u8, t: f64) -> u8 {
    (a as f64 + (b as f64 - a as f64) * t.clamp(0.0, 1.0)).round() as u8
}

fn change_color(theme: Theme, v: f64) -> RGBColor {
    let bg = theme.background();
    let target = if v >= 0.0 {
        theme.profit()
    } else {
        theme.loss()
    };
    let t = v.abs().min(1.0);
    RGBColor(
        lerp(bg.0, target.0, t),
        lerp(bg.1, target.1, t),
        lerp(bg.2, target.2, t),
    )
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    data: &[CurrencyPair],
    cfg: &CurrencyMatrixConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if data.is_empty() {
        return Err(BtError::EmptySeries("currency data".into()));
    }
    fill_background(&root, cfg.theme)?;

    let (w, h) = root.dim_in_pixel();
    let top_pad = 50;
    let bottom_pad = 50;
    let side_pad = 80;
    let grid_w = w as f64 - 2.0 * side_pad as f64;
    let grid_h = h as f64 - top_pad as f64 - bottom_pad as f64;

    root.draw(&Text::new(
        cfg.title.as_str(),
        (w as i32 / 2, 20),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let n = 6;
    let cell_w = grid_w / n as f64;
    let cell_h = grid_h / n as f64;

    let max_abs_change = data
        .iter()
        .map(|d| d.change_pct.abs())
        .fold(0.0_f64, f64::max)
        .max(0.01);

    for pair in data {
        let i = match pair.base.as_str() {
            "USD" => 0,
            "EUR" => 1,
            "GBP" => 2,
            "JPY" => 3,
            "INR" => 4,
            "AUD" => 5,
            _ => continue,
        };
        let j = match pair.quote.as_str() {
            "USD" => 0,
            "EUR" => 1,
            "GBP" => 2,
            "JPY" => 3,
            "INR" => 4,
            "AUD" => 5,
            _ => continue,
        };

        let x = side_pad as f64 + j as f64 * cell_w;
        let y = top_pad as f64 + i as f64 * cell_h;

        let color = change_color(cfg.theme, pair.change_pct / max_abs_change);
        root.draw(&Rectangle::new(
            [
                (x as i32, y as i32),
                ((x + cell_w) as i32, (y + cell_h) as i32),
            ],
            color.filled(),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        root.draw(&Rectangle::new(
            [
                (x as i32, y as i32),
                ((x + cell_w) as i32, (y + cell_h) as i32),
            ],
            cfg.theme.border().stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        // Rate
        root.draw(&Text::new(
            format!("{:.4}", pair.rate),
            (x as i32 + 5, y as i32 + 10),
            (LABEL_FONT, 11).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        // Change
        root.draw(&Text::new(
            format!("{:+.2}%", pair.change_pct),
            (x as i32 + 5, y as i32 + 26),
            (LABEL_FONT, 10)
                .into_font()
                .color(&cfg.theme.text().mix(0.7)),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    // Row/column labels
    let labels = ["USD", "EUR", "GBP", "JPY", "INR", "AUD"];
    for (i, label) in labels.iter().enumerate() {
        root.draw(&Text::new(
            *label,
            (
                side_pad as i32 - 30,
                (top_pad as f64 + i as f64 * cell_h + cell_h / 2.0) as i32,
            ),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        root.draw(&Text::new(
            *label,
            (
                side_pad as i32 + i as i32 * cell_w as i32 + 10,
                top_pad as i32 - 15,
            ),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(data: &[CurrencyPair], cfg: &CurrencyMatrixConfig, path: &str) -> Result<()> {
    render(png_root(path)?, data, cfg)
}

pub fn render_svg(data: &[CurrencyPair], cfg: &CurrencyMatrixConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, data, cfg)
}

pub fn render_sample_png(cfg: &CurrencyMatrixConfig, path: &str) -> Result<()> {
    let data = sample_currency();
    render_png(&data, cfg, path)
}

pub fn render_sample_svg(cfg: &CurrencyMatrixConfig, path: &str) -> Result<()> {
    let data = sample_currency();
    render_svg(&data, cfg, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let data = sample_currency();
        let cfg = CurrencyMatrixConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_currency_matrix.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&data, &cfg, &path).unwrap();
    }
}
