// crates/bt-viz/src/eco_surprise.rs
// Author: Sourish Dey

//! ECO: Economic surprise indicator.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct EcoDataPoint {
    pub period: String,
    pub actual: f64,
    pub forecast: f64,
    pub surprise: f64,
}

impl EcoDataPoint {
    pub fn new(period: impl Into<String>, actual: f64, forecast: f64) -> Self {
        let surprise = actual - forecast;
        Self {
            period: period.into(),
            actual,
            forecast,
            surprise,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EcoSurpriseConfig {
    pub title: String,
    pub theme: Theme,
    pub show_forecast: bool,
}

impl Default for EcoSurpriseConfig {
    fn default() -> Self {
        Self {
            title: "Economic Surprise Indicator".to_string(),
            theme: Theme::Dark,
            show_forecast: true,
        }
    }
}

impl EcoSurpriseConfig {
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
    pub fn show_forecast(mut self, s: bool) -> Self {
        self.show_forecast = s;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    data: &[EcoDataPoint],
    cfg: &EcoSurpriseConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if data.is_empty() {
        return Err(BtError::EmptySeries("economic data".into()));
    }
    fill_background(&root, cfg.theme)?;

    let s_min = data
        .iter()
        .map(|d| d.surprise)
        .fold(f64::INFINITY, f64::min);
    let s_max = data
        .iter()
        .map(|d| d.surprise)
        .fold(f64::NEG_INFINITY, f64::max);
    let pad = (s_max - s_min).max(0.1) * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            cfg.title.clone(),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0..data.len(), (s_min - pad)..(s_max + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .x_labels(data.len())
        .x_label_formatter(&|x| data.get(*x).map(|d| d.period.clone()).unwrap_or_default())
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(LineSeries::new(
            vec![(0, 0.0), (data.len(), 0.0)],
            cfg.theme.border().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(data.iter().enumerate().map(|(i, d)| {
            let color = if d.surprise >= 0.0 {
                cfg.theme.profit()
            } else {
                cfg.theme.loss()
            };
            Rectangle::new(
                [(i, 0.0_f64.max(s_min - pad)), (i + 1, d.surprise)],
                color.filled(),
            )
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    if cfg.show_forecast {
        chart
            .draw_series(LineSeries::new(
                data.iter().enumerate().map(|(i, d)| (i, d.forecast)),
                cfg.theme.info().mix(0.5).stroke_width(1),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?
            .label("Forecast")
            .legend(|(x, y)| {
                PathElement::new(
                    vec![(x, y), (x + 20, y)],
                    cfg.theme.info().mix(0.5).stroke_width(1),
                )
            });

        chart
            .configure_series_labels()
            .background_style(cfg.theme.background().mix(0.9))
            .border_style(cfg.theme.border())
            .label_font((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
            .draw()
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(data: &[EcoDataPoint], cfg: &EcoSurpriseConfig, path: &str) -> Result<()> {
    render(png_root(path)?, data, cfg)
}

pub fn render_svg(data: &[EcoDataPoint], cfg: &EcoSurpriseConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, data, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let data: Vec<EcoDataPoint> = (0..10)
            .map(|i| {
                EcoDataPoint::new(
                    format!("M{}", i + 1),
                    5.0 + (i as f64 * 0.3),
                    5.0 + (i as f64 * 0.2),
                )
            })
            .collect();
        let cfg = EcoSurpriseConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_eco_surprise.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&data, &cfg, &path).unwrap();
    }
}
