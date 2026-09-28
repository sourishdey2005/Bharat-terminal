// crates/bt-viz/src/iv_surface.rs
// Author: Sourish Dey

//! Implied volatility surface (strike x tenor). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct IvSurfaceConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for IvSurfaceConfig {
    fn default() -> Self {
        Self {
            title: "IV Surface".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl IvSurfaceConfig {
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

fn blend_colors(a: RGBColor, b: RGBColor, t: f64) -> RGBColor {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: u8, y: u8| (x as f64 + (y as f64 - x as f64) * t).round() as u8;
    RGBColor(lerp(a.0, b.0), lerp(a.1, b.1), lerp(a.2, b.2))
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &IvSurfaceConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let spot = series.candles.last().map(|c| c.close).unwrap_or(100.0);
    let base = (spot / 10.0).round() * 10.0;
    let strikes: Vec<f64> = (-5..=5).map(|i| base + i as f64 * 10.0).collect();
    let tenors = [7.0f64, 15.0, 30.0, 60.0, 90.0];

    let (w, h) = root.dim_in_pixel();
    let grid_x = 80.0;
    let grid_y = 60.0;
    let grid_w = w as f64 - grid_x - 40.0;
    let grid_h = h as f64 - grid_y - 70.0;
    let cw = grid_w / strikes.len() as f64;
    let ch = grid_h / tenors.len() as f64;

    root.draw(&Text::new(
        format!("{} — {} (Spot {:.2})", cfg.title, series.symbol, spot),
        (10, 10),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let iv_min = 0.05;
    let iv_max = 0.60;

    for (ti, &tenor) in tenors.iter().enumerate() {
        for (si, &k) in strikes.iter().enumerate() {
            let m = (k - spot).abs() / spot;
            let smile = 0.18 * m * m * 100.0;
            let term = 0.02 * (tenor / 30.0).ln().max(0.0);
            let iv = (0.22 + smile + term).clamp(iv_min, iv_max);
            let t = (iv - iv_min) / (iv_max - iv_min);
            let color = blend_colors(cfg.theme.info(), cfg.theme.loss(), t);
            let x = grid_x + si as f64 * cw;
            let y = grid_y + ti as f64 * ch;
            root.draw(&Rectangle::new(
                [
                    (x as i32, y as i32),
                    ((x + cw) as i32 - 1, (y + ch) as i32 - 1),
                ],
                color.filled(),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
            if cw > 40.0 && ch > 24.0 {
                root.draw(&Text::new(
                    format!("{:.1}", iv * 100.0),
                    (x as i32 + 6, y as i32 + ch as i32 / 2 - 6),
                    (LABEL_FONT, 11).into_font().color(&cfg.theme.text()),
                ))
                .map_err(|e| BtError::Render(e.to_string()))?;
            }
        }
        root.draw(&Text::new(
            format!("{}D", tenor as i32),
            (10, (grid_y + ti as f64 * ch + ch / 2.0) as i32),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    for (si, &k) in strikes.iter().enumerate() {
        let x = grid_x + si as f64 * cw + cw / 2.0;
        root.draw(&Text::new(
            format!("{:.0}", k),
            (x as i32 - 14, (grid_y + grid_h + 4.0) as i32),
            (LABEL_FONT, 11).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &IvSurfaceConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &IvSurfaceConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("NIFTY", 100, 1, 100.0);
        let cfg = IvSurfaceConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_iv_surface.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
