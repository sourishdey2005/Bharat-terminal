// crates/bt-viz/src/india_macro.rs
// Author: Sourish Dey

//! India macro dashboard (GDP, inflation, trade, monsoon, PMI). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct IndiaMacroConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for IndiaMacroConfig {
    fn default() -> Self {
        Self {
            title: "India Macro Dashboard".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl IndiaMacroConfig {
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

fn sample_macro() -> Vec<(String, f64, f64, String)> {
    vec![
        ("GDP Growth".to_string(), 7.8, 7.2, "YoY %".to_string()),
        ("CPI Inflation".to_string(), 5.1, 5.5, "YoY %".to_string()),
        ("Core CPI".to_string(), 4.2, 4.5, "YoY %".to_string()),
        ("IIP Growth".to_string(), 4.9, 4.2, "YoY %".to_string()),
        (
            "Trade Deficit".to_string(),
            -18.5,
            -20.2,
            "USD Bn".to_string(),
        ),
        ("Fiscal Def".to_string(), -5.6, -5.9, "% GDP".to_string()),
        ("PMI Mfg".to_string(), 57.5, 56.8, "Index".to_string()),
        ("PMI Svcs".to_string(), 60.5, 59.2, "Index".to_string()),
        ("Monsoon".to_string(), 95.0, 100.0, "% Normal".to_string()),
        ("Unemployment".to_string(), 7.2, 7.8, "%".to_string()),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &IndiaMacroConfig,
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

    let row_h = (h as i32 - 100) / (data.len() as i32 + 1);
    let col_px = [10, 280, 480, 680, 880];

    for (x, label) in col_px
        .iter()
        .zip(["Indicator", "Current", "Previous", "Unit", "Signal"].iter())
    {
        root.draw(&Text::new(
            label.to_string(),
            (*x, 45),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.accent()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    for (i, (name, cur, prev, unit)) in data.iter().enumerate() {
        let y = 45 + (i as i32 + 1) * row_h;
        root.draw(&Text::new(
            name.clone(),
            (col_px[0], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            format!("{:.1}", cur),
            (col_px[1], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            format!("{:.1}", prev),
            (col_px[2], y + row_h / 2 - 6),
            (LABEL_FONT, 12)
                .into_font()
                .color(&cfg.theme.text().mix(0.7)),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            unit.clone(),
            (col_px[3], y + row_h / 2 - 6),
            (LABEL_FONT, 12)
                .into_font()
                .color(&cfg.theme.text().mix(0.7)),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        let improving = if name.contains("Deficit")
            || name.contains("Unemployment")
            || name.contains("Inflation")
        {
            cur < prev
        } else {
            cur > prev
        };
        let signal = if improving { "Improving" } else { "Worsening" };
        let sig_color = if improving {
            cfg.theme.profit()
        } else {
            cfg.theme.loss()
        };
        root.draw(&Text::new(
            signal.to_string(),
            (col_px[4], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&sig_color),
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

pub fn render_png(series: &OhlcvSeries, cfg: &IndiaMacroConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &IndiaMacroConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("MACRO", 100, 1, 100.0);
        let cfg = IndiaMacroConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_india_macro.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
