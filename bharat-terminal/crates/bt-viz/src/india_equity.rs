// crates/bt-viz/src/india_equity.rs
// Author: Sourish Dey

//! India equity dashboard (indices, sectoral, market cap). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct IndiaEquityConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for IndiaEquityConfig {
    fn default() -> Self {
        Self {
            title: "India Equity Dashboard".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl IndiaEquityConfig {
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

fn sample_equity() -> Vec<(String, f64, f64, f64)> {
    vec![
        ("Nifty 50".to_string(), 24812.0, 0.8, 195.5),
        ("Sensex".to_string(), 81450.0, 0.6, 280.2),
        ("Nifty Bank".to_string(), 51250.0, 1.2, 45.8),
        ("Nifty IT".to_string(), 42150.0, -0.8, 38.5),
        ("Nifty Auto".to_string(), 24850.0, 1.8, 22.4),
        ("Nifty Pharma".to_string(), 22100.0, -0.5, 18.2),
        ("Nifty Metal".to_string(), 9850.0, 2.5, 12.8),
        ("Nifty Realty".to_string(), 1120.0, 3.2, 8.5),
        ("Nifty FMCG".to_string(), 58200.0, 0.3, 52.5),
        ("Nifty Energy".to_string(), 42500.0, 0.9, 28.5),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &IndiaEquityConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_equity();
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
        .zip(["Index", "Level", "Change %", "M Cap (L Cr)", "Trend"].iter())
    {
        root.draw(&Text::new(
            label.to_string(),
            (*x, 45),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.accent()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    for (i, (name, level, change, mcap)) in data.iter().enumerate() {
        let y = 45 + (i as i32 + 1) * row_h;
        root.draw(&Text::new(
            name.clone(),
            (col_px[0], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            format!("{:.0}", level),
            (col_px[1], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        let color = if *change >= 0.0 {
            cfg.theme.profit()
        } else {
            cfg.theme.loss()
        };
        root.draw(&Text::new(
            format!("{:+.2}%", change),
            (col_px[2], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&color),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            format!("{:.1}", mcap),
            (col_px[3], y + row_h / 2 - 6),
            (LABEL_FONT, 12)
                .into_font()
                .color(&cfg.theme.text().mix(0.8)),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        let bars = (change.abs() * 5.0) as i32;
        for b in 0..bars.max(1) {
            root.draw(&Rectangle::new(
                [
                    (col_px[4] + b * 8, y + row_h / 2 - 8),
                    (col_px[4] + b * 8 + 6, y + row_h / 2 + 4),
                ],
                color.filled(),
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

pub fn render_png(series: &OhlcvSeries, cfg: &IndiaEquityConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &IndiaEquityConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("EQ", 100, 1, 100.0);
        let cfg = IndiaEquityConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_india_equity.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
