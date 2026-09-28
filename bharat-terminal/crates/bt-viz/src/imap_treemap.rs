// crates/bt-viz/src/imap_treemap.rs
// Author: Sourish Dey

//! IMAP: Market-cap-sized treemap.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct ImapNode {
    pub label: String,
    pub market_cap: f64,
    pub pct_change: f64,
}

impl ImapNode {
    pub fn new(label: impl Into<String>, market_cap: f64, pct_change: f64) -> Self {
        Self { label: label.into(), market_cap: market_cap.max(0.0001), pct_change }
    }
}

#[derive(Debug, Clone)]
pub struct ImapTreemapConfig {
    pub title: String,
    pub theme: Theme,
    pub min_tile_size: f64,
}

impl Default for ImapTreemapConfig {
    fn default() -> Self {
        Self {
            title: "Market Cap Treemap".to_string(),
            theme: Theme::Dark,
            min_tile_size: 40.0,
        }
    }
}

impl ImapTreemapConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn min_tile_size(mut self, s: f64) -> Self { self.min_tile_size = s.max(10.0); self }
}

fn layout(nodes: &[ImapNode], x: f64, y: f64, w: f64, h: f64) -> Vec<(f64, f64, f64, f64)> {
    if nodes.is_empty() {
        return vec![];
    }
    if nodes.len() == 1 {
        return vec![(x, y, w, h)];
    }
    let total: f64 = nodes.iter().map(|n| n.market_cap).sum();
    let mid = nodes.len() / 2;
    let (left, right) = nodes.split_at(mid);
    let left_total: f64 = left.iter().map(|n| n.market_cap).sum();
    let frac = (left_total / total).clamp(0.05, 0.95);

    let mut out = Vec::with_capacity(nodes.len());
    if w >= h {
        let split_w = w * frac;
        out.extend(layout(left, x, y, split_w, h));
        out.extend(layout(right, x + split_w, y, w - split_w, h));
    } else {
        let split_h = h * frac;
        out.extend(layout(left, x, y, w, split_h));
        out.extend(layout(right, x, y + split_h, w, h - split_h));
    }
    out
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    nodes: &[ImapNode],
    cfg: &ImapTreemapConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    if nodes.is_empty() {
        return Err(BtError::EmptySeries("treemap nodes".into()));
    }
    fill_background(&root, cfg.theme)?;

    let mut sorted: Vec<ImapNode> = nodes.to_vec();
    sorted.sort_by(|a, b| b.market_cap.partial_cmp(&a.market_cap).unwrap_or(std::cmp::Ordering::Equal));

    let (title_area, body_area) = root.split_vertically(50);
    title_area
        .draw(&Text::new(
            cfg.title.clone(),
            (15, 10),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    let (w, h) = body_area.dim_in_pixel();
    let rects = layout(&sorted, 0.0, 0.0, w as f64, (h - 30) as f64);

    for (node, (rx, ry, rw, rh)) in sorted.iter().zip(rects.iter()) {
        let t = (node.pct_change.abs() / 5.0).clamp(0.15, 1.0);
        let target = if node.pct_change >= 0.0 { cfg.theme.profit() } else { cfg.theme.loss() };
        let bg = cfg.theme.background();
        let lerp = |a: u8, b: u8| (a as f64 + (b as f64 - a as f64) * t).round() as u8;
        let color = RGBColor(lerp(bg.0, target.0), lerp(bg.1, target.1), lerp(bg.2, target.2));

        body_area
            .draw(&Rectangle::new(
                [(*rx as i32, *ry as i32), ((*rx + *rw) as i32, (*ry + *rh) as i32)],
                color.filled(),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
        body_area
            .draw(&Rectangle::new(
                [(*rx as i32, *ry as i32), ((*rx + *rw) as i32, (*ry + *rh) as i32)],
                cfg.theme.border().stroke_width(1),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;

        if *rw > cfg.min_tile_size && *rh > 30.0 {
            body_area
                .draw(&Text::new(
                    node.label.clone(),
                    (*rx as i32 + 6, *ry as i32 + 6),
                    (LABEL_FONT, 13).into_font().color(&cfg.theme.text()),
                ))
                .map_err(|e| BtError::Render(e.to_string()))?;
            body_area
                .draw(&Text::new(
                    format!("{:+.2}%", node.pct_change),
                    (*rx as i32 + 6, *ry as i32 + 24),
                    (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
                ))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(nodes: &[ImapNode], cfg: &ImapTreemapConfig, path: &str) -> Result<()> {
    render(png_root(path)?, nodes, cfg)
}

pub fn render_svg(nodes: &[ImapNode], cfg: &ImapTreemapConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, nodes, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let nodes: Vec<ImapNode> = (0..12)
            .map(|i| ImapNode::new(format!("STK{}", i), (12 - i) as f64 * 1000.0, (i as f64 - 6.0) * 0.5))
            .collect();
        let cfg = ImapTreemapConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_imap_treemap.png").to_str().unwrap().to_string();
        render_png(&nodes, &cfg, &path).unwrap();
    }
}
