// crates/bt-viz/src/ddis_distribution.rs
// Author: Sourish Dey

//! DDIS: Debt maturity distribution.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct MaturityBucket {
    pub label: String,
    pub amount: f64,
    pub rate: f64,
}

impl MaturityBucket {
    pub fn new(label: impl Into<String>, amount: f64, rate: f64) -> Self {
        Self {
            label: label.into(),
            amount: amount.max(0.0),
            rate,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DdisDistributionConfig {
    pub title: String,
    pub theme: Theme,
    pub buckets: usize,
    pub show_rates: bool,
}

impl Default for DdisDistributionConfig {
    fn default() -> Self {
        Self {
            title: "Debt Maturity Distribution".to_string(),
            theme: Theme::Dark,
            buckets: 8,
            show_rates: true,
        }
    }
}

impl DdisDistributionConfig {
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
    pub fn buckets(mut self, b: usize) -> Self {
        self.buckets = b.max(3).min(20);
        self
    }
    pub fn show_rates(mut self, s: bool) -> Self {
        self.show_rates = s;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    buckets: &[MaturityBucket],
    cfg: &DdisDistributionConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if buckets.is_empty() {
        return Err(BtError::EmptySeries("maturity buckets".into()));
    }
    fill_background(&root, cfg.theme)?;

    let max_amount = buckets.iter().map(|b| b.amount).fold(0.0_f64, f64::max);
    let max_rate = buckets.iter().map(|b| b.rate).fold(0.0_f64, f64::max);

    let (amount_area, rate_area) = if cfg.show_rates {
        let split = root.split_vertically((60).percent());
        (split.0, Some(split.1))
    } else {
        (root.clone(), None)
    };

    let mut amount_chart = ChartBuilder::on(&amount_area)
        .caption(
            cfg.title.clone(),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(80)
        .build_cartesian_2d(0..buckets.len(), 0.0..max_amount * 1.1)
        .map_err(|e| BtError::Render(e.to_string()))?;

    amount_chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .x_labels(buckets.len())
        .x_label_formatter(&|x| buckets.get(*x).map(|b| b.label.clone()).unwrap_or_default())
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    amount_chart
        .draw_series(
            buckets.iter().enumerate().map(|(i, b)| {
                Rectangle::new([(i, 0.0), (i + 1, b.amount)], cfg.theme.info().filled())
            }),
        )
        .map_err(|e| BtError::Render(e.to_string()))?;

    if let Some(rate_area) = rate_area {
        let mut rate_chart = ChartBuilder::on(&rate_area)
            .caption(
                "Rates",
                (TITLE_FONT, 16).into_font().color(&cfg.theme.text()),
            )
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(60)
            .build_cartesian_2d(0..buckets.len(), 0.0..max_rate * 1.2)
            .map_err(|e| BtError::Render(e.to_string()))?;

        rate_chart
            .configure_mesh()
            .label_style((LABEL_FONT, 10).into_font().color(&cfg.theme.text()))
            .axis_style(&cfg.theme.border())
            .disable_x_mesh()
            .draw()
            .map_err(|e| BtError::Render(e.to_string()))?;

        rate_chart
            .draw_series(LineSeries::new(
                buckets.iter().enumerate().map(|(i, b)| (i, b.rate)),
                cfg.theme.accent().stroke_width(2),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;

        rate_chart
            .draw_series(
                buckets
                    .iter()
                    .enumerate()
                    .map(|(i, b)| Circle::new((i, b.rate), 4, cfg.theme.accent().filled())),
            )
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(
    buckets: &[MaturityBucket],
    cfg: &DdisDistributionConfig,
    path: &str,
) -> Result<()> {
    render(png_root(path)?, buckets, cfg)
}

pub fn render_svg(
    buckets: &[MaturityBucket],
    cfg: &DdisDistributionConfig,
    path: &str,
) -> Result<()> {
    render(svg_root(path)?, buckets, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let buckets: Vec<MaturityBucket> = (0..8)
            .map(|i| {
                MaturityBucket::new(
                    format!("{}Y", i + 1),
                    (8 - i) as f64 * 100.0,
                    5.0 + i as f64 * 0.3,
                )
            })
            .collect();
        let cfg = DdisDistributionConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_ddis_distribution.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&buckets, &cfg, &path).unwrap();
    }
}
