// crates/bt-viz/src/3d_price_volume.rs
// Author: Sourish Dey

//! 3D price-volume heightmap — price × volume over time as a 3D terrain.
//! Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct PriceVolume3DConfig {
    pub title: String,
    pub theme: Theme,
    pub bars: usize,
}

impl Default for PriceVolume3DConfig {
    fn default() -> Self {
        Self { title: "3D Price-Volume".to_string(), theme: Theme::Dark, bars: 30 }
    }
}

impl PriceVolume3DConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn bars(mut self, n: usize) -> Self { self.bars = n.clamp(5, 60); self }
}

fn project(x: f64, y: f64, z: f64, cx: f64, cy: f64) -> (i32, i32) {
    let focal = 500.0;
    let s = focal / (focal + z);
    ((cx + x * s) as i32, (cy - y * s) as i32)
}

fn shade(c: RGBColor, f: f64) -> RGBColor {
    let f = f.clamp(0.15, 1.2);
    RGBColor(
        (c.0 as f64 * f).min(255.0) as u8,
        (c.1 as f64 * f).min(255.0) as u8,
        (c.2 as f64 * f).min(255.0) as u8,
    )
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &PriceVolume3DConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let (w, h) = root.dim_in_pixel();
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0 + 40.0;

    root.draw(&Text::new(
        format!("{} — {}", cfg.title, series.symbol),
        (w as i32 / 2 - 120, 14),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let n = series.candles.len().min(cfg.bars);
    let step = series.candles.len() / n.max(1);
    let max_vol = series.candles.iter().map(|c| c.volume).fold(0.0_f64, f64::max);
    let min_p = series.candles.iter().map(|c| c.low).fold(f64::MAX, f64::min);
    let max_p = series.candles.iter().map(|c| c.high).fold(f64::MIN, f64::max);
    let pr = (max_p - min_p).max(1e-9);

    let x_span = 340.0;
    let y_span = 100.0;
    let z_span = 180.0;

    for i in (0..n).rev() {
        let c = &series.candles[i * step];
        let x = (i as f64 / n as f64 - 0.5) * x_span;
        let y_base = ((c.close - min_p) / pr - 0.5) * y_span;
        let z = (c.volume / max_vol.max(1.0)) * z_span;

        let base = if c.is_bullish() { cfg.theme.profit() } else { cfg.theme.loss() };
        let bw = 4.0;
        let bh = 6.0 + (c.high - c.low) / pr * 40.0;

        let b = [
            project(x - bw, y_base, z, cx, cy),
            project(x + bw, y_base, z, cx, cy),
            project(x + bw, y_base, z + 8.0, cx, cy),
            project(x - bw, y_base, z + 8.0, cx, cy),
        ];
        let t = [
            project(x - bw, y_base + bh, z, cx, cy),
            project(x + bw, y_base + bh, z, cx, cy),
            project(x + bw, y_base + bh, z + 8.0, cx, cy),
            project(x - bw, y_base + bh, z + 8.0, cx, cy),
        ];

        let faces = [
            (vec![b[0], b[1], t[1], t[0]], 0.55),
            (vec![b[1], b[2], t[2], t[1]], 0.75),
            (vec![b[2], b[3], t[3], t[2]], 0.65),
            (vec![t[0], t[1], t[2], t[3]], 1.0),
        ];
        for (pts, f) in faces {
            root.draw(&Polygon::new(pts, shade(base, f).filled()))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    root.draw(&Text::new(
        "X: time  Y: price  Z: volume",
        (w as i32 / 2 - 90, h as i32 - 40),
        (LABEL_FONT, 11).into_font().color(&cfg.theme.text().mix(0.6)),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &PriceVolume3DConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &PriceVolume3DConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = PriceVolume3DConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_3d_price_volume.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
