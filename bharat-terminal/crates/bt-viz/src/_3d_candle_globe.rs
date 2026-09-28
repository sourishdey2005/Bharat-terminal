// crates/bt-viz/src/3d_candle_globe.rs
// Author: Sourish Dey

//! 3D candle globe — candles wrapped around a rotating globe (24h cycle).
//! Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CandleGlobe3DConfig {
    pub title: String,
    pub theme: Theme,
    pub hour: f64,
}

impl Default for CandleGlobe3DConfig {
    fn default() -> Self {
        Self { title: "3D Candle Globe".to_string(), theme: Theme::Dark, hour: 14.0 }
    }
}

impl CandleGlobe3DConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn hour(mut self, h: f64) -> Self { self.hour = h.rem_euclid(24.0); self }
}

fn project(x: f64, y: f64, z: f64, cx: f64, cy: f64) -> (i32, i32, f64) {
    let focal = 420.0;
    let s = focal / (focal + z);
    ((cx + x * s) as i32, (cy - y * s) as i32, s)
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
    cfg: &CandleGlobe3DConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let (w, h) = root.dim_in_pixel();
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0 + 30.0;

    root.draw(&Text::new(
        format!("{} — {} ({:02}:00)", cfg.title, series.symbol, cfg.hour as i32),
        (w as i32 / 2 - 150, 14),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let n = series.candles.len().min(24);
    let step = series.candles.len() / n.max(1);
    let min_p = series.candles.iter().map(|c| c.low).fold(f64::MAX, f64::min);
    let max_p = series.candles.iter().map(|c| c.high).fold(f64::MIN, f64::max);
    let pr = (max_p - min_p).max(1e-9);

    let radius = 130.0;
    let rot = cfg.hour / 24.0 * std::f64::consts::TAU;
    let cos_r = rot.cos();
    let sin_r = rot.sin();

    // Globe wireframe
    for lat in [-60.0_f64, -30.0, 0.0, 30.0, 60.0] {
        let lat_r = lat.to_radians();
        let r = radius * lat_r.cos();
        let y = radius * lat_r.sin();
        let pts: Vec<(i32, i32)> = (0..=60)
            .map(|k| {
                let theta = k as f64 / 60.0 * std::f64::consts::TAU;
                let x = r * theta.cos();
                let z = r * theta.sin();
                let rx = x * cos_r - z * sin_r;
                let rz = x * sin_r + z * cos_r;
                let (sx, sy, _) = project(rx, y, rz, cx, cy);
                (sx, sy)
            })
            .collect();
        root.draw(&PathElement::new(pts, cfg.theme.border().mix(0.4).stroke_width(1)))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }
    for lon in 0..12 {
        let theta = lon as f64 / 12.0 * std::f64::consts::TAU;
        let pts: Vec<(i32, i32)> = (0..=40)
            .map(|k| {
                let lat = -90.0 + k as f64 / 40.0 * 180.0;
                let lat_r = lat.to_radians();
                let x = radius * lat_r.cos() * theta.cos();
                let y = radius * lat_r.sin();
                let z = radius * lat_r.cos() * theta.sin();
                let rx = x * cos_r - z * sin_r;
                let rz = x * sin_r + z * cos_r;
                let (sx, sy, _) = project(rx, y, rz, cx, cy);
                (sx, sy)
            })
            .collect();
        root.draw(&PathElement::new(pts, cfg.theme.border().mix(0.4).stroke_width(1)))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    // Candles on globe surface
    let mut candles_3d: Vec<((i32, i32, f64), bool, f64)> = Vec::with_capacity(n);
    for i in 0..n {
        let c = &series.candles[i * step];
        let theta = i as f64 / n as f64 * std::f64::consts::TAU;
        let lat = ((c.close - min_p) / pr - 0.5) * 1.2;
        let lat_r = lat;
        let r = radius + 8.0;
        let x = r * lat_r.cos() * theta.cos();
        let y = r * lat_r.sin();
        let z = r * lat_r.cos() * theta.sin();
        let rx = x * cos_r - z * sin_r;
        let rz = x * sin_r + z * cos_r;
        let (sx, sy, s) = project(rx, y, rz, cx, cy);
        candles_3d.push(((sx, sy, s), c.is_bullish(), (c.high - c.low) / pr));
    }

    candles_3d.sort_by(|a, b| a.0 .2.partial_cmp(&b.0 .2).unwrap_or(std::cmp::Ordering::Equal));

    for ((sx, sy, s), is_bullish, body_h) in &candles_3d {
        let size = (3.0 * s) as i32 + 1;
        let color = if *is_bullish { cfg.theme.profit() } else { cfg.theme.loss() };
        let h = (body_h * 30.0 * s) as i32 + 2;
        root.draw(&Rectangle::new(
            [(*sx - size, *sy - h / 2), (*sx + size, *sy + h / 2)],
            shade(color, 0.4 + 0.6 * s).filled(),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    root.draw(&Text::new(
        "24h cycle — candles on rotating globe",
        (w as i32 / 2 - 110, h as i32 - 40),
        (LABEL_FONT, 11).into_font().color(&cfg.theme.text().mix(0.6)),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &CandleGlobe3DConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &CandleGlobe3DConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = CandleGlobe3DConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_3d_candle_globe.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
