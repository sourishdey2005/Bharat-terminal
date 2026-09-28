// crates/bt-viz/src/india_breadth.rs
// Author: Sourish Dey

//! India market breadth (advances, declines, unchanged). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct IndiaBreadthConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for IndiaBreadthConfig {
    fn default() -> Self {
        Self {
            title: "India Market Breadth".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl IndiaBreadthConfig {
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

fn sample_breadth() -> Vec<(String, f64, f64, f64)> {
    vec![
        ("Nifty 50".to_string(), 32.0, 18.0, 0.0),
        ("Nifty Next 50".to_string(), 28.0, 22.0, 0.0),
        ("Nifty 100".to_string(), 60.0, 40.0, 0.0),
        ("Nifty 200".to_string(), 110.0, 90.0, 0.0),
        ("Nifty 500".to_string(), 280.0, 220.0, 0.0),
        ("Nifty Midcap 150".to_string(), 85.0, 65.0, 0.0),
        ("Nifty Smallcap 250".to_string(), 140.0, 110.0, 0.0),
        ("Nifty Microcap 250".to_string(), 130.0, 120.0, 0.0),
        ("Nifty Total Mkt".to_string(), 900.0, 600.0, 0.0),
        ("BSE 500".to_string(), 270.0, 230.0, 0.0),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &IndiaBreadthConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_breadth();
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
        .zip(["Index", "Advances", "Declines", "Unchanged", "A/D Ratio"].iter())
    {
        root.draw(&Text::new(
            label.to_string(),
            (*x, 45),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.accent()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    for (i, (name, adv, dec, unch)) in data.iter().enumerate() {
        let y = 45 + (i as i32 + 1) * row_h;
        root.draw(&Text::new(
            name.clone(),
            (col_px[0], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            format!("{:.0}", adv),
            (col_px[1], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.profit()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            format!("{:.0}", dec),
            (col_px[2], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.loss()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            format!("{:.0}", unch),
            (col_px[3], y + row_h / 2 - 6),
            (LABEL_FONT, 12)
                .into_font()
                .color(&cfg.theme.text().mix(0.7)),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        let ratio = if *dec > 0.0 { adv / dec } else { 99.0 };
        let ratio_color = if ratio >= 1.0 {
            cfg.theme.profit()
        } else {
            cfg.theme.loss()
        };
        root.draw(&Text::new(
            format!("{:.2}", ratio),
            (col_px[4], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&ratio_color),
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

pub fn render_png(series: &OhlcvSeries, cfg: &IndiaBreadthConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &IndiaBreadthConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("BREADTH", 100, 1, 100.0);
        let cfg = IndiaBreadthConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_india_breadth.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
