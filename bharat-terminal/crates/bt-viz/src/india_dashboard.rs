// crates/bt-viz/src/india_dashboard.rs
// Author: Sourish Dey

//! Customizable India dashboard (multi-widget overview). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct IndiaDashboardConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for IndiaDashboardConfig {
    fn default() -> Self {
        Self {
            title: "India Dashboard".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl IndiaDashboardConfig {
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

fn sample_widgets() -> Vec<(String, String, String)> {
    vec![
        (
            "Nifty 50".to_string(),
            "24,812".to_string(),
            "+0.8%".to_string(),
        ),
        (
            "Sensex".to_string(),
            "81,450".to_string(),
            "+0.6%".to_string(),
        ),
        (
            "USD/INR".to_string(),
            "83.25".to_string(),
            "-0.1%".to_string(),
        ),
        (
            "10Y G-Sec".to_string(),
            "7.05%".to_string(),
            "+2bp".to_string(),
        ),
        (
            "Crude Oil".to_string(),
            "$84.50".to_string(),
            "-0.8%".to_string(),
        ),
        (
            "Gold".to_string(),
            "72,500".to_string(),
            "+0.5%".to_string(),
        ),
        (
            "FII Flow".to_string(),
            "+2,100 Cr".to_string(),
            "Net Buy".to_string(),
        ),
        (
            "DII Flow".to_string(),
            "+1,450 Cr".to_string(),
            "Net Buy".to_string(),
        ),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &IndiaDashboardConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_widgets();
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

    for (i, (name, value, change)) in data.iter().enumerate() {
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

        root.draw(&Text::new(
            value.clone(),
            (x as i32 + 10, y as i32 + ch as i32 / 2 - 4),
            (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        let change_color = if change.starts_with('+') || change == "Net Buy" {
            cfg.theme.profit()
        } else if change.starts_with('-') {
            cfg.theme.loss()
        } else {
            cfg.theme.accent()
        };
        root.draw(&Text::new(
            change.clone(),
            (x as i32 + 10, y as i32 + ch as i32 - 20),
            (LABEL_FONT, 11).into_font().color(&change_color),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &IndiaDashboardConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &IndiaDashboardConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("INDIA", 100, 1, 100.0);
        let cfg = IndiaDashboardConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_india_dashboard.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
