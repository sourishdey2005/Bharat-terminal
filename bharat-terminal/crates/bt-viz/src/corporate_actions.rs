// crates/bt-viz/src/corporate_actions.rs
// Author: Sourish Dey

//! Corporate actions calendar (dividends, bonuses, splits, buybacks). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CorporateActionsConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for CorporateActionsConfig {
    fn default() -> Self {
        Self {
            title: "Corporate Actions".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl CorporateActionsConfig {
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

fn sample_actions() -> Vec<(String, String, String, String)> {
    vec![
        (
            "RELIANCE".to_string(),
            "Dividend".to_string(),
            "Rs 10/sh".to_string(),
            "15-Oct".to_string(),
        ),
        (
            "TCS".to_string(),
            "Bonus".to_string(),
            "1:1".to_string(),
            "20-Oct".to_string(),
        ),
        (
            "INFY".to_string(),
            "Dividend".to_string(),
            "Rs 18/sh".to_string(),
            "25-Oct".to_string(),
        ),
        (
            "HDFCBANK".to_string(),
            "Split".to_string(),
            "1:5".to_string(),
            "01-Nov".to_string(),
        ),
        (
            "SBIN".to_string(),
            "Buyback".to_string(),
            "Rs 21,000 Cr".to_string(),
            "10-Nov".to_string(),
        ),
        (
            "ITC".to_string(),
            "Dividend".to_string(),
            "Rs 6.25/sh".to_string(),
            "15-Nov".to_string(),
        ),
        (
            "WIPRO".to_string(),
            "Dividend".to_string(),
            "Rs 1/sh".to_string(),
            "20-Nov".to_string(),
        ),
        (
            "TATAMOTORS".to_string(),
            "Rights".to_string(),
            "1:9".to_string(),
            "01-Dec".to_string(),
        ),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &CorporateActionsConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_actions();
    let (w, h) = root.dim_in_pixel();

    root.draw(&Text::new(
        format!("{} — {}", cfg.title, series.symbol),
        (10, 10),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let row_h = (h as i32 - 100) / (data.len() as i32 + 1);
    let col_px = [10, 220, 420, 650, 880];

    for (x, label) in col_px
        .iter()
        .zip(["Symbol", "Action", "Detail", "Ex-Date", "Type"].iter())
    {
        root.draw(&Text::new(
            label.to_string(),
            (*x, 45),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.accent()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    for (i, (symbol, action, detail, date)) in data.iter().enumerate() {
        let y = 45 + (i as i32 + 1) * row_h;
        root.draw(&Text::new(
            symbol.clone(),
            (col_px[0], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        let color = match action.as_str() {
            "Dividend" => cfg.theme.profit(),
            "Bonus" => cfg.theme.info(),
            "Split" => cfg.theme.accent(),
            "Buyback" => cfg.theme.loss(),
            _ => cfg.theme.text(),
        };
        root.draw(&Text::new(
            action.clone(),
            (col_px[1], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&color),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            detail.clone(),
            (col_px[2], y + row_h / 2 - 6),
            (LABEL_FONT, 12)
                .into_font()
                .color(&cfg.theme.text().mix(0.8)),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            date.clone(),
            (col_px[3], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            action.clone(),
            (col_px[4], y + row_h / 2 - 6),
            (LABEL_FONT, 10)
                .into_font()
                .color(&cfg.theme.text().mix(0.6)),
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

pub fn render_png(series: &OhlcvSeries, cfg: &CorporateActionsConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &CorporateActionsConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("CA", 100, 1, 100.0);
        let cfg = CorporateActionsConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_corporate_actions.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
