// crates/bt-viz/src/rolling_moments.rs
// Author: Sourish Dey

//! Rolling skewness and kurtosis chart. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct RollingMomentsConfig {
    pub title: String,
    pub theme: Theme,
    pub window: usize,
}

impl Default for RollingMomentsConfig {
    fn default() -> Self {
        Self {
            title: "Rolling Moments".to_string(),
            theme: Theme::Dark,
            window: 20,
        }
    }
}

impl RollingMomentsConfig {
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
    pub fn window(mut self, w: usize) -> Self {
        self.window = w.max(5);
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &RollingMomentsConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    let returns = series.returns();
    if returns.len() < cfg.window + 1 {
        return Err(BtError::InvalidInput(format!(
            "series too short ({}) for rolling window {}",
            returns.len(),
            cfg.window
        )));
    }
    fill_background(&root, cfg.theme)?;

    let (skew, kurt) = bt_analytics::rolling_moments(&returns, cfg.window);
    let valid_skew: Vec<f64> = skew.iter().copied().filter(|v| !v.is_nan()).collect();
    if valid_skew.is_empty() {
        return Err(BtError::InvalidInput("no valid moments".into()));
    }

    let all_vals: Vec<f64> = valid_skew
        .iter()
        .chain(kurt.iter().filter(|v| !v.is_nan()))
        .copied()
        .collect();
    let y_lo = all_vals.iter().cloned().fold(f64::INFINITY, f64::min) - 0.5;
    let y_hi = all_vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max) + 0.5;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} (window={})", cfg.title, cfg.window),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(15)
        .x_label_area_size(30)
        .y_label_area_size(50)
        .build_cartesian_2d(0f64..returns.len() as f64, y_lo..y_hi)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 11).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Zero reference
    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(0.0, 0.0), (returns.len() as f64, 0.0)],
            cfg.theme.border().stroke_width(1),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    let skew_pts: Vec<(f64, f64)> = skew
        .iter()
        .enumerate()
        .filter(|(_, v)| !v.is_nan())
        .map(|(i, &v)| (i as f64, v))
        .collect();

    chart
        .draw_series(LineSeries::new(
            skew_pts,
            cfg.theme.accent().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Skewness")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 15, y)],
                cfg.theme.accent().stroke_width(2),
            )
        });

    let kurt_pts: Vec<(f64, f64)> = kurt
        .iter()
        .enumerate()
        .filter(|(_, v)| !v.is_nan())
        .map(|(i, &v)| (i as f64, v))
        .collect();

    chart
        .draw_series(LineSeries::new(kurt_pts, cfg.theme.info().stroke_width(2)))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Kurtosis (excess)")
        .legend(|(x, y)| {
            PathElement::new(vec![(x, y), (x + 15, y)], cfg.theme.info().stroke_width(2))
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

pub fn render_png(series: &OhlcvSeries, cfg: &RollingMomentsConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &RollingMomentsConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 200, 1, 100.0);
        let cfg = RollingMomentsConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_rolling_moments.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
