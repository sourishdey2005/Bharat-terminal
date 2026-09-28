// crates/bt-viz/src/yas_analysis.rs
// Author: Sourish Dey

//! YAS: Yield and spread analysis with duration/convexity.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct YasAnalysisConfig {
    pub title: String,
    pub theme: Theme,
    pub spread_tenor: f64,
    pub show_duration: bool,
    pub show_convexity: bool,
    pub bond_yield: f64,
    pub benchmark_yield: f64,
    pub duration: f64,
    pub convexity: f64,
}

impl Default for YasAnalysisConfig {
    fn default() -> Self {
        Self {
            title: "Yield & Spread Analysis".to_string(),
            theme: Theme::Dark,
            spread_tenor: 10.0,
            show_duration: true,
            show_convexity: true,
            bond_yield: 7.5,
            benchmark_yield: 6.5,
            duration: 7.2,
            convexity: 0.8,
        }
    }
}

impl YasAnalysisConfig {
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
    pub fn spread_tenor(mut self, t: f64) -> Self {
        self.spread_tenor = t.max(0.1);
        self
    }
    pub fn show_duration(mut self, s: bool) -> Self {
        self.show_duration = s;
        self
    }
    pub fn show_convexity(mut self, s: bool) -> Self {
        self.show_convexity = s;
        self
    }
    pub fn bond_yield(mut self, y: f64) -> Self {
        self.bond_yield = y;
        self
    }
    pub fn benchmark_yield(mut self, y: f64) -> Self {
        self.benchmark_yield = y;
        self
    }
    pub fn duration(mut self, d: f64) -> Self {
        self.duration = d.max(0.0);
        self
    }
    pub fn convexity(mut self, c: f64) -> Self {
        self.convexity = c.max(0.0);
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    cfg: &YasAnalysisConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    fill_background(&root, cfg.theme)?;

    let spread = cfg.bond_yield - cfg.benchmark_yield;
    let spread_color = if spread >= 0.0 {
        cfg.theme.profit()
    } else {
        cfg.theme.loss()
    };

    let (title_area, body_area) = root.split_vertically(50);
    title_area
        .draw(&Text::new(
            cfg.title.clone(),
            (15, 10),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    let (w, h) = body_area.dim_in_pixel();
    let col_w = w as usize / 3;

    body_area
        .draw(&Text::new(
            format!("Bond Yield: {:.2}%", cfg.bond_yield),
            (10, 10),
            (LABEL_FONT, 14).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    body_area
        .draw(&Text::new(
            format!("Benchmark: {:.2}%", cfg.benchmark_yield),
            (10, 30),
            (LABEL_FONT, 14).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    body_area
        .draw(&Text::new(
            format!("Spread: {:+.2} bps", spread * 100.0),
            (10, 50),
            (LABEL_FONT, 16).into_font().color(&spread_color),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    if cfg.show_duration {
        body_area
            .draw(&Text::new(
                format!("Duration: {:.2}", cfg.duration),
                (col_w as i32 + 10, 10),
                (LABEL_FONT, 14).into_font().color(&cfg.theme.info()),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    if cfg.show_convexity {
        body_area
            .draw(&Text::new(
                format!("Convexity: {:.2}", cfg.convexity),
                (col_w as i32 + 10, 30),
                (LABEL_FONT, 14).into_font().color(&cfg.theme.accent()),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    let bar_y = 80;
    let bar_h = 30;
    let max_spread = 5.0;
    let spread_frac = (spread.abs() / max_spread).min(1.0);
    let bar_w = (col_w as f64 * spread_frac) as i32;

    body_area
        .draw(&Rectangle::new(
            [(10, bar_y), (10 + bar_w, bar_y + bar_h)],
            spread_color.filled(),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    body_area
        .draw(&Text::new(
            format!("{:.1} bps", spread.abs() * 100.0),
            (15, bar_y + 5),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    let (w2, _h2) = root.dim_in_pixel();
    root.draw(&Text::new(
        format!("Tenor: {:.1}Y", cfg.spread_tenor),
        (w2 as i32 - 150, 60),
        (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(cfg: &YasAnalysisConfig, path: &str) -> Result<()> {
    render(png_root(path)?, cfg)
}

pub fn render_svg(cfg: &YasAnalysisConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let cfg = YasAnalysisConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_yas_analysis.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&cfg, &path).unwrap();
    }
}
