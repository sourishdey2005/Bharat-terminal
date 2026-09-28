// crates/bt-viz/src/usdinr_chart.rs
// Author: Sourish Dey

//! USD/INR chart with Bollinger-style bands. Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

/// USD/INR rate data point.
#[derive(Debug, Clone)]
pub struct FxPoint {
    pub date: String,
    pub rate: f64,
}

/// Sample USD/INR data (60 days).
fn sample_usdinr() -> Vec<FxPoint> {
    let mut data = Vec::new();
    let mut rate = 83.20;
    for i in 0..60 {
        let day = i + 1;
        let month = if day <= 30 { "Aug" } else { "Sep" };
        let d = if day <= 30 { day } else { day - 30 };
        let shock = ((i * 7 + 3) % 13) as f64 / 100.0 - 0.06;
        rate += shock;
        data.push(FxPoint {
            date: format!("{:02} {}", d, month),
            rate,
        });
    }
    data
}

#[derive(Debug, Clone)]
pub struct UsdInrConfig {
    pub title: String,
    pub theme: Theme,
    pub band_mult: f64,
}

impl Default for UsdInrConfig {
    fn default() -> Self {
        Self {
            title: "USD/INR".to_string(),
            theme: Theme::Dark,
            band_mult: 2.0,
        }
    }
}

impl UsdInrConfig {
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
    pub fn band_mult(mut self, m: f64) -> Self {
        self.band_mult = m.max(0.5);
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    data: &[FxPoint],
    cfg: &UsdInrConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if data.len() < 20 {
        return Err(BtError::InvalidInput("need at least 20 data points".into()));
    }
    fill_background(&root, cfg.theme)?;

    let window = 20;
    let rates: Vec<f64> = data.iter().map(|p| p.rate).collect();
    let mut upper = vec![f64::NAN; rates.len()];
    let mut lower = vec![f64::NAN; rates.len()];
    let mut middle = vec![f64::NAN; rates.len()];

    for i in (window - 1)..rates.len() {
        let slice = &rates[i + 1 - window..=i];
        let mean = slice.iter().sum::<f64>() / window as f64;
        let std = (slice.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / window as f64).sqrt();
        middle[i] = mean;
        upper[i] = mean + cfg.band_mult * std;
        lower[i] = mean - cfg.band_mult * std;
    }

    let y_lo = lower
        .iter()
        .filter(|v| !v.is_nan())
        .cloned()
        .fold(f64::INFINITY, f64::min);
    let y_hi = upper
        .iter()
        .filter(|v| !v.is_nan())
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let pad = (y_hi - y_lo) * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            &cfg.title,
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(15)
        .x_label_area_size(35)
        .y_label_area_size(55)
        .build_cartesian_2d(0f64..data.len() as f64, y_lo - pad..y_hi + pad)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .x_labels(10)
        .x_label_formatter(&|idx| {
            data.get(*idx as usize)
                .map(|p| p.date.clone())
                .unwrap_or_default()
        })
        .y_desc("INR per USD")
        .label_style((LABEL_FONT, 11).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Band fill
    let band_pts: Vec<(f64, f64)> = upper
        .iter()
        .enumerate()
        .filter(|(_, v)| !v.is_nan())
        .map(|(i, &v)| (i as f64, v))
        .collect();
    let band_pts_lower: Vec<(f64, f64)> = lower
        .iter()
        .enumerate()
        .filter(|(_, v)| !v.is_nan())
        .map(|(i, &v)| (i as f64, v))
        .collect();

    let mut band_polygon = band_pts.clone();
    band_polygon.extend(band_pts_lower.iter().rev().cloned());

    chart
        .draw_series(std::iter::once(Polygon::new(
            band_polygon,
            cfg.theme.info().mix(0.1).filled(),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Middle band
    let mid_pts: Vec<(f64, f64)> = middle
        .iter()
        .enumerate()
        .filter(|(_, v)| !v.is_nan())
        .map(|(i, &v)| (i as f64, v))
        .collect();
    chart
        .draw_series(LineSeries::new(mid_pts, cfg.theme.info().stroke_width(1)))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("SMA(20)")
        .legend(|(x, y)| {
            PathElement::new(vec![(x, y), (x + 15, y)], cfg.theme.info().stroke_width(1))
        });

    // Rate line
    let rate_pts: Vec<(f64, f64)> = rates
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v))
        .collect();
    chart
        .draw_series(LineSeries::new(
            rate_pts,
            cfg.theme.accent().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("USD/INR")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 15, y)],
                cfg.theme.accent().stroke_width(2),
            )
        });

    chart
        .configure_series_labels()
        .background_style(cfg.theme.background())
        .label_font((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(data: &[FxPoint], cfg: &UsdInrConfig, path: &str) -> Result<()> {
    render(png_root(path)?, data, cfg)
}

pub fn render_svg(data: &[FxPoint], cfg: &UsdInrConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, data, cfg)
}

pub fn render_sample_png(cfg: &UsdInrConfig, path: &str) -> Result<()> {
    let data = sample_usdinr();
    render_png(&data, cfg, path)
}

pub fn render_sample_svg(cfg: &UsdInrConfig, path: &str) -> Result<()> {
    let data = sample_usdinr();
    render_svg(&data, cfg, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let data = sample_usdinr();
        let cfg = UsdInrConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_usdinr.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&data, &cfg, &path).unwrap();
    }
}
