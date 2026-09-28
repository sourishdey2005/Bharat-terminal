// crates/bt-viz/src/ovme_payoff.rs
// Author: Sourish Dey

//! OVME: Option payoff diagrams.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct OvmePayoffConfig {
    pub title: String,
    pub theme: Theme,
    pub strike: f64,
    pub premium: f64,
    pub is_call: bool,
    pub quantity: i32,
}

impl Default for OvmePayoffConfig {
    fn default() -> Self {
        Self {
            title: "Option Payoff Diagram".to_string(),
            theme: Theme::Dark,
            strike: 100.0,
            premium: 5.0,
            is_call: true,
            quantity: 1,
        }
    }
}

impl OvmePayoffConfig {
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
    pub fn strike(mut self, s: f64) -> Self {
        self.strike = s.max(0.0);
        self
    }
    pub fn premium(mut self, p: f64) -> Self {
        self.premium = p.max(0.0);
        self
    }
    pub fn is_call(mut self, c: bool) -> Self {
        self.is_call = c;
        self
    }
    pub fn quantity(mut self, q: i32) -> Self {
        self.quantity = q.max(1);
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    cfg: &OvmePayoffConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    fill_background(&root, cfg.theme)?;

    let s_min = cfg.strike * 0.5;
    let s_max = cfg.strike * 1.5;
    let n_points = 100;

    let payoff: Vec<(f64, f64)> = (0..=n_points)
        .map(|i| {
            let s = s_min + (s_max - s_min) * i as f64 / n_points as f64;
            let raw = if cfg.is_call {
                (s - cfg.strike).max(0.0)
            } else {
                (cfg.strike - s).max(0.0)
            };
            let pnl = (raw - cfg.premium) * cfg.quantity as f64;
            (s, pnl)
        })
        .collect();

    let pnl_min = payoff.iter().map(|(_, p)| *p).fold(f64::INFINITY, f64::min);
    let pnl_max = payoff
        .iter()
        .map(|(_, p)| *p)
        .fold(f64::NEG_INFINITY, f64::max);
    let pad = (pnl_max - pnl_min).max(1.0) * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!(
                "{} — {} {} @ {:.0}",
                cfg.title,
                cfg.quantity,
                if cfg.is_call { "Call" } else { "Put" },
                cfg.strike
            ),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(s_min..s_max, (pnl_min - pad)..(pnl_max + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .x_desc("Underlying Price")
        .y_desc("P&L")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(LineSeries::new(
            vec![(s_min, 0.0), (s_max, 0.0)],
            cfg.theme.border().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(LineSeries::new(
            payoff.clone(),
            cfg.theme.accent().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Payoff")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                cfg.theme.accent().stroke_width(2),
            )
        });

    let strike_pnl = -cfg.premium;
    chart
        .draw_series(std::iter::once(Circle::new(
            (cfg.strike, strike_pnl),
            5,
            cfg.theme.info().filled(),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

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

pub fn render_png(cfg: &OvmePayoffConfig, path: &str) -> Result<()> {
    render(png_root(path)?, cfg)
}

pub fn render_svg(cfg: &OvmePayoffConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let cfg = OvmePayoffConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_ovme_payoff.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&cfg, &path).unwrap();
    }
}
