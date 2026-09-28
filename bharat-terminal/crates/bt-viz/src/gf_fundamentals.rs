// crates/bt-viz/src/gf_fundamentals.rs
// Author: Sourish Dey

//! GF: Fundamentals time series.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct FundamentalPoint {
    pub period: String,
    pub revenue: f64,
    pub net_income: f64,
    pub eps: f64,
    pub pe_ratio: f64,
}

impl FundamentalPoint {
    pub fn new(
        period: impl Into<String>,
        revenue: f64,
        net_income: f64,
        eps: f64,
        pe_ratio: f64,
    ) -> Self {
        Self {
            period: period.into(),
            revenue,
            net_income,
            eps,
            pe_ratio,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GfFundamentalsConfig {
    pub title: String,
    pub theme: Theme,
    pub metrics: Vec<String>,
}

impl Default for GfFundamentalsConfig {
    fn default() -> Self {
        Self {
            title: "Fundamentals Time Series".to_string(),
            theme: Theme::Dark,
            metrics: vec!["revenue".into(), "net_income".into(), "eps".into()],
        }
    }
}

impl GfFundamentalsConfig {
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
    pub fn metrics(mut self, m: Vec<String>) -> Self {
        self.metrics = m;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    data: &[FundamentalPoint],
    cfg: &GfFundamentalsConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if data.is_empty() {
        return Err(BtError::EmptySeries("fundamental data".into()));
    }
    fill_background(&root, cfg.theme)?;

    let colors = [
        cfg.theme.profit(),
        cfg.theme.info(),
        cfg.theme.accent(),
        cfg.theme.loss(),
    ];

    let mut all_values: Vec<f64> = Vec::new();
    for point in data {
        for metric in &cfg.metrics {
            match metric.as_str() {
                "revenue" => all_values.push(point.revenue),
                "net_income" => all_values.push(point.net_income),
                "eps" => all_values.push(point.eps),
                "pe_ratio" => all_values.push(point.pe_ratio),
                _ => {}
            }
        }
    }

    let v_min = all_values.iter().cloned().fold(f64::INFINITY, f64::min);
    let v_max = all_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let pad = (v_max - v_min).max(1.0) * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            cfg.title.clone(),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(80)
        .build_cartesian_2d(0..data.len(), (v_min - pad)..(v_max + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .x_labels(data.len())
        .x_label_formatter(&|x| data.get(*x).map(|p| p.period.clone()).unwrap_or_default())
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    for (mi, metric) in cfg.metrics.iter().enumerate() {
        let color = colors[mi % colors.len()];
        let values: Vec<f64> = data
            .iter()
            .map(|p| match metric.as_str() {
                "revenue" => p.revenue,
                "net_income" => p.net_income,
                "eps" => p.eps,
                "pe_ratio" => p.pe_ratio,
                _ => 0.0,
            })
            .collect();

        chart
            .draw_series(LineSeries::new(
                values.iter().enumerate().map(|(i, &v)| (i, v)),
                color.stroke_width(2),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?
            .label(metric)
            .legend(move |(x, y)| {
                PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2))
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

pub fn render_png(data: &[FundamentalPoint], cfg: &GfFundamentalsConfig, path: &str) -> Result<()> {
    render(png_root(path)?, data, cfg)
}

pub fn render_svg(data: &[FundamentalPoint], cfg: &GfFundamentalsConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, data, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let data: Vec<FundamentalPoint> = (0..8)
            .map(|i| {
                FundamentalPoint::new(
                    format!("Q{}", i + 1),
                    1000.0 + i as f64 * 100.0,
                    200.0 + i as f64 * 20.0,
                    10.0 + i as f64 * 2.0,
                    15.0 + i as f64 * 0.5,
                )
            })
            .collect();
        let cfg = GfFundamentalsConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_gf_fundamentals.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&data, &cfg, &path).unwrap();
    }
}
