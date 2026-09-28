// crates/bt-viz/src/factor_exposure.rs
// Author: Sourish Dey

//! Factor risk exposure bars.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct FactorExposure {
    pub name: String,
    pub exposure: f64,
    pub contribution: f64,
}

impl FactorExposure {
    pub fn new(name: impl Into<String>, exposure: f64, contribution: f64) -> Self {
        Self { name: name.into(), exposure, contribution }
    }
}

#[derive(Debug, Clone)]
pub struct FactorExposureConfig {
    pub title: String,
    pub theme: Theme,
    pub show_contribution: bool,
}

impl Default for FactorExposureConfig {
    fn default() -> Self {
        Self {
            title: "Factor Risk Exposure".to_string(),
            theme: Theme::Dark,
            show_contribution: true,
        }
    }
}

impl FactorExposureConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn show_contribution(mut self, s: bool) -> Self { self.show_contribution = s; self }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    factors: &[FactorExposure],
    cfg: &FactorExposureConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    if factors.is_empty() {
        return Err(BtError::EmptySeries("factor exposures".into()));
    }
    fill_background(&root, cfg.theme)?;

    let mut e_min = f64::INFINITY;
    let mut e_max = f64::NEG_INFINITY;
    for f in factors {
        e_min = e_min.min(f.exposure);
        e_max = e_max.max(f.exposure);
        if cfg.show_contribution {
            e_min = e_min.min(f.contribution);
            e_max = e_max.max(f.contribution);
        }
    }
    let pad = (e_max - e_min).max(0.1) * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            cfg.title.clone(),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0..factors.len(), (e_min - pad)..(e_max + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .x_labels(factors.len())
        .x_label_formatter(&|x| factors.get(*x).map(|f| f.name.clone()).unwrap_or_default())
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(factors.iter().enumerate().map(|(i, f)| {
            let color = if f.exposure >= 0.0 { cfg.theme.profit() } else { cfg.theme.loss() };
            Rectangle::new([(i, 0.0_f64.max(e_min - pad)), (i + 1, f.exposure)], color.filled())
        }))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Exposure")
        .legend(|(x, y)| {
            Rectangle::new([(x, y - 5), (x + 20, y + 5)], cfg.theme.profit().filled())
        });

    if cfg.show_contribution {
        chart
            .draw_series(factors.iter().enumerate().map(|(i, f)| {
                let color = if f.contribution >= 0.0 { cfg.theme.info() } else { cfg.theme.accent() };
                Rectangle::new([(i, 0.0_f64.max(e_min - pad)), (i + 1, f.contribution)], color.mix(0.5).filled())
            }))
            .map_err(|e| BtError::Render(e.to_string()))?
            .label("Contribution")
            .legend(|(x, y)| {
                Rectangle::new([(x, y - 5), (x + 20, y + 5)], cfg.theme.info().mix(0.5).filled())
            });
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

pub fn render_png(factors: &[FactorExposure], cfg: &FactorExposureConfig, path: &str) -> Result<()> {
    render(png_root(path)?, factors, cfg)
}

pub fn render_svg(factors: &[FactorExposure], cfg: &FactorExposureConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, factors, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let factors: Vec<FactorExposure> = (0..8)
            .map(|i| FactorExposure::new(format!("F{}", i), (8 - i) as f64 * 0.1 - 0.4, (8 - i) as f64 * 0.05 - 0.2))
            .collect();
        let cfg = FactorExposureConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_factor_exposure.png").to_str().unwrap().to_string();
        render_png(&factors, &cfg, &path).unwrap();
    }
}
