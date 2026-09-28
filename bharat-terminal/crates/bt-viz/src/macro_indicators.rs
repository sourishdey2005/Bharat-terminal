// crates/bt-viz/src/macro_indicators.rs
// Author: Sourish Dey

//! Indian macro indicators dashboard (GDP, CPI, IIP, CAD, fiscal). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct MacroIndicatorsConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for MacroIndicatorsConfig {
    fn default() -> Self {
        Self {
            title: "India Macro Indicators".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl MacroIndicatorsConfig {
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

fn sample_macro() -> Vec<(String, f64, String)> {
    vec![
        ("GDP Growth".to_string(), 7.8, "YoY %".to_string()),
        ("CPI Inflation".to_string(), 5.1, "YoY %".to_string()),
        ("Core CPI".to_string(), 4.2, "YoY %".to_string()),
        ("IIP Growth".to_string(), 4.9, "YoY %".to_string()),
        ("CAD/GDP".to_string(), -1.3, "% of GDP".to_string()),
        ("Fiscal Def/GDP".to_string(), -5.6, "% of GDP".to_string()),
        ("Unemployment".to_string(), 7.2, "%".to_string()),
        ("GST Revenue".to_string(), 1.82, "L Cr/mo".to_string()),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &MacroIndicatorsConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_macro();
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

    for (i, (name, value, unit)) in data.iter().enumerate() {
        let col = i % cols;
        let row = i / cols;
        let x = col as f64 * cell_w + 10.0;
        let y = 50.0 + row as f64 * cell_h + 10.0;
        let cw = cell_w - 20.0;
        let ch = cell_h - 20.0;

        root.draw(&Rectangle::new(
            [(x as i32, y as i32), ((x + cw) as i32, (y + ch) as i32)],
            cfg.theme.border().filled(),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Rectangle::new(
            [(x as i32, y as i32), ((x + cw) as i32, (y + ch) as i32)],
            cfg.theme.border().stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        root.draw(&Text::new(
            name.clone(),
            (x as i32 + 10, y as i32 + 12),
            (LABEL_FONT, 12)
                .into_font()
                .color(&cfg.theme.text().mix(0.8)),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        let color = if *value < 0.0 {
            cfg.theme.loss()
        } else {
            cfg.theme.profit()
        };
        root.draw(&Text::new(
            format!("{:.2}", value),
            (x as i32 + 10, y as i32 + ch as i32 / 2 - 4),
            (TITLE_FONT, 22).into_font().color(&color),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        root.draw(&Text::new(
            unit.clone(),
            (x as i32 + 10, y as i32 + ch as i32 - 20),
            (LABEL_FONT, 10)
                .into_font()
                .color(&cfg.theme.text().mix(0.6)),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &MacroIndicatorsConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &MacroIndicatorsConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("INDIA", 100, 1, 100.0);
        let cfg = MacroIndicatorsConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_macro_indicators.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
