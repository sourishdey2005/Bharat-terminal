//! Tier 6 #39 / Tier 7 #47 â€” Market Map Treemap (sector x size x change).
//! Made by Sourish Dey.
//!
//! Implements a real squarified-ish slice-and-dice treemap layout (simple,
//! deterministic, and dependency-free) rather than drawing plain, unsized
//! rectangles, so box area actually encodes market cap as Bloomberg's BMAP
//! does.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct TreemapNode {
    pub label: String,
    pub market_cap: f64,
    /// Percent change, used to color the tile (green = up, red = down).
    pub pct_change: f64,
}

impl TreemapNode {
    pub fn new(label: impl Into<String>, market_cap: f64, pct_change: f64) -> Self {
        Self {
            label: label.into(),
            market_cap: market_cap.max(0.0001),
            pct_change,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// Slice-and-dice treemap layout: recursively splits the current
/// rectangle along its longer axis, allocating area proportional to
/// each node's weight. Simpler than a true squarified algorithm but
/// deterministic, dependency-free, and produces reasonable aspect
/// ratios for typical sector-count inputs (10-50 nodes).
pub fn layout(nodes: &[TreemapNode], rect: Rect) -> Vec<Rect> {
    if nodes.is_empty() {
        return vec![];
    }
    if nodes.len() == 1 {
        return vec![rect];
    }
    let total: f64 = nodes.iter().map(|n| n.market_cap).sum();
    let mid = nodes.len() / 2;
    let (left_nodes, right_nodes) = nodes.split_at(mid);
    let left_total: f64 = left_nodes.iter().map(|n| n.market_cap).sum();
    let frac = (left_total / total).clamp(0.05, 0.95);

    let mut out = Vec::with_capacity(nodes.len());
    if rect.w >= rect.h {
        let split_w = rect.w * frac;
        let left_rect = Rect {
            x: rect.x,
            y: rect.y,
            w: split_w,
            h: rect.h,
        };
        let right_rect = Rect {
            x: rect.x + split_w,
            y: rect.y,
            w: rect.w - split_w,
            h: rect.h,
        };
        out.extend(layout(left_nodes, left_rect));
        out.extend(layout(right_nodes, right_rect));
    } else {
        let split_h = rect.h * frac;
        let top_rect = Rect {
            x: rect.x,
            y: rect.y,
            w: rect.w,
            h: split_h,
        };
        let bottom_rect = Rect {
            x: rect.x,
            y: rect.y + split_h,
            w: rect.w,
            h: rect.h - split_h,
        };
        out.extend(layout(left_nodes, top_rect));
        out.extend(layout(right_nodes, bottom_rect));
    }
    out
}

#[derive(Debug, Clone)]
pub struct TreemapConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for TreemapConfig {
    fn default() -> Self {
        Self {
            title: "Market Map".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl TreemapConfig {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

fn change_color(theme: Theme, pct: f64) -> RGBColor {
    let t = (pct.abs() / 5.0).clamp(0.15, 1.0);
    let target = if pct >= 0.0 {
        theme.profit()
    } else {
        theme.loss()
    };
    let bg = theme.background();
    let lerp = |a: u8, b: u8| (a as f64 + (b as f64 - a as f64) * t).round() as u8;
    RGBColor(
        lerp(bg.0, target.0),
        lerp(bg.1, target.1),
        lerp(bg.2, target.2),
    )
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    nodes: &[TreemapNode],
    cfg: &TreemapConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if nodes.is_empty() {
        return Err(BtError::EmptySeries("treemap nodes".into()));
    }
    fill_background(&root, cfg.theme)?;

    let mut sorted: Vec<TreemapNode> = nodes.to_vec();
    sorted.sort_by(|a, b| b.market_cap.partial_cmp(&a.market_cap).unwrap());

    let (title_area, body_area) = root.split_vertically(40);
    title_area
        .draw(&Text::new(
            cfg.title.clone(),
            (15, 8),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    let (w, h) = body_area.dim_in_pixel();
    let full = Rect {
        x: 0.0,
        y: 0.0,
        w: w as f64,
        h: (h - 30) as f64,
    };
    let rects = layout(&sorted, full);

    for (node, r) in sorted.iter().zip(rects.iter()) {
        let color = change_color(cfg.theme, node.pct_change);
        body_area
            .draw(&Rectangle::new(
                [
                    (r.x as i32, r.y as i32),
                    ((r.x + r.w) as i32, (r.y + r.h) as i32),
                ],
                color.filled(),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
        body_area
            .draw(&Rectangle::new(
                [
                    (r.x as i32, r.y as i32),
                    ((r.x + r.w) as i32, (r.y + r.h) as i32),
                ],
                cfg.theme.border().stroke_width(1),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;

        if r.w > 60.0 && r.h > 30.0 {
            body_area
                .draw(&Text::new(
                    node.label.clone(),
                    ((r.x + 6.0) as i32, (r.y + 6.0) as i32),
                    (LABEL_FONT, 13).into_font().color(&cfg.theme.text()),
                ))
                .map_err(|e| BtError::Render(e.to_string()))?;
            body_area
                .draw(&Text::new(
                    format!("{:+.2}%", node.pct_change),
                    ((r.x + 6.0) as i32, (r.y + 24.0) as i32),
                    (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
                ))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(nodes: &[TreemapNode], cfg: &TreemapConfig, path: &str) -> Result<()> {
    render(png_root(path)?, nodes, cfg)
}

pub fn render_svg(nodes: &[TreemapNode], cfg: &TreemapConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, nodes, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_constructs() {
        let cfg = TreemapConfig::new();
        let _ = cfg;
    }
}
