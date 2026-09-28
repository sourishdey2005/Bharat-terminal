// crates/bt-viz/src/money_market.rs
// Author: Sourish Dey

//! Money market rates dashboard (MCP, T-Bill, CD, CP, MIBOR). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct MoneyMarketConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for MoneyMarketConfig {
    fn default() -> Self {
        Self { title: "Money Market Rates".to_string(), theme: Theme::Dark }
    }
}

impl MoneyMarketConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
}

fn sample_rates() -> Vec<(String, f64, f64)> {
    vec![
        ("MCP 1M".to_string(), 6.55, 6.60),
        ("MCP 3M".to_string(), 6.62, 6.68),
        ("T-Bill 91D".to_string(), 6.45, 6.50),
        ("T-Bill 364D".to_string(), 6.71, 6.75),
        ("CD 3M".to_string(), 6.85, 6.95),
        ("CD 6M".to_string(), 7.05, 7.15),
        ("CP 3M".to_string(), 6.95, 7.05),
        ("CP 6M".to_string(), 7.15, 7.25),
        ("MIBOR ON".to_string(), 6.65, 6.70),
        ("MIBOR 1M".to_string(), 6.72, 6.78),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &MoneyMarketConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_rates();
    let (w, h) = root.dim_in_pixel();

    root.draw(&Text::new(
        format!("{} — {}", cfg.title, series.symbol),
        (10, 10),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let row_h = (h as i32 - 100) / (data.len() as i32 + 1);
    let col_rate = w as i32 * 2 / 3;

    root.draw(&Text::new("Instrument".to_string(), (10, 45), (LABEL_FONT, 12).into_font().color(&cfg.theme.accent())))
        .map_err(|e| BtError::Render(e.to_string()))?;
    root.draw(&Text::new("Bid".to_string(), (col_rate - 60, 45), (LABEL_FONT, 12).into_font().color(&cfg.theme.accent())))
        .map_err(|e| BtError::Render(e.to_string()))?;
    root.draw(&Text::new("Offer".to_string(), (col_rate + 20, 45), (LABEL_FONT, 12).into_font().color(&cfg.theme.accent())))
        .map_err(|e| BtError::Render(e.to_string()))?;
    root.draw(&Text::new("Spread".to_string(), (col_rate + 100, 45), (LABEL_FONT, 12).into_font().color(&cfg.theme.accent())))
        .map_err(|e| BtError::Render(e.to_string()))?;

    for (i, (name, bid, offer)) in data.iter().enumerate() {
        let y = 45 + (i as i32 + 1) * row_h;
        root.draw(&Text::new(name.clone(), (10, y + row_h / 2 - 6), (LABEL_FONT, 12).into_font().color(&cfg.theme.text())))
            .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(format!("{:.2}", bid), (col_rate - 60, y + row_h / 2 - 6), (LABEL_FONT, 12).into_font().color(&cfg.theme.profit())))
            .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(format!("{:.2}", offer), (col_rate + 20, y + row_h / 2 - 6), (LABEL_FONT, 12).into_font().color(&cfg.theme.loss())))
            .map_err(|e| BtError::Render(e.to_string()))?;
        let spread = offer - bid;
        root.draw(&Text::new(format!("{:.0}bp", spread * 100.0), (col_rate + 100, y + row_h / 2 - 6), (LABEL_FONT, 12).into_font().color(&cfg.theme.info())))
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

pub fn render_png(series: &OhlcvSeries, cfg: &MoneyMarketConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &MoneyMarketConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("MM", 100, 1, 100.0);
        let cfg = MoneyMarketConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_money_market.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
