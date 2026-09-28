// crates/bt-viz/src/wei_monitor.rs
// Author: Sourish Dey

//! WEI: World equity monitor with sparklines.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct WeiIndex {
    pub name: String,
    pub values: Vec<f64>,
    pub change_pct: f64,
}

impl WeiIndex {
    pub fn new(name: impl Into<String>, values: Vec<f64>, change_pct: f64) -> Self {
        Self {
            name: name.into(),
            values,
            change_pct,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WeiMonitorConfig {
    pub title: String,
    pub theme: Theme,
    pub sparkline_width: usize,
    pub sparkline_height: usize,
}

impl Default for WeiMonitorConfig {
    fn default() -> Self {
        Self {
            title: "World Equity Monitor".to_string(),
            theme: Theme::Dark,
            sparkline_width: 12,
            sparkline_height: 40,
        }
    }
}

impl WeiMonitorConfig {
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
    pub fn sparkline_width(mut self, w: usize) -> Self {
        self.sparkline_width = w.max(5).min(50);
        self
    }
    pub fn sparkline_height(mut self, h: usize) -> Self {
        self.sparkline_height = h.max(20).min(100);
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    indices: &[WeiIndex],
    cfg: &WeiMonitorConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if indices.is_empty() {
        return Err(BtError::EmptySeries("world indices".into()));
    }
    fill_background(&root, cfg.theme)?;

    let (title_area, body_area) = root.split_vertically(50);
    title_area
        .draw(&Text::new(
            cfg.title.clone(),
            (15, 10),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    let (w, h) = body_area.dim_in_pixel();
    let cols = 4;
    let rows = (indices.len() + cols - 1) / cols;
    let cell_w = w as usize / cols;
    let cell_h = (h as usize - 30) / rows.max(1);

    for (idx, index) in indices.iter().enumerate() {
        let col = idx % cols;
        let row = idx / cols;
        let x0 = col * cell_w;
        let y0 = 30 + row * cell_h;

        body_area
            .draw(&Text::new(
                index.name.clone(),
                (x0 as i32 + 5, y0 as i32 + 2),
                (LABEL_FONT, 13).into_font().color(&cfg.theme.text()),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;

        let change_color = if index.change_pct >= 0.0 {
            cfg.theme.profit()
        } else {
            cfg.theme.loss()
        };
        body_area
            .draw(&Text::new(
                format!("{:+.2}%", index.change_pct),
                (x0 as i32 + 5, y0 as i32 + 18),
                (LABEL_FONT, 12).into_font().color(&change_color),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;

        if index.values.len() >= 2 {
            let v_min = index.values.iter().cloned().fold(f64::INFINITY, f64::min);
            let v_max = index
                .values
                .iter()
                .cloned()
                .fold(f64::NEG_INFINITY, f64::max);
            let v_range = (v_max - v_min).max(1e-6);
            let spark_w = cell_w - 20;
            let spark_h = cfg.sparkline_height;

            let points: Vec<(f64, f64)> = index
                .values
                .iter()
                .enumerate()
                .map(|(i, &v)| {
                    let x = i as f64 / (index.values.len() - 1).max(1) as f64 * spark_w as f64;
                    let y = (1.0 - (v - v_min) / v_range) * spark_h as f64;
                    (x, y)
                })
                .collect();

            let path_points: Vec<(i32, i32)> = points
                .iter()
                .map(|&(x, y)| ((x0 + 10) as i32 + x as i32, (y0 + 35) as i32 + y as i32))
                .collect();

            body_area
                .draw(&PathElement::new(path_points, change_color.stroke_width(1)))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(indices: &[WeiIndex], cfg: &WeiMonitorConfig, path: &str) -> Result<()> {
    render(png_root(path)?, indices, cfg)
}

pub fn render_svg(indices: &[WeiIndex], cfg: &WeiMonitorConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, indices, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let indices: Vec<WeiIndex> = (0..8)
            .map(|i| {
                let values: Vec<f64> = (0..20)
                    .map(|j| 100.0 + (j as f64 * (i as f64 + 1.0) * 0.5))
                    .collect();
                WeiIndex::new(format!("IDX{}", i), values, (i as f64 - 4.0) * 0.5)
            })
            .collect();
        let cfg = WeiMonitorConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_wei_monitor.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&indices, &cfg, &path).unwrap();
    }
}
