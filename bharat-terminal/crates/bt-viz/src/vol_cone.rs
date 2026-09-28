// crates/bt-viz/src/vol_cone.rs
// Author: Sourish Dey

//! Realized vs implied vol cone. Made by Sourish Dey.

use bt_analytics::rolling_volatility;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct VolConeConfig {
    pub title: String,
    pub theme: Theme,
    pub window: usize,
    pub implied_vol: f64,
    pub periods_per_year: usize,
}

impl Default for VolConeConfig {
    fn default() -> Self {
        Self {
            title: "Volatility Cone".to_string(),
            theme: Theme::Dark,
            window: 20,
            implied_vol: 0.20,
            periods_per_year: 252,
        }
    }
}

impl VolConeConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn window(mut self, w: usize) -> Self { self.window = w; self }
    pub fn implied_vol(mut self, v: f64) -> Self { self.implied_vol = v; self }
    pub fn periods_per_year(mut self, p: usize) -> Self { self.periods_per_year = p; self }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &VolConeConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let returns = series.returns();
    let rv = rolling_volatility(&returns, cfg.window, cfg.periods_per_year);
    let valid: Vec<f64> = rv.iter().filter(|v| v.is_finite()).cloned().collect();
    if valid.is_empty() {
        return Err(BtError::EmptySeries("realized vol".into()));
    }
    let y_max = valid
        .iter()
        .cloned()
        .fold(0.0_f64, f64::max)
        .max(cfg.implied_vol * 100.0);
    let n = rv.len() as f64;

    let mut chart = ChartBuilder::on(&root)
        .caption(&cfg.title, (TITLE_FONT, 22).into_font().color(&cfg.theme.text()))
        .margin(15)
        .x_label_area_size(35)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..n, 0.0..(y_max * 1.2))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .x_desc("Bar")
        .y_desc("Volatility (%)")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let points: Vec<(f64, f64)> = rv
        .iter()
        .enumerate()
        .filter(|(_, v)| v.is_finite())
        .map(|(i, v)| (i as f64, *v * 100.0))
        .collect();

    chart
        .draw_series(AreaSeries::new(
            points.clone(),
            0.0,
            cfg.theme.info().mix(0.3),
        ).border_style(cfg.theme.info().stroke_width(2)))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Realized Vol")
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.info().stroke_width(2)));

    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(0.0, cfg.implied_vol * 100.0), (n, cfg.implied_vol * 100.0)],
            cfg.theme.accent().stroke_width(2),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label(format!("Implied Vol: {:.1}%", cfg.implied_vol * 100.0))
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.accent().stroke_width(2)));

    chart
        .configure_series_labels()
        .background_style(cfg.theme.background().mix(0.8))
        .border_style(cfg.theme.border())
        .label_font((LABEL_FONT, 13).into_font().color(&cfg.theme.text()))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &VolConeConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &VolConeConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = VolConeConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_vol_cone.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
