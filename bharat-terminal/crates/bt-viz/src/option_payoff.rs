// crates/bt-viz/src/option_payoff.rs
// Author: Sourish Dey

//! Option payoff diagram. Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct OptionPayoffConfig {
    pub title: String,
    pub theme: Theme,
    pub strike: f64,
    pub premium: f64,
    pub is_call: bool,
    pub is_long: bool,
    pub n_points: usize,
}

impl Default for OptionPayoffConfig {
    fn default() -> Self {
        Self {
            title: "Option Payoff".to_string(),
            theme: Theme::Dark,
            strike: 100.0,
            premium: 5.0,
            is_call: true,
            is_long: true,
            n_points: 100,
        }
    }
}

impl OptionPayoffConfig {
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
        self.strike = s;
        self
    }
    pub fn premium(mut self, p: f64) -> Self {
        self.premium = p;
        self
    }
    pub fn is_call(mut self, c: bool) -> Self {
        self.is_call = c;
        self
    }
    pub fn is_long(mut self, l: bool) -> Self {
        self.is_long = l;
        self
    }
    pub fn n_points(mut self, n: usize) -> Self {
        self.n_points = n;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    cfg: &OptionPayoffConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    fill_background(&root, cfg.theme)?;

    let n = cfg.n_points.max(2);
    let s_min = cfg.strike * 0.5;
    let s_max = cfg.strike * 1.5;
    let step = (s_max - s_min) / (n as f64 - 1.0);

    let mut points = Vec::with_capacity(n);
    for i in 0..n {
        let s = s_min + step * i as f64;
        let intrinsic = if cfg.is_call {
            (s - cfg.strike).max(0.0)
        } else {
            (cfg.strike - s).max(0.0)
        };
        let payoff = if cfg.is_long {
            intrinsic - cfg.premium
        } else {
            cfg.premium - intrinsic
        };
        points.push((s, payoff));
    }

    let y_min = points.iter().map(|p| p.1).fold(f64::MAX, f64::min);
    let y_max = points.iter().map(|p| p.1).fold(f64::MIN, f64::max);
    let pad = (y_max - y_min).max(1.0) * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            &cfg.title,
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(15)
        .x_label_area_size(35)
        .y_label_area_size(60)
        .build_cartesian_2d(s_min..s_max, (y_min - pad)..(y_max + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .x_desc("Underlying Price")
        .y_desc("P&L")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(LineSeries::new(
            points.clone(),
            cfg.theme.accent().stroke_width(3),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(s_min, 0.0), (s_max, 0.0)],
            cfg.theme.border().stroke_width(1),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(cfg.strike, y_min - pad), (cfg.strike, y_max + pad)],
            cfg.theme.info().stroke_width(1),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(std::iter::once(Text::new(
            format!("Strike: {:.0}", cfg.strike),
            (cfg.strike, y_max + pad * 0.5),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.info()),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    let label = format!(
        "{} {} (Premium: {:.2})",
        if cfg.is_long { "Long" } else { "Short" },
        if cfg.is_call { "Call" } else { "Put" },
        cfg.premium
    );
    chart
        .draw_series(std::iter::once(Text::new(
            label,
            (s_min + (s_max - s_min) * 0.05, y_max + pad * 0.3),
            (LABEL_FONT, 13).into_font().color(&cfg.theme.accent()),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(cfg: &OptionPayoffConfig, path: &str) -> Result<()> {
    render(png_root(path)?, cfg)
}

pub fn render_svg(cfg: &OptionPayoffConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let cfg = OptionPayoffConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_option_payoff.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&cfg, &path).unwrap();
    }
}
