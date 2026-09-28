// crates/bt-viz/src/mov_contribution.rs
// Author: Sourish Dey

//! MOV: Index mover contribution bars.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct MoverEntry {
    pub symbol: String,
    pub contribution: f64,
    pub weight: f64,
}

impl MoverEntry {
    pub fn new(symbol: impl Into<String>, contribution: f64, weight: f64) -> Self {
        Self {
            symbol: symbol.into(),
            contribution,
            weight: weight.max(0.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MovContributionConfig {
    pub title: String,
    pub theme: Theme,
    pub top_n: usize,
    pub show_weights: bool,
}

impl Default for MovContributionConfig {
    fn default() -> Self {
        Self {
            title: "Index Mover Contribution".to_string(),
            theme: Theme::Dark,
            top_n: 10,
            show_weights: true,
        }
    }
}

impl MovContributionConfig {
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
    pub fn top_n(mut self, n: usize) -> Self {
        self.top_n = n.max(3).min(30);
        self
    }
    pub fn show_weights(mut self, s: bool) -> Self {
        self.show_weights = s;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    movers: &[MoverEntry],
    cfg: &MovContributionConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if movers.is_empty() {
        return Err(BtError::EmptySeries("movers".into()));
    }
    fill_background(&root, cfg.theme)?;

    let mut sorted: Vec<MoverEntry> = movers.to_vec();
    sorted.sort_by(|a, b| {
        b.contribution
            .partial_cmp(&a.contribution)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let top: Vec<MoverEntry> = sorted.into_iter().take(cfg.top_n).collect();

    let c_min = top
        .iter()
        .map(|m| m.contribution)
        .fold(f64::INFINITY, f64::min);
    let c_max = top
        .iter()
        .map(|m| m.contribution)
        .fold(f64::NEG_INFINITY, f64::max);
    let pad = (c_max - c_min).max(0.1) * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — Top {}", cfg.title, cfg.top_n),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0..top.len(), (c_min - pad)..(c_max + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .x_labels(top.len())
        .x_label_formatter(&|x| top.get(*x).map(|m| m.symbol.clone()).unwrap_or_default())
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(top.iter().enumerate().map(|(i, m)| {
            let color = if m.contribution >= 0.0 {
                cfg.theme.profit()
            } else {
                cfg.theme.loss()
            };
            Rectangle::new(
                [(i, 0.0_f64.max(c_min - pad)), (i + 1, m.contribution)],
                color.filled(),
            )
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    if cfg.show_weights {
        let (w, h) = root.dim_in_pixel();
        for (i, m) in top.iter().enumerate() {
            root.draw(&Text::new(
                format!("{:.1}%", m.weight * 100.0),
                (w as i32 / 2 - 100 + i as i32 * 20, h as i32 - 60),
                (LABEL_FONT, 10).into_font().color(&cfg.theme.text()),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(movers: &[MoverEntry], cfg: &MovContributionConfig, path: &str) -> Result<()> {
    render(png_root(path)?, movers, cfg)
}

pub fn render_svg(movers: &[MoverEntry], cfg: &MovContributionConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, movers, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let movers: Vec<MoverEntry> = (0..10)
            .map(|i| {
                MoverEntry::new(
                    format!("STK{}", i),
                    (10 - i) as f64 * 0.3,
                    (10 - i) as f64 * 0.02,
                )
            })
            .collect();
        let cfg = MovContributionConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_mov_contribution.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&movers, &cfg, &path).unwrap();
    }
}
