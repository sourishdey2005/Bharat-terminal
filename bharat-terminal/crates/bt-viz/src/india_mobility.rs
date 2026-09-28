// crates/bt-viz/src/india_mobility.rs
// Author: Sourish Dey

//! Mobile access status (app version, sync, connectivity). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct IndiaMobilityConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for IndiaMobilityConfig {
    fn default() -> Self {
        Self { title: "Mobile Access Status".to_string(), theme: Theme::Dark }
    }
}

impl IndiaMobilityConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
}

fn sample_mobile() -> Vec<(String, String, String, f64)> {
    vec![
        ("Android App".to_string(), "v2.4.1".to_string(), "Connected".to_string(), 98.5),
        ("iOS App".to_string(), "v2.4.0".to_string(), "Connected".to_string(), 97.8),
        ("PWA Web".to_string(), "v2.4.1".to_string(), "Connected".to_string(), 99.2),
        ("API Gateway".to_string(), "v3.1.0".to_string(), "Connected".to_string(), 99.9),
        ("WebSocket".to_string(), "v1.8.2".to_string(), "Connected".to_string(), 99.5),
        ("Push Notif".to_string(), "v1.2.0".to_string(), "Degraded".to_string(), 85.0),
        ("Offline Sync".to_string(), "v1.5.0".to_string(), "Connected".to_string(), 96.0),
        ("Biometric Auth".to_string(), "v2.0.0".to_string(), "Connected".to_string(), 99.0),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &IndiaMobilityConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_mobile();
    let (w, h) = root.dim_in_pixel();

    root.draw(&Text::new(
        format!("{} — {}", cfg.title, series.symbol),
        (10, 10),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let row_h = (h as i32 - 100) / (data.len() as i32 + 1);
    let col_px = [10, 280, 480, 700, 900];

    for (x, label) in col_px.iter().zip(["Service", "Version", "Status", "Uptime %", "Health"].iter()) {
        root.draw(&Text::new(label.to_string(), ( *x, 45), (LABEL_FONT, 12).into_font().color(&cfg.theme.accent())))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    for (i, (name, version, status, uptime)) in data.iter().enumerate() {
        let y = 45 + (i as i32 + 1) * row_h;
        root.draw(&Text::new(name.clone(), ( col_px[0], y + row_h / 2 - 6), (LABEL_FONT, 12).into_font().color(&cfg.theme.text())))
            .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(version.clone(), ( col_px[1], y + row_h / 2 - 6), (LABEL_FONT, 12).into_font().color(&cfg.theme.text().mix(0.8))))
            .map_err(|e| BtError::Render(e.to_string()))?;
        let status_color = if status == "Connected" { cfg.theme.profit() } else { cfg.theme.accent() };
        root.draw(&Text::new(status.clone(), ( col_px[2], y + row_h / 2 - 6), (LABEL_FONT, 12).into_font().color(&status_color)))
            .map_err(|e| BtError::Render(e.to_string()))?;
        let up_color = if *uptime >= 99.0 { cfg.theme.profit() } else if *uptime >= 95.0 { cfg.theme.accent() } else { cfg.theme.loss() };
        root.draw(&Text::new(format!("{:.1}", uptime), ( col_px[3], y + row_h / 2 - 6), (LABEL_FONT, 12).into_font().color(&up_color)))
            .map_err(|e| BtError::Render(e.to_string()))?;
        let bar_w = (uptime * 2.0) as i32;
        root.draw(&Rectangle::new(
            [(col_px[4], y + row_h / 2 - 8), (col_px[4] + bar_w.min(200), y + row_h / 2 + 4)],
            up_color.filled(),
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

pub fn render_png(series: &OhlcvSeries, cfg: &IndiaMobilityConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &IndiaMobilityConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("MOBILE", 100, 1, 100.0);
        let cfg = IndiaMobilityConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_india_mobility.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
