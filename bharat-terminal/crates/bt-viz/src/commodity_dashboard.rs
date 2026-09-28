// crates/bt-viz/src/commodity_dashboard.rs
// Author: Sourish Dey

//! MCX/NCDEX commodity dashboard. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CommodityDashboardConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for CommodityDashboardConfig {
    fn default() -> Self {
        Self {
            title: "MCX/NCDEX Commodities".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl CommodityDashboardConfig {
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

fn sample_commodities() -> Vec<(String, f64, f64, String)> {
    vec![
        ("GOLD".to_string(), 72500.0, 0.8, "10g".to_string()),
        ("SILVER".to_string(), 85500.0, 1.2, "1kg".to_string()),
        ("CRUDE OIL".to_string(), 5850.0, -0.6, "bbl".to_string()),
        ("NATURAL GAS".to_string(), 185.0, 2.1, "mmBtu".to_string()),
        ("COPPER".to_string(), 845.0, 0.4, "kg".to_string()),
        ("ZINC".to_string(), 245.0, -0.3, "kg".to_string()),
        ("ALUMINIUM".to_string(), 215.0, 0.2, "kg".to_string()),
        ("COTTON".to_string(), 58500.0, 1.5, "bale".to_string()),
        ("MENTHA OIL".to_string(), 950.0, -0.8, "kg".to_string()),
        ("CARDAMOM".to_string(), 1450.0, 0.9, "kg".to_string()),
        ("TURMERIC".to_string(), 7200.0, -1.1, "qtl".to_string()),
        ("GUAR SEED".to_string(), 5800.0, 0.6, "qtl".to_string()),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &CommodityDashboardConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_commodities();
    let (w, h) = root.dim_in_pixel();

    root.draw(&Text::new(
        format!("{} — {}", cfg.title, series.symbol),
        (10, 10),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let row_h = (h as i32 - 100) / (data.len() as i32 + 1);
    let col_px = [10, 200, 420, 620, 820];

    for (x, label) in col_px
        .iter()
        .zip(["Commodity", "Price (INR)", "Change %", "Unit", "Trend"].iter())
    {
        root.draw(&Text::new(
            label.to_string(),
            (*x, 45),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.accent()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    for (i, (name, price, chg, unit)) in data.iter().enumerate() {
        let y = 45 + (i as i32 + 1) * row_h;
        root.draw(&Text::new(
            name.clone(),
            (col_px[0], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            format!("{:.2}", price),
            (col_px[1], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        let color = if *chg >= 0.0 {
            cfg.theme.profit()
        } else {
            cfg.theme.loss()
        };
        root.draw(&Text::new(
            format!("{:+.2}%", chg),
            (col_px[2], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&color),
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
        let bars = (chg.abs() * 3.0) as i32;
        let bar_color = if *chg >= 0.0 {
            cfg.theme.profit()
        } else {
            cfg.theme.loss()
        };
        for b in 0..bars.max(1) {
            root.draw(&Rectangle::new(
                [
                    (col_px[4] + b * 8, y + row_h / 2 - 8),
                    (col_px[4] + b * 8 + 6, y + row_h / 2 + 4),
                ],
                bar_color.filled(),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
        }
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

pub fn render_png(series: &OhlcvSeries, cfg: &CommodityDashboardConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &CommodityDashboardConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("MCX", 100, 1, 100.0);
        let cfg = CommodityDashboardConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_commodity_dashboard.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
