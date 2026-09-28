// crates/bt-viz/src/ipo_pipeline.rs
// Author: Sourish Dey

//! IPO pipeline tracker (upcoming, listed, subscribed). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct IpoPipelineConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for IpoPipelineConfig {
    fn default() -> Self {
        Self { title: "IPO Pipeline".to_string(), theme: Theme::Dark }
    }
}

impl IpoPipelineConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
}

fn sample_ipos() -> Vec<(String, String, f64, f64)> {
    vec![
        ("Nova AgriTech".to_string(), "Listed".to_string(), 15.0, 8.5),
        ("Surya Roshni".to_string(), "Listed".to_string(), 12.0, 3.2),
        ("Kaka Industries".to_string(), "Upcoming".to_string(), 8.0, 0.0),
        ("V.L.Infraprojects".to_string(), "Upcoming".to_string(), 5.0, 0.0),
        ("Mangalam Worldwide".to_string(), "Live".to_string(), 10.0, 12.5),
        ("Transtech Optelec".to_string(), "Live".to_string(), 6.0, 4.8),
        ("Apeejay Surrendra".to_string(), "Filed".to_string(), 20.0, 0.0),
        ("OYO Rooms".to_string(), "Filed".to_string(), 50.0, 0.0),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &IpoPipelineConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_ipos();
    let (w, h) = root.dim_in_pixel();

    root.draw(&Text::new(
        format!("{} — {}", cfg.title, series.symbol),
        (10, 10),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let row_h = (h as i32 - 100) / (data.len() as i32 + 1);
    let col_px = [10, 250, 450, 650, 850];

    for (x, label) in col_px.iter().zip(["Company", "Status", "Issue Size (Cr)", "Subscribed (x)", "Band"].iter()) {
        root.draw(&Text::new(label.to_string(), ( *x, 45), (LABEL_FONT, 12).into_font().color(&cfg.theme.accent())))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    for (i, (name, status, size, sub)) in data.iter().enumerate() {
        let y = 45 + (i as i32 + 1) * row_h;
        root.draw(&Text::new(name.clone(), ( col_px[0], y + row_h / 2 - 6), (LABEL_FONT, 12).into_font().color(&cfg.theme.text())))
            .map_err(|e| BtError::Render(e.to_string()))?;
        let color = match status.as_str() {
            "Listed" => cfg.theme.profit(),
            "Live" => cfg.theme.info(),
            "Upcoming" => cfg.theme.accent(),
            _ => cfg.theme.text(),
        };
        root.draw(&Text::new(status.clone(), ( col_px[1], y + row_h / 2 - 6), (LABEL_FONT, 12).into_font().color(&color)))
            .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(format!("{:.0}", size), ( col_px[2], y + row_h / 2 - 6), (LABEL_FONT, 12).into_font().color(&cfg.theme.text())))
            .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(format!("{:.1}x", sub), ( col_px[3], y + row_h / 2 - 6), (LABEL_FONT, 12).into_font().color(&cfg.theme.text())))
            .map_err(|e| BtError::Render(e.to_string()))?;
        let bar_w = (sub * 15.0).min(200.0) as i32;
        root.draw(&Rectangle::new(
            [(col_px[4], y + row_h / 2 - 8), (col_px[4] + bar_w, y + row_h / 2 + 4)],
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

pub fn render_png(series: &OhlcvSeries, cfg: &IpoPipelineConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &IpoPipelineConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("IPO", 100, 1, 100.0);
        let cfg = IpoPipelineConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_ipo_pipeline.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
