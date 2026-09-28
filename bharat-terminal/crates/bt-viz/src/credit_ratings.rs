// crates/bt-viz/src/credit_ratings.rs
// Author: Sourish Dey

//! Credit ratings dashboard (CRISIL/ICRA/CARE ratings). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CreditRatingsConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for CreditRatingsConfig {
    fn default() -> Self {
        Self {
            title: "Credit Ratings".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl CreditRatingsConfig {
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

fn sample_ratings() -> Vec<(String, String, String, f64)> {
    vec![
        (
            "SBI".to_string(),
            "CRISIL".to_string(),
            "AAA/Stable".to_string(),
            0.95,
        ),
        (
            "HDFC Bank".to_string(),
            "ICRA".to_string(),
            "AAA/Stable".to_string(),
            0.94,
        ),
        (
            "Reliance".to_string(),
            "CARE".to_string(),
            "AAA/Stable".to_string(),
            0.92,
        ),
        (
            "Tata Steel".to_string(),
            "CRISIL".to_string(),
            "AA/Positive".to_string(),
            0.82,
        ),
        (
            "Adani Ports".to_string(),
            "ICRA".to_string(),
            "AA+/Stable".to_string(),
            0.85,
        ),
        (
            "JSW Steel".to_string(),
            "CARE".to_string(),
            "AA/Stable".to_string(),
            0.80,
        ),
        (
            "BHEL".to_string(),
            "CRISIL".to_string(),
            "A/Negative".to_string(),
            0.55,
        ),
        (
            "Jet Airways".to_string(),
            "ICRA".to_string(),
            "D".to_string(),
            0.10,
        ),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &CreditRatingsConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_ratings();
    let (w, h) = root.dim_in_pixel();

    root.draw(&Text::new(
        format!("{} — {}", cfg.title, series.symbol),
        (10, 10),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let row_h = (h as i32 - 100) / (data.len() as i32 + 1);
    let col_px = [10, 220, 400, 620, 850];

    for (x, label) in col_px
        .iter()
        .zip(["Issuer", "Agency", "Rating", "Score", "Bar"].iter())
    {
        root.draw(&Text::new(
            label.to_string(),
            (*x, 45),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.accent()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    for (i, (name, agency, rating, score)) in data.iter().enumerate() {
        let y = 45 + (i as i32 + 1) * row_h;
        root.draw(&Text::new(
            name.clone(),
            (col_px[0], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            agency.clone(),
            (col_px[1], y + row_h / 2 - 6),
            (LABEL_FONT, 12)
                .into_font()
                .color(&cfg.theme.text().mix(0.8)),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        let color = if *score >= 0.8 {
            cfg.theme.profit()
        } else if *score >= 0.5 {
            cfg.theme.accent()
        } else {
            cfg.theme.loss()
        };
        root.draw(&Text::new(
            rating.clone(),
            (col_px[2], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&color),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            format!("{:.0}%", score * 100.0),
            (col_px[3], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        let bar_w = (score * 250.0) as i32;
        root.draw(&Rectangle::new(
            [
                (col_px[4], y + row_h / 2 - 8),
                (col_px[4] + bar_w, y + row_h / 2 + 4),
            ],
            color.filled(),
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

pub fn render_png(series: &OhlcvSeries, cfg: &CreditRatingsConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &CreditRatingsConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("CR", 100, 1, 100.0);
        let cfg = CreditRatingsConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_credit_ratings.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
