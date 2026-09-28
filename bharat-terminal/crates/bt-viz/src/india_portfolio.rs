// crates/bt-viz/src/india_portfolio.rs
// Author: Sourish Dey

//! India portfolio with tax analytics (LTCG, STCG, dividend). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct IndiaPortfolioConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for IndiaPortfolioConfig {
    fn default() -> Self {
        Self {
            title: "India Portfolio".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl IndiaPortfolioConfig {
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

fn sample_portfolio() -> Vec<(String, f64, f64, f64, f64)> {
    vec![
        ("RELIANCE".to_string(), 25.0, 28.5, 1.2, 0.5),
        ("HDFCBANK".to_string(), 18.0, 19.2, 0.8, 0.3),
        ("INFY".to_string(), 15.0, 14.2, 0.6, 0.4),
        ("TCS".to_string(), 12.0, 13.5, 0.9, 0.2),
        ("ITC".to_string(), 10.0, 11.8, 1.5, 0.6),
        ("SBIN".to_string(), 8.0, 9.5, 1.1, 0.4),
        ("TATAMOTORS".to_string(), 7.0, 8.2, 0.7, 0.3),
        ("WIPRO".to_string(), 5.0, 4.8, 0.3, 0.2),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &IndiaPortfolioConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_portfolio();
    let (w, h) = root.dim_in_pixel();

    root.draw(&Text::new(
        format!("{} — {}", cfg.title, series.symbol),
        (10, 10),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let row_h = (h as i32 - 100) / (data.len() as i32 + 1);
    let col_px = [10, 200, 380, 560, 740, 920];

    for (x, label) in col_px.iter().zip(
        [
            "Symbol",
            "Invested (L)",
            "Current (L)",
            "P&L %",
            "Div (L)",
            "Tax (L)",
        ]
        .iter(),
    ) {
        root.draw(&Text::new(
            label.to_string(),
            (*x, 45),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.accent()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    for (i, (symbol, inv, cur, pnl, div)) in data.iter().enumerate() {
        let y = 45 + (i as i32 + 1) * row_h;
        root.draw(&Text::new(
            symbol.clone(),
            (col_px[0], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            format!("{:.1}", inv),
            (col_px[1], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            format!("{:.1}", cur),
            (col_px[2], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        let pnl_color = if *pnl >= 0.0 {
            cfg.theme.profit()
        } else {
            cfg.theme.loss()
        };
        root.draw(&Text::new(
            format!("{:+.1}%", pnl),
            (col_px[3], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&pnl_color),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            format!("{:.2}", div),
            (col_px[4], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.info()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        let tax = div * 0.10 + if *pnl > 0.0 { (cur - inv) * 0.15 } else { 0.0 };
        root.draw(&Text::new(
            format!("{:.2}", tax),
            (col_px[5], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.loss()),
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

pub fn render_png(series: &OhlcvSeries, cfg: &IndiaPortfolioConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &IndiaPortfolioConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("PORT", 100, 1, 100.0);
        let cfg = IndiaPortfolioConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_india_portfolio.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
