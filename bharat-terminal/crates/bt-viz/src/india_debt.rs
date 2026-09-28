// crates/bt-viz/src/india_debt.rs
// Author: Sourish Dey

//! India debt dashboard (G-Sec, SDL, corporate bonds, T-Bills). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct IndiaDebtConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for IndiaDebtConfig {
    fn default() -> Self {
        Self {
            title: "India Debt Dashboard".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl IndiaDebtConfig {
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

fn sample_debt() -> Vec<(String, f64, f64, String)> {
    vec![
        ("91D T-Bill".to_string(), 6.45, 0.02, "Auction".to_string()),
        ("182D T-Bill".to_string(), 6.62, 0.03, "Auction".to_string()),
        ("364D T-Bill".to_string(), 6.71, 0.02, "Auction".to_string()),
        ("2Y G-Sec".to_string(), 6.78, 0.01, "Listed".to_string()),
        ("5Y G-Sec".to_string(), 6.91, 0.02, "Listed".to_string()),
        ("10Y G-Sec".to_string(), 7.05, 0.03, "Listed".to_string()),
        ("15Y G-Sec".to_string(), 7.12, 0.02, "Listed".to_string()),
        ("30Y G-Sec".to_string(), 7.24, 0.04, "Listed".to_string()),
        ("SDL 10Y".to_string(), 7.18, 0.03, "Listed".to_string()),
        ("AAA Corp".to_string(), 7.65, 0.05, "Listed".to_string()),
        ("AA Corp".to_string(), 8.15, 0.08, "Listed".to_string()),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &IndiaDebtConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_debt();
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
        .zip(["Instrument", "Yield %", "Chg (bp)", "Status", "Spread"].iter())
    {
        root.draw(&Text::new(
            label.to_string(),
            (*x, 45),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.accent()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    for (i, (name, yld, chg, status)) in data.iter().enumerate() {
        let y = 45 + (i as i32 + 1) * row_h;
        root.draw(&Text::new(
            name.clone(),
            (col_px[0], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            format!("{:.2}", yld),
            (col_px[1], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        let chg_color = if *chg > 0.03 {
            cfg.theme.loss()
        } else if *chg > 0.0 {
            cfg.theme.accent()
        } else {
            cfg.theme.profit()
        };
        root.draw(&Text::new(
            format!("{:+.0}", chg * 100.0),
            (col_px[2], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&chg_color),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            status.clone(),
            (col_px[3], y + row_h / 2 - 6),
            (LABEL_FONT, 12)
                .into_font()
                .color(&cfg.theme.text().mix(0.7)),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        let spread = ((yld - 6.45) * 100.0) as i32;
        root.draw(&Rectangle::new(
            [
                (col_px[4], y + row_h / 2 - 8),
                (col_px[4] + spread.max(1), y + row_h / 2 + 4),
            ],
            cfg.theme.info().filled(),
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

pub fn render_png(series: &OhlcvSeries, cfg: &IndiaDebtConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &IndiaDebtConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("DEBT", 100, 1, 100.0);
        let cfg = IndiaDebtConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_india_debt.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
