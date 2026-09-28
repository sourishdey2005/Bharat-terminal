// crates/bt-viz/src/mrr_ranked.rs
// Author: Sourish Dey

//! MRR: Member ranked returns.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct RankedEntry {
    pub name: String,
    pub return_pct: f64,
    pub market_cap: f64,
}

impl RankedEntry {
    pub fn new(name: impl Into<String>, return_pct: f64, market_cap: f64) -> Self {
        Self { name: name.into(), return_pct, market_cap: market_cap.max(0.0) }
    }
}

#[derive(Debug, Clone)]
pub struct MrrRankedConfig {
    pub title: String,
    pub theme: Theme,
    pub top_n: usize,
    pub ascending: bool,
}

impl Default for MrrRankedConfig {
    fn default() -> Self {
        Self {
            title: "Member Ranked Returns".to_string(),
            theme: Theme::Dark,
            top_n: 15,
            ascending: false,
        }
    }
}

impl MrrRankedConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn top_n(mut self, n: usize) -> Self { self.top_n = n.max(3).min(50); self }
    pub fn ascending(mut self, a: bool) -> Self { self.ascending = a; self }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    entries: &[RankedEntry],
    cfg: &MrrRankedConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    if entries.is_empty() {
        return Err(BtError::EmptySeries("ranked entries".into()));
    }
    fill_background(&root, cfg.theme)?;

    let mut sorted: Vec<RankedEntry> = entries.to_vec();
    sorted.sort_by(|a, b| {
        if cfg.ascending {
            a.return_pct.partial_cmp(&b.return_pct).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            b.return_pct.partial_cmp(&a.return_pct).unwrap_or(std::cmp::Ordering::Equal)
        }
    });
    let top: Vec<RankedEntry> = sorted.into_iter().take(cfg.top_n).collect();

    let r_min = top.iter().map(|e| e.return_pct).fold(f64::INFINITY, f64::min);
    let r_max = top.iter().map(|e| e.return_pct).fold(f64::NEG_INFINITY, f64::max);
    let pad = (r_max - r_min).max(0.1) * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — Top {}", cfg.title, cfg.top_n),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0..top.len(), (r_min - pad)..(r_max + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .x_labels(top.len())
        .x_label_formatter(&|x| top.get(*x).map(|e| e.name.clone()).unwrap_or_default())
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(top.iter().enumerate().map(|(i, e)| {
            let color = if e.return_pct >= 0.0 { cfg.theme.profit() } else { cfg.theme.loss() };
            Rectangle::new(
                [(i, 0.0_f64.max(r_min - pad)), (i + 1, e.return_pct)],
                color.filled(),
            )
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(entries: &[RankedEntry], cfg: &MrrRankedConfig, path: &str) -> Result<()> {
    render(png_root(path)?, entries, cfg)
}

pub fn render_svg(entries: &[RankedEntry], cfg: &MrrRankedConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, entries, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let entries: Vec<RankedEntry> = (0..15)
            .map(|i| RankedEntry::new(format!("MEM{}", i), (15 - i) as f64 * 0.8 - 5.0, (15 - i) as f64 * 100.0))
            .collect();
        let cfg = MrrRankedConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_mrr_ranked.png").to_str().unwrap().to_string();
        render_png(&entries, &cfg, &path).unwrap();
    }
}
