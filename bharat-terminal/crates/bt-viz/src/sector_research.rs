// crates/bt-viz/src/sector_research.rs
// Author: Sourish Dey

//! Sector research summary (P/E, growth, outlook). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct SectorResearchConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for SectorResearchConfig {
    fn default() -> Self {
        Self {
            title: "Sector Research".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl SectorResearchConfig {
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

fn sample_research() -> Vec<(String, f64, f64, String)> {
    vec![
        (
            "IT Services".to_string(),
            28.5,
            12.5,
            "Overweight".to_string(),
        ),
        (
            "Private Banks".to_string(),
            22.3,
            15.8,
            "Overweight".to_string(),
        ),
        ("Pharma".to_string(), 24.8, 10.2, "Neutral".to_string()),
        ("Auto".to_string(), 18.5, 14.2, "Overweight".to_string()),
        ("Metals".to_string(), 12.2, 8.5, "Underweight".to_string()),
        ("Realty".to_string(), 35.2, 18.5, "Overweight".to_string()),
        ("FMCG".to_string(), 45.8, 9.8, "Neutral".to_string()),
        ("Infra".to_string(), 15.5, 16.5, "Overweight".to_string()),
        ("Media".to_string(), 32.5, 11.2, "Neutral".to_string()),
        ("PSU Banks".to_string(), 8.5, 12.8, "Overweight".to_string()),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &SectorResearchConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_research();
    let (w, h) = root.dim_in_pixel();

    root.draw(&Text::new(
        format!("{} — {}", cfg.title, series.symbol),
        (10, 10),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let row_h = (h as i32 - 100) / (data.len() as i32 + 1);
    let col_px = [10, 250, 450, 650, 850];

    for (x, label) in col_px
        .iter()
        .zip(["Sector", "P/E", "EPS Growth", "Outlook", "Signal"].iter())
    {
        root.draw(&Text::new(
            label.to_string(),
            (*x, 45),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.accent()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    for (i, (name, pe, growth, outlook)) in data.iter().enumerate() {
        let y = 45 + (i as i32 + 1) * row_h;
        root.draw(&Text::new(
            name.clone(),
            (col_px[0], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            format!("{:.1}", pe),
            (col_px[1], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            format!("{:.1}%", growth),
            (col_px[2], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        let color = match outlook.as_str() {
            "Overweight" => cfg.theme.profit(),
            "Underweight" => cfg.theme.loss(),
            _ => cfg.theme.accent(),
        };
        root.draw(&Text::new(
            outlook.clone(),
            (col_px[3], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&color),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        let signal = if outlook == "Overweight" {
            "BUY"
        } else if outlook == "Underweight" {
            "SELL"
        } else {
            "HOLD"
        };
        root.draw(&Text::new(
            signal.to_string(),
            (col_px[4], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&color),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
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

pub fn render_png(series: &OhlcvSeries, cfg: &SectorResearchConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &SectorResearchConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("SR", 100, 1, 100.0);
        let cfg = SectorResearchConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_sector_research.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
