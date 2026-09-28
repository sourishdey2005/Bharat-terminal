// crates/bt-viz/src/port_attribution.rs
// Author: Sourish Dey

//! PORT: Brinson attribution waterfall.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct AttributionEntry {
    pub name: String,
    pub value: f64,
}

impl AttributionEntry {
    pub fn new(name: impl Into<String>, value: f64) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PortAttributionConfig {
    pub title: String,
    pub theme: Theme,
    pub show_total: bool,
}

impl Default for PortAttributionConfig {
    fn default() -> Self {
        Self {
            title: "Brinson Attribution Waterfall".to_string(),
            theme: Theme::Dark,
            show_total: true,
        }
    }
}

impl PortAttributionConfig {
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
    pub fn show_total(mut self, s: bool) -> Self {
        self.show_total = s;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    entries: &[AttributionEntry],
    cfg: &PortAttributionConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if entries.is_empty() {
        return Err(BtError::EmptySeries("attribution entries".into()));
    }
    fill_background(&root, cfg.theme)?;

    let total: f64 = entries.iter().map(|e| e.value).sum();
    let mut cumulative = 0.0_f64;
    let mut waterfall: Vec<(f64, f64)> = Vec::new();

    for entry in entries {
        let start = cumulative;
        cumulative += entry.value;
        waterfall.push((start, cumulative));
    }

    let v_min = waterfall
        .iter()
        .map(|(s, e)| (*s).min(*e))
        .fold(f64::INFINITY, f64::min);
    let v_max = waterfall
        .iter()
        .map(|(s, e)| (*s).max(*e))
        .fold(f64::NEG_INFINITY, f64::max);
    let pad = (v_max - v_min).max(0.1) * 0.1;

    let n_bars = entries.len() + if cfg.show_total { 1 } else { 0 };
    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — Total: {:.2}%", cfg.title, total),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0..n_bars, (v_min - pad)..(v_max + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .x_labels(entries.len())
        .x_label_formatter(&|x| entries.get(*x).map(|e| e.name.clone()).unwrap_or_default())
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    for (i, (start, end)) in waterfall.iter().enumerate() {
        let value = end - start;
        let color = if value >= 0.0 {
            cfg.theme.profit()
        } else {
            cfg.theme.loss()
        };
        let y_start = (*start).min(*end);
        let y_end = (*start).max(*end);

        chart
            .draw_series(std::iter::once(Rectangle::new(
                [(i, y_start), (i + 1, y_end)],
                color.filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;

        root.draw(&Text::new(
            format!("{:+.2}", value),
            (i as i32 * 100 + 10, 80),
            (LABEL_FONT, 10).into_font().color(&color),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    if cfg.show_total {
        chart
            .draw_series(std::iter::once(Rectangle::new(
                [(entries.len(), 0.0), (entries.len() + 1, total)],
                cfg.theme.accent().filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(
    entries: &[AttributionEntry],
    cfg: &PortAttributionConfig,
    path: &str,
) -> Result<()> {
    render(png_root(path)?, entries, cfg)
}

pub fn render_svg(
    entries: &[AttributionEntry],
    cfg: &PortAttributionConfig,
    path: &str,
) -> Result<()> {
    render(svg_root(path)?, entries, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let entries = vec![
            AttributionEntry::new("Allocation", 1.2),
            AttributionEntry::new("Selection", 0.8),
            AttributionEntry::new("Interaction", 0.3),
            AttributionEntry::new("Currency", -0.1),
        ];
        let cfg = PortAttributionConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_port_attribution.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&entries, &cfg, &path).unwrap();
    }
}
