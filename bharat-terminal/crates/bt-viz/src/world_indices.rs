// crates/bt-viz/src/world_indices.rs
// Author: Sourish Dey

//! World equity indices dashboard. Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

/// World index data point.
#[derive(Debug, Clone)]
pub struct WorldIndex {
    pub name: String,
    pub region: String,
    pub value: f64,
    pub change_pct: f64,
    pub ytd_pct: f64,
}

/// Sample world indices data.
fn sample_indices() -> Vec<WorldIndex> {
    vec![
        WorldIndex { name: "S&P 500".to_string(), region: "Americas".to_string(), value: 5650.0, change_pct: 0.45, ytd_pct: 18.2 },
        WorldIndex { name: "NASDAQ".to_string(), region: "Americas".to_string(), value: 18200.0, change_pct: 0.72, ytd_pct: 20.5 },
        WorldIndex { name: "Dow Jones".to_string(), region: "Americas".to_string(), value: 41200.0, change_pct: -0.12, ytd_pct: 12.8 },
        WorldIndex { name: "FTSE 100".to_string(), region: "Europe".to_string(), value: 8250.0, change_pct: 0.18, ytd_pct: 6.4 },
        WorldIndex { name: "DAX".to_string(), region: "Europe".to_string(), value: 19400.0, change_pct: -0.35, ytd_pct: 14.1 },
        WorldIndex { name: "CAC 40".to_string(), region: "Europe".to_string(), value: 7950.0, change_pct: 0.08, ytd_pct: 3.2 },
        WorldIndex { name: "Nikkei 225".to_string(), region: "Asia".to_string(), value: 39100.0, change_pct: 1.15, ytd_pct: 16.7 },
        WorldIndex { name: "Hang Seng".to_string(), region: "Asia".to_string(), value: 20100.0, change_pct: -0.85, ytd_pct: 8.9 },
        WorldIndex { name: "Shanghai".to_string(), region: "Asia".to_string(), value: 3250.0, change_pct: 0.22, ytd_pct: 4.5 },
        WorldIndex { name: "Nifty 50".to_string(), region: "Asia".to_string(), value: 24800.0, change_pct: 0.55, ytd_pct: 13.6 },
        WorldIndex { name: "Sensex".to_string(), region: "Asia".to_string(), value: 81500.0, change_pct: 0.48, ytd_pct: 11.2 },
        WorldIndex { name: "ASX 200".to_string(), region: "Oceania".to_string(), value: 8100.0, change_pct: 0.30, ytd_pct: 7.8 },
    ]
}

#[derive(Debug, Clone)]
pub struct WorldIndicesConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for WorldIndicesConfig {
    fn default() -> Self {
        Self {
            title: "World Equity Indices".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl WorldIndicesConfig {
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
    data: &[WorldIndex],
    cfg: &WorldIndicesConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if data.is_empty() {
        return Err(BtError::EmptySeries("indices data".into()));
    }
    fill_background(&root, cfg.theme)?;

    let (w, h) = root.dim_in_pixel();
    let top_pad = 50;
    let bottom_pad = 50;
    let chart_h = h as f64 - top_pad as f64 - bottom_pad as f64;

    root.draw(&Text::new(
        cfg.title.as_str(),
        (w as i32 / 2, 20),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let n = data.len();
    let bar_h = chart_h / n as f64 * 0.7;
    let gap = chart_h / n as f64 * 0.3;
    let label_w = 140.0;
    let val_w = 100.0;
    let bar_area_w = w as f64 - label_w - val_w - 40.0;

    let max_abs_change = data
        .iter()
        .map(|d| d.change_pct.abs())
        .fold(0.0_f64, f64::max)
        .max(0.5);

    for (i, idx) in data.iter().enumerate() {
        let y = top_pad as f64 + i as f64 * (bar_h + gap);

        // Name
        root.draw(&Text::new(
            idx.name.as_str(),
            (10, (y + bar_h / 2.0) as i32),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        // Region
        root.draw(&Text::new(
            idx.region.as_str(),
            (10, (y + bar_h / 2.0 + 14.0) as i32),
            (LABEL_FONT, 9)
                .into_font()
                .color(&cfg.theme.text().mix(0.5)),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        // Change bar
        let bar_w = (idx.change_pct.abs() / max_abs_change) * bar_area_w / 2.0;
        let bar_x = label_w + bar_area_w / 2.0;
        let color = if idx.change_pct >= 0.0 {
            cfg.theme.profit().mix(0.7)
        } else {
            cfg.theme.loss().mix(0.7)
        };

        if idx.change_pct >= 0.0 {
            root.draw(&Rectangle::new(
                [
                    (bar_x as i32, y as i32),
                    ((bar_x + bar_w) as i32, (y + bar_h) as i32),
                ],
                color.filled(),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
        } else {
            root.draw(&Rectangle::new(
                [
                    ((bar_x - bar_w) as i32, y as i32),
                    (bar_x as i32, (y + bar_h) as i32),
                ],
                color.filled(),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
        }

        // Zero line
        root.draw(&PathElement::new(
            vec![
                (bar_x as i32, top_pad as i32),
                (bar_x as i32, (top_pad as f64 + chart_h) as i32),
            ],
            cfg.theme.border().stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        // Value
        root.draw(&Text::new(
            format!("{:.0}", idx.value),
            (label_w as i32 + bar_area_w as i32 + 10, (y + bar_h / 2.0) as i32),
            (LABEL_FONT, 11).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        // Change %
        let change_str = format!("{:+.2}%", idx.change_pct);
        root.draw(&Text::new(
            change_str,
            (label_w as i32 + bar_area_w as i32 + 10, (y + bar_h / 2.0 + 14.0) as i32),
            (LABEL_FONT, 10).into_font().color(&color),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(data: &[WorldIndex], cfg: &WorldIndicesConfig, path: &str) -> Result<()> {
    render(png_root(path)?, data, cfg)
}

pub fn render_svg(data: &[WorldIndex], cfg: &WorldIndicesConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, data, cfg)
}

pub fn render_sample_png(cfg: &WorldIndicesConfig, path: &str) -> Result<()> {
    let data = sample_indices();
    render_png(&data, cfg, path)
}

pub fn render_sample_svg(cfg: &WorldIndicesConfig, path: &str) -> Result<()> {
    let data = sample_indices();
    render_svg(&data, cfg, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let data = sample_indices();
        let cfg = WorldIndicesConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_world_indices.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&data, &cfg, &path).unwrap();
    }
}
