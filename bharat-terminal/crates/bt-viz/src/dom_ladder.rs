// crates/bt-viz/src/dom_ladder.rs
// Author: Sourish Dey

//! DOM: Order-book ladder visualization.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct DomLevel {
    pub price: f64,
    pub bid_size: f64,
    pub ask_size: f64,
}

impl DomLevel {
    pub fn new(price: f64, bid_size: f64, ask_size: f64) -> Self {
        Self {
            price,
            bid_size,
            ask_size,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DomLadderConfig {
    pub title: String,
    pub theme: Theme,
    pub levels: usize,
    pub max_depth: f64,
}

impl Default for DomLadderConfig {
    fn default() -> Self {
        Self {
            title: "Order Book Ladder".to_string(),
            theme: Theme::Dark,
            levels: 15,
            max_depth: 10000.0,
        }
    }
}

impl DomLadderConfig {
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
    pub fn levels(mut self, l: usize) -> Self {
        self.levels = l.max(5).min(50);
        self
    }
    pub fn max_depth(mut self, d: f64) -> Self {
        self.max_depth = d.max(1.0);
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    levels: &[DomLevel],
    cfg: &DomLadderConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if levels.is_empty() {
        return Err(BtError::EmptySeries("DOM levels".into()));
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
    let row_h = (h as usize - 30) / cfg.levels;
    let mid_x = w as usize / 2;

    for (i, level) in levels.iter().enumerate().take(cfg.levels) {
        let y = 30 + i * row_h;
        let bid_frac = (level.bid_size / cfg.max_depth).min(1.0);
        let ask_frac = (level.ask_size / cfg.max_depth).min(1.0);

        let bid_w = (mid_x as f64 * bid_frac) as i32;
        body_area
            .draw(&Rectangle::new(
                [
                    (mid_x as i32 - bid_w, y as i32),
                    (mid_x as i32, (y + row_h) as i32),
                ],
                cfg.theme.profit().mix(0.4).filled(),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;

        let ask_w = (mid_x as f64 * ask_frac) as i32;
        body_area
            .draw(&Rectangle::new(
                [
                    (mid_x as i32, y as i32),
                    (mid_x as i32 + ask_w, (y + row_h) as i32),
                ],
                cfg.theme.loss().mix(0.4).filled(),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;

        body_area
            .draw(&Text::new(
                format!("{:.2}", level.price),
                (mid_x as i32 - 40, y as i32 + 2),
                (LABEL_FONT, 11).into_font().color(&cfg.theme.text()),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    body_area
        .draw(&Text::new(
            "BID".to_string(),
            (mid_x as i32 - 80, 5),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.profit()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    body_area
        .draw(&Text::new(
            "ASK".to_string(),
            (mid_x as i32 + 40, 5),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.loss()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(levels: &[DomLevel], cfg: &DomLadderConfig, path: &str) -> Result<()> {
    render(png_root(path)?, levels, cfg)
}

pub fn render_svg(levels: &[DomLevel], cfg: &DomLadderConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, levels, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let levels: Vec<DomLevel> = (0..15)
            .map(|i| {
                DomLevel::new(
                    100.0 + i as f64 * 0.5,
                    (15 - i) as f64 * 500.0,
                    i as f64 * 500.0,
                )
            })
            .collect();
        let cfg = DomLadderConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_dom_ladder.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&levels, &cfg, &path).unwrap();
    }
}
