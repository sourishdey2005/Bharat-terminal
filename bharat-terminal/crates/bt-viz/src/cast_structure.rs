// crates/bt-viz/src/cast_structure.rs
// Author: Sourish Dey

//! CAST: Capital structure tree.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CapitalLayer {
    pub name: String,
    pub value: f64,
    pub rate: f64,
    pub priority: usize,
}

impl CapitalLayer {
    pub fn new(name: impl Into<String>, value: f64, rate: f64, priority: usize) -> Self {
        Self { name: name.into(), value: value.max(0.0), rate, priority }
    }
}

#[derive(Debug, Clone)]
pub struct CastStructureConfig {
    pub title: String,
    pub theme: Theme,
    pub show_rates: bool,
}

impl Default for CastStructureConfig {
    fn default() -> Self {
        Self {
            title: "Capital Structure".to_string(),
            theme: Theme::Dark,
            show_rates: true,
        }
    }
}

impl CastStructureConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn show_rates(mut self, s: bool) -> Self { self.show_rates = s; self }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    layers: &[CapitalLayer],
    cfg: &CastStructureConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    if layers.is_empty() {
        return Err(BtError::EmptySeries("capital layers".into()));
    }
    fill_background(&root, cfg.theme)?;

    let total: f64 = layers.iter().map(|l| l.value).sum();
    let mut sorted: Vec<CapitalLayer> = layers.to_vec();
    sorted.sort_by_key(|l| l.priority);

    let (title_area, body_area) = root.split_vertically(50);
    title_area
        .draw(&Text::new(
            format!("{} — Total: {:.0}", cfg.title, total),
            (15, 10),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    let (w, h) = body_area.dim_in_pixel();
    let bar_h = (h as usize - 60) / sorted.len();
    let max_val = sorted.iter().map(|l| l.value).fold(0.0_f64, f64::max);

    for (i, layer) in sorted.iter().enumerate() {
        let y = 40 + i * bar_h;
        let frac = if max_val > 0.0 { layer.value / max_val } else { 0.0 };
        let bar_w = (w as f64 * frac * 0.7) as i32;

        let color = match layer.priority {
            0 => cfg.theme.loss(),
            1 => cfg.theme.accent(),
            2 => cfg.theme.info(),
            _ => cfg.theme.profit(),
        };

        body_area
            .draw(&Rectangle::new(
                [(10, y as i32), (10 + bar_w, (y + bar_h - 5) as i32)],
                color.filled(),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;

        body_area
            .draw(&Text::new(
                layer.name.clone(),
                (15, y as i32 + 5),
                (LABEL_FONT, 13).into_font().color(&cfg.theme.text()),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;

        body_area
            .draw(&Text::new(
                format!("{:.0}", layer.value),
                (15 + bar_w, y as i32 + 5),
                (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;

        if cfg.show_rates {
            body_area
                .draw(&Text::new(
                    format!("{:.2}%", layer.rate),
                    (w as i32 - 80, y as i32 + 5),
                    (LABEL_FONT, 12).into_font().color(&color),
                ))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(layers: &[CapitalLayer], cfg: &CastStructureConfig, path: &str) -> Result<()> {
    render(png_root(path)?, layers, cfg)
}

pub fn render_svg(layers: &[CapitalLayer], cfg: &CastStructureConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, layers, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let layers = vec![
            CapitalLayer::new("Senior Debt", 500.0, 6.5, 0),
            CapitalLayer::new("Sub Debt", 200.0, 8.5, 1),
            CapitalLayer::new("Preferred", 100.0, 9.0, 2),
            CapitalLayer::new("Equity", 300.0, 12.0, 3),
        ];
        let cfg = CastStructureConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_cast_structure.png").to_str().unwrap().to_string();
        render_png(&layers, &cfg, &path).unwrap();
    }
}
