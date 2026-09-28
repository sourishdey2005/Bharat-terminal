// crates/bt-viz/src/regulatory_updates.rs
// Author: Sourish Dey

//! SEBI/RBI regulatory updates tracker. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct RegulatoryUpdatesConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for RegulatoryUpdatesConfig {
    fn default() -> Self {
        Self { title: "SEBI/RBI Updates".to_string(), theme: Theme::Dark }
    }
}

impl RegulatoryUpdatesConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
}

fn sample_regulatory() -> Vec<(String, String, String, String)> {
    vec![
        ("SEBI".to_string(), "F&O position limits revised".to_string(), "28-Sep".to_string(), "Active".to_string()),
        ("RBI".to_string(), "Liquidity adjustment facility".to_string(), "27-Sep".to_string(), "Active".to_string()),
        ("SEBI".to_string(), "Mutual fund stress test norms".to_string(), "25-Sep".to_string(), "Draft".to_string()),
        ("RBI".to_string(), "Digital lending guidelines".to_string(), "22-Sep".to_string(), "Active".to_string()),
        ("SEBI".to_string(), "Insider trading amendments".to_string(), "20-Sep".to_string(), "Active".to_string()),
        ("RBI".to_string(), "Co-lending guidelines".to_string(), "18-Sep".to_string(), "Draft".to_string()),
        ("SEBI".to_string(), "ESG disclosure norms".to_string(), "15-Sep".to_string(), "Active".to_string()),
        ("RBI".to_string(), "Pre-sanctioned credit lines".to_string(), "12-Sep".to_string(), "Active".to_string()),
        ("SEBI".to_string(), "Algorithmic trading audit".to_string(), "10-Sep".to_string(), "Draft".to_string()),
        ("RBI".to_string(), "CBDC pilot expansion".to_string(), "08-Sep".to_string(), "Active".to_string()),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &RegulatoryUpdatesConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_regulatory();
    let (w, h) = root.dim_in_pixel();

    root.draw(&Text::new(
        format!("{} — {}", cfg.title, series.symbol),
        (10, 10),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let row_h = (h as i32 - 100) / (data.len() as i32 + 1);
    let col_px = [10, 150, 700, 900, 1050];

    for (x, label) in col_px.iter().zip(["Regulator", "Update", "Date", "Status", "Impact"].iter()) {
        root.draw(&Text::new(label.to_string(), ( *x, 45), (LABEL_FONT, 12).into_font().color(&cfg.theme.accent())))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    for (i, (reg, update, date, status)) in data.iter().enumerate() {
        let y = 45 + (i as i32 + 1) * row_h;
        let reg_color = if reg == "SEBI" { cfg.theme.info() } else { cfg.theme.accent() };
        root.draw(&Text::new(reg.clone(), ( col_px[0], y + row_h / 2 - 6), (LABEL_FONT, 12).into_font().color(&reg_color)))
            .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(update.clone(), ( col_px[1], y + row_h / 2 - 6), (LABEL_FONT, 12).into_font().color(&cfg.theme.text())))
            .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(date.clone(), ( col_px[2], y + row_h / 2 - 6), (LABEL_FONT, 12).into_font().color(&cfg.theme.text().mix(0.7))))
            .map_err(|e| BtError::Render(e.to_string()))?;
        let status_color = if status == "Active" { cfg.theme.profit() } else { cfg.theme.accent() };
        root.draw(&Text::new(status.clone(), ( col_px[3], y + row_h / 2 - 6), (LABEL_FONT, 12).into_font().color(&status_color)))
            .map_err(|e| BtError::Render(e.to_string()))?;
        let impact = if status == "Active" { "High" } else { "Medium" };
        root.draw(&Text::new(impact.to_string(), ( col_px[4], y + row_h / 2 - 6), (LABEL_FONT, 12).into_font().color(&cfg.theme.text().mix(0.7))))
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

pub fn render_png(series: &OhlcvSeries, cfg: &RegulatoryUpdatesConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &RegulatoryUpdatesConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("REG", 100, 1, 100.0);
        let cfg = RegulatoryUpdatesConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_regulatory_updates.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
