// crates/bt-viz/src/walg_life.rs
// Author: Sourish Dey

//! WALG: Weighted average life projection.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct WalgLifeConfig {
    pub title: String,
    pub theme: Theme,
    pub prepayment_speed: f64,
    pub original_wal: f64,
    pub principal: f64,
    pub periods: usize,
}

impl Default for WalgLifeConfig {
    fn default() -> Self {
        Self {
            title: "Weighted Average Life Projection".to_string(),
            theme: Theme::Dark,
            prepayment_speed: 100.0,
            original_wal: 10.0,
            principal: 1_000_000.0,
            periods: 20,
        }
    }
}

impl WalgLifeConfig {
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
    pub fn prepayment_speed(mut self, s: f64) -> Self {
        self.prepayment_speed = s.max(0.0);
        self
    }
    pub fn original_wal(mut self, w: f64) -> Self {
        self.original_wal = w.max(0.1);
        self
    }
    pub fn principal(mut self, p: f64) -> Self {
        self.principal = p.max(0.0);
        self
    }
    pub fn periods(mut self, p: usize) -> Self {
        self.periods = p.max(5).min(50);
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    cfg: &WalgLifeConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    fill_background(&root, cfg.theme)?;

    let speed_factor = cfg.prepayment_speed / 100.0;
    let wal_values: Vec<f64> = (0..cfg.periods)
        .map(|i| {
            let t = i as f64;
            cfg.original_wal * (1.0 - speed_factor * (1.0 - (-t / cfg.original_wal).exp()))
        })
        .collect();

    let w_min = wal_values.iter().cloned().fold(f64::INFINITY, f64::min);
    let w_max = wal_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let pad = (w_max - w_min).max(0.1) * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — PSA {:.0}", cfg.title, cfg.prepayment_speed),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..cfg.periods as f64, (w_min - pad)..(w_max + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .x_desc("Period")
        .y_desc("WAL (years)")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(LineSeries::new(
            wal_values.iter().enumerate().map(|(i, &w)| (i as f64, w)),
            cfg.theme.accent().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Projected WAL")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                cfg.theme.accent().stroke_width(2),
            )
        });

    chart
        .draw_series(LineSeries::new(
            vec![
                (0.0, cfg.original_wal),
                (cfg.periods as f64, cfg.original_wal),
            ],
            cfg.theme.border().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Original WAL")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                cfg.theme.border().mix(0.5).stroke_width(1),
            )
        });

    chart
        .configure_series_labels()
        .background_style(cfg.theme.background().mix(0.9))
        .border_style(cfg.theme.border())
        .label_font((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(cfg: &WalgLifeConfig, path: &str) -> Result<()> {
    render(png_root(path)?, cfg)
}

pub fn render_svg(cfg: &WalgLifeConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let cfg = WalgLifeConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_walg_life.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&cfg, &path).unwrap();
    }
}
