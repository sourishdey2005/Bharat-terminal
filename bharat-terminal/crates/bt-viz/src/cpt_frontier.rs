// crates/bt-viz/src/cpt_frontier.rs
// Author: Sourish Dey

//! CPT: Efficient frontier.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CptFrontierConfig {
    pub title: String,
    pub theme: Theme,
    pub n_points: usize,
    pub show_assets: bool,
    pub risk_free: f64,
}

impl Default for CptFrontierConfig {
    fn default() -> Self {
        Self {
            title: "Efficient Frontier".to_string(),
            theme: Theme::Dark,
            n_points: 20,
            show_assets: true,
            risk_free: 0.05,
        }
    }
}

impl CptFrontierConfig {
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
    pub fn n_points(mut self, n: usize) -> Self {
        self.n_points = n.max(5).min(100);
        self
    }
    pub fn show_assets(mut self, s: bool) -> Self {
        self.show_assets = s;
        self
    }
    pub fn risk_free(mut self, r: f64) -> Self {
        self.risk_free = r.max(0.0);
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    cfg: &CptFrontierConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    fill_background(&root, cfg.theme)?;

    let assets: Vec<(f64, f64)> = vec![
        (0.15, 0.10),
        (0.20, 0.15),
        (0.12, 0.08),
        (0.25, 0.18),
        (0.18, 0.12),
        (0.30, 0.22),
    ];

    let mut frontier: Vec<(f64, f64)> = Vec::with_capacity(cfg.n_points);
    for i in 0..cfg.n_points {
        let t = i as f64 / (cfg.n_points - 1).max(1) as f64;
        let risk = 0.10 + t * 0.20;
        let ret = cfg.risk_free + 0.08 * (1.0 - (-risk / 0.15).exp());
        frontier.push((risk, ret));
    }

    let mut all_risks: Vec<f64> = frontier.iter().map(|(r, _)| *r).collect();
    let mut all_rets: Vec<f64> = frontier.iter().map(|(_, r)| *r).collect();
    if cfg.show_assets {
        for (risk, ret) in &assets {
            all_risks.push(*risk);
            all_rets.push(*ret);
        }
    }

    let r_min = all_risks.iter().cloned().fold(f64::INFINITY, f64::min);
    let r_max = all_risks.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let ret_min = all_rets.iter().cloned().fold(f64::INFINITY, f64::min);
    let ret_max = all_rets.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let r_pad = (r_max - r_min).max(0.01) * 0.1;
    let ret_pad = (ret_max - ret_min).max(0.01) * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — RF: {:.1}%", cfg.title, cfg.risk_free * 100.0),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(
            (r_min - r_pad)..(r_max + r_pad),
            (ret_min - ret_pad)..(ret_max + ret_pad),
        )
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .x_desc("Risk (Volatility)")
        .y_desc("Expected Return")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(LineSeries::new(
            frontier.clone(),
            cfg.theme.accent().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Efficient Frontier")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                cfg.theme.accent().stroke_width(2),
            )
        });

    if cfg.show_assets {
        chart
            .draw_series(
                assets
                    .iter()
                    .map(|(risk, ret)| Circle::new((*risk, *ret), 5, cfg.theme.info().filled())),
            )
            .map_err(|e| BtError::Render(e.to_string()))?
            .label("Assets")
            .legend(|(x, y)| Circle::new((x + 10, y), 4, cfg.theme.info().filled()));
    }

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

pub fn render_png(cfg: &CptFrontierConfig, path: &str) -> Result<()> {
    render(png_root(path)?, cfg)
}

pub fn render_svg(cfg: &CptFrontierConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let cfg = CptFrontierConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_cpt_frontier.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&cfg, &path).unwrap();
    }
}
