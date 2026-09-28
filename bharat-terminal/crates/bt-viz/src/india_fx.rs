// crates/bt-viz/src/india_fx.rs
// Author: Sourish Dey

//! India FX dashboard (USD/INR, EUR/INR, GBP/INR, JPY/INR). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct IndiaFxConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for IndiaFxConfig {
    fn default() -> Self {
        Self { title: "India FX Dashboard".to_string(), theme: Theme::Dark }
    }
}

impl IndiaFxConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
}

fn sample_fx() -> Vec<(String, f64, f64, f64)> {
    vec![
        ("USD/INR".to_string(), 83.25, 83.30, -0.12),
        ("EUR/INR".to_string(), 90.15, 90.22, 0.08),
        ("GBP/INR".to_string(), 105.80, 105.95, 0.15),
        ("JPY/INR".to_string(), 0.558, 0.559, -0.05),
        ("AUD/INR".to_string(), 54.20, 54.35, 0.10),
        ("CAD/INR".to_string(), 61.50, 61.62, -0.06),
        ("SGD/INR".to_string(), 62.10, 62.18, 0.04),
        ("CHF/INR".to_string(), 93.40, 93.55, 0.12),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &IndiaFxConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_fx();
    let (w, h) = root.dim_in_pixel();

    root.draw(&Text::new(
        format!("{} — {}", cfg.title, series.symbol),
        (10, 10),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let row_h = (h as i32 - 100) / (data.len() as i32 + 1);
    let col_px = [10, 250, 450, 650, 850];

    for (x, label) in col_px.iter().zip(["Pair", "Bid", "Offer", "Change %", "Trend"].iter()) {
        root.draw(&Text::new(label.to_string(), ( *x, 45), (LABEL_FONT, 12).into_font().color(&cfg.theme.accent())))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    for (i, (pair, bid, offer, change)) in data.iter().enumerate() {
        let y = 45 + (i as i32 + 1) * row_h;
        root.draw(&Text::new(pair.clone(), ( col_px[0], y + row_h / 2 - 6), (LABEL_FONT, 12).into_font().color(&cfg.theme.text())))
            .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(format!("{:.4}", bid), ( col_px[1], y + row_h / 2 - 6), (LABEL_FONT, 12).into_font().color(&cfg.theme.profit())))
            .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(format!("{:.4}", offer), ( col_px[2], y + row_h / 2 - 6), (LABEL_FONT, 12).into_font().color(&cfg.theme.loss())))
            .map_err(|e| BtError::Render(e.to_string()))?;
        let color = if *change >= 0.0 { cfg.theme.profit() } else { cfg.theme.loss() };
        root.draw(&Text::new(format!("{:+.2}%", change), ( col_px[3], y + row_h / 2 - 6), (LABEL_FONT, 12).into_font().color(&color)))
            .map_err(|e| BtError::Render(e.to_string()))?;
        let bars = (change.abs() * 30.0) as i32;
        for b in 0..bars.max(1) {
            root.draw(&Rectangle::new(
                [(col_px[4] + b * 8, y + row_h / 2 - 8), (col_px[4] + b * 8 + 6, y + row_h / 2 + 4)],
                color.filled(),
            ))
    .map_err(|e| BtError::Render(e.to_string()))?;
        }
        root.draw(&PathElement::new(
            vec![(0, y + row_h), (w as i32, y + row_h)],
            cfg.theme.border().stroke_width(1),
        ))
    .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &IndiaFxConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &IndiaFxConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("FX", 100, 1, 100.0);
        let cfg = IndiaFxConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_india_fx.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
