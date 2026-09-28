// crates/bt-viz/src/sector_wheel.rs
// Author: Sourish Dey

//! Sector rotation wheel chart. Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

/// Sector data for the rotation wheel.
#[derive(Debug, Clone)]
pub struct SectorData {
    pub name: String,
    pub momentum: f64,  // -1 to 1
    pub relative_strength: f64, // 0 to 2, 1 = market average
}

/// Sample sector data.
fn sample_sectors() -> Vec<SectorData> {
    vec![
        SectorData { name: "IT".to_string(), momentum: 0.75, relative_strength: 1.35 },
        SectorData { name: "Banking".to_string(), momentum: 0.60, relative_strength: 1.22 },
        SectorData { name: "Pharma".to_string(), momentum: 0.45, relative_strength: 1.10 },
        SectorData { name: "FMCG".to_string(), momentum: 0.30, relative_strength: 1.05 },
        SectorData { name: "Auto".to_string(), momentum: 0.55, relative_strength: 1.18 },
        SectorData { name: "Metals".to_string(), momentum: 0.80, relative_strength: 1.42 },
        SectorData { name: "Energy".to_string(), momentum: 0.20, relative_strength: 0.95 },
        SectorData { name: "Realty".to_string(), momentum: -0.30, relative_strength: 0.78 },
        SectorData { name: "Infra".to_string(), momentum: 0.40, relative_strength: 1.12 },
        SectorData { name: "Media".to_string(), momentum: -0.15, relative_strength: 0.88 },
        SectorData { name: "Chemicals".to_string(), momentum: 0.10, relative_strength: 0.98 },
        SectorData { name: "Consumer".to_string(), momentum: 0.25, relative_strength: 1.02 },
    ]
}

#[derive(Debug, Clone)]
pub struct SectorWheelConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for SectorWheelConfig {
    fn default() -> Self {
        Self {
            title: "Sector Rotation Wheel".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl SectorWheelConfig {
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

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    data: &[SectorData],
    cfg: &SectorWheelConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if data.is_empty() {
        return Err(BtError::EmptySeries("sector data".into()));
    }
    fill_background(&root, cfg.theme)?;

    let (w, h) = root.dim_in_pixel();
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0 + 10.0;
    let r_max = (w.min(h) as f64) * 0.38;
    let r_min = r_max * 0.15;

    root.draw(&Text::new(
        cfg.title.as_str(),
        (w as i32 / 2, 20),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    // Quadrant backgrounds
    let quadrants = [
        (0.0, std::f64::consts::FRAC_PI_2, "Leading", cfg.theme.profit().mix(0.08)),
        (std::f64::consts::FRAC_PI_2, std::f64::consts::PI, "Weakening", cfg.theme.loss().mix(0.08)),
        (std::f64::consts::PI, std::f64::consts::PI * 1.5, "Lagging", cfg.theme.loss().mix(0.05)),
        (std::f64::consts::PI * 1.5, std::f64::consts::TAU, "Improving", cfg.theme.info().mix(0.08)),
    ];

    for (start, end, label, color) in &quadrants {
        let segments = 20;
        let mut pts = Vec::new();
        for s in 0..=segments {
            let t = start + (end - start) * (s as f64 / segments as f64);
            pts.push(((cx + r_max * t.cos()) as i32, (cy + r_max * t.sin()) as i32));
        }
        for s in (0..=segments).rev() {
            let t = start + (end - start) * (s as f64 / segments as f64);
            pts.push(((cx + r_min * t.cos()) as i32, (cy + r_min * t.sin()) as i32));
        }
        root.draw(&Polygon::new(pts, color.filled()))
            .map_err(|e| BtError::Render(e.to_string()))?;

        let mid = (start + end) / 2.0;
        root.draw(&Text::new(
            *label,
            ((cx + (r_max + 20.0) * mid.cos()) as i32 - 20, (cy + (r_max + 20.0) * mid.sin()) as i32),
            (LABEL_FONT, 11)
                .into_font()
                .color(&cfg.theme.text().mix(0.6)),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    // Axis lines
    root.draw(&PathElement::new(
        vec![(cx as i32, (cy - r_max) as i32), (cx as i32, (cy + r_max) as i32)],
        cfg.theme.border().stroke_width(1),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;
    root.draw(&PathElement::new(
        vec![((cx - r_max) as i32, cy as i32), ((cx + r_max) as i32, cy as i32)],
        cfg.theme.border().stroke_width(1),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    // Axis labels
    root.draw(&Text::new(
        "Momentum +",
        (cx as i32 + 5, (cy - r_max) as i32 - 15),
        (LABEL_FONT, 10)
            .into_font()
            .color(&cfg.theme.profit().mix(0.7)),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;
    root.draw(&Text::new(
        "Momentum -",
        (cx as i32 + 5, (cy + r_max) as i32 + 5),
        (LABEL_FONT, 10)
            .into_font()
            .color(&cfg.theme.loss().mix(0.7)),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;
    root.draw(&Text::new(
        "RS +",
        ((cx + r_max) as i32 + 5, cy as i32 - 5),
        (LABEL_FONT, 10)
            .into_font()
            .color(&cfg.theme.text().mix(0.7)),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;
    root.draw(&Text::new(
        "RS -",
        ((cx - r_max) as i32 - 40, cy as i32 - 5),
        (LABEL_FONT, 10)
            .into_font()
            .color(&cfg.theme.text().mix(0.7)),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    // Sector bubbles
    let n = data.len();
    for (i, sector) in data.iter().enumerate() {
        let angle = (i as f64 / n as f64) * std::f64::consts::TAU - std::f64::consts::FRAC_PI_2;
        let r = r_min + (sector.relative_strength - 0.5) / 1.5 * (r_max - r_min);
        let x = cx + r * angle.cos() * sector.momentum.abs().max(0.1);
        let y = cy - r * angle.sin() * sector.momentum.abs().max(0.1);

        let color = if sector.momentum > 0.3 {
            cfg.theme.profit().mix(0.8)
        } else if sector.momentum < -0.1 {
            cfg.theme.loss().mix(0.8)
        } else {
            cfg.theme.info().mix(0.8)
        };

        let bubble_r = 8.0 + sector.relative_strength * 4.0;
        root.draw(&Circle::new(
            (x as i32, y as i32),
            bubble_r as i32,
            color.filled(),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        root.draw(&Text::new(
            sector.name.as_str(),
            (x as i32 - 15, y as i32 - 5),
            (LABEL_FONT, 10).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(data: &[SectorData], cfg: &SectorWheelConfig, path: &str) -> Result<()> {
    render(png_root(path)?, data, cfg)
}

pub fn render_svg(data: &[SectorData], cfg: &SectorWheelConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, data, cfg)
}

pub fn render_sample_png(cfg: &SectorWheelConfig, path: &str) -> Result<()> {
    let data = sample_sectors();
    render_png(&data, cfg, path)
}

pub fn render_sample_svg(cfg: &SectorWheelConfig, path: &str) -> Result<()> {
    let data = sample_sectors();
    render_svg(&data, cfg, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let data = sample_sectors();
        let cfg = SectorWheelConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_sector_wheel.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&data, &cfg, &path).unwrap();
    }
}
