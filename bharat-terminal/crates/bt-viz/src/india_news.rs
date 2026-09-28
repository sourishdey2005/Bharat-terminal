// crates/bt-viz/src/india_news.rs
// Author: Sourish Dey

//! India news feed (market-moving headlines). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct IndiaNewsConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for IndiaNewsConfig {
    fn default() -> Self {
        Self { title: "India News Feed".to_string(), theme: Theme::Dark }
    }
}

impl IndiaNewsConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
}

fn sample_news() -> Vec<(String, String, String)> {
    vec![
        ("RBI holds repo rate at 6.50%".to_string(), "RBI".to_string(), "Neutral".to_string()),
        ("GST collections rise 12% YoY".to_string(), "Govt".to_string(), "Positive".to_string()),
        ("FII inflows hit 6-month high".to_string(), "Market".to_string(), "Positive".to_string()),
        ("Crude oil falls below $85".to_string(), "Commodity".to_string(), "Positive".to_string()),
        ("Rupee strengthens to 83.10".to_string(), "FX".to_string(), "Positive".to_string()),
        ("IT sector faces H-1B visa concerns".to_string(), "Sector".to_string(), "Negative".to_string()),
        ("Auto sales surge in festive season".to_string(), "Auto".to_string(), "Positive".to_string()),
        ("Inflation eases to 5.1%".to_string(), "Macro".to_string(), "Positive".to_string()),
        ("SEBI tightens F&O norms".to_string(), "Regulator".to_string(), "Negative".to_string()),
        ("Monsoon deficit narrows to 5%".to_string(), "Agri".to_string(), "Positive".to_string()),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &IndiaNewsConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_news();
    let (w, h) = root.dim_in_pixel();

    root.draw(&Text::new(
        format!("{} — {}", cfg.title, series.symbol),
        (10, 10),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let row_h = (h as i32 - 100) / (data.len() as i32 + 1);
    let col_px = [10, 700, 950];

    for (x, label) in col_px.iter().zip(["Headline", "Source", "Sentiment"].iter()) {
        root.draw(&Text::new(label.to_string(), ( *x, 45), (LABEL_FONT, 12).into_font().color(&cfg.theme.accent())))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    for (i, (headline, source, sentiment)) in data.iter().enumerate() {
        let y = 45 + (i as i32 + 1) * row_h;
        let color = match sentiment.as_str() {
            "Positive" => cfg.theme.profit(),
            "Negative" => cfg.theme.loss(),
            _ => cfg.theme.accent(),
        };
        root.draw(&Text::new(headline.clone(), ( col_px[0], y + row_h / 2 - 6), (LABEL_FONT, 12).into_font().color(&cfg.theme.text())))
            .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(source.clone(), ( col_px[1], y + row_h / 2 - 6), (LABEL_FONT, 12).into_font().color(&cfg.theme.text().mix(0.7))))
            .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(sentiment.clone(), ( col_px[2], y + row_h / 2 - 6), (LABEL_FONT, 12).into_font().color(&color)))
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

pub fn render_png(series: &OhlcvSeries, cfg: &IndiaNewsConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &IndiaNewsConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("NEWS", 100, 1, 100.0);
        let cfg = IndiaNewsConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_india_news.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
