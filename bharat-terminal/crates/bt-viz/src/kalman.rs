// crates/bt-viz/src/kalman.rs
// Author: Sourish Dey

//! Kalman filter state-space smoothing chart. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

/// 1-D Kalman filter: estimates a hidden price level from noisy observations.
/// Returns (filtered, predicted) price series.
pub fn kalman_filter(
    observations: &[f64],
    process_var: f64,
    measurement_var: f64,
) -> (Vec<f64>, Vec<f64>) {
    let n = observations.len();
    let mut filtered = vec![0.0; n];
    let mut predicted = vec![0.0; n];

    if n == 0 {
        return (filtered, predicted);
    }

    let mut x = observations[0];
    let mut p = 1.0;
    filtered[0] = x;
    predicted[0] = x;

    for i in 1..n {
        // Predict
        let p_pred = p + process_var;
        // Update
        let k = p_pred / (p_pred + measurement_var);
        x = x + k * (observations[i] - x);
        p = (1.0 - k) * p_pred;
        filtered[i] = x;
        predicted[i] = x;
    }

    (filtered, predicted)
}

#[derive(Debug, Clone)]
pub struct KalmanConfig {
    pub title: String,
    pub theme: Theme,
    pub process_var: f64,
    pub measurement_var: f64,
}

impl Default for KalmanConfig {
    fn default() -> Self {
        Self {
            title: "Kalman Filter".to_string(),
            theme: Theme::Dark,
            process_var: 0.01,
            measurement_var: 1.0,
        }
    }
}

impl KalmanConfig {
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
    pub fn process_var(mut self, v: f64) -> Self {
        self.process_var = v.max(1e-6);
        self
    }
    pub fn measurement_var(mut self, v: f64) -> Self {
        self.measurement_var = v.max(1e-6);
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &KalmanConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    let closes = series.closes();
    if closes.len() < 2 {
        return Err(BtError::InvalidInput("series too short for kalman".into()));
    }
    fill_background(&root, cfg.theme)?;

    let (filtered, _) = kalman_filter(&closes, cfg.process_var, cfg.measurement_var);

    let all_vals: Vec<f64> = closes.iter().chain(filtered.iter()).copied().collect();
    let y_lo = all_vals
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min);
    let y_hi = all_vals
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let pad = (y_hi - y_lo) * 0.05;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} (Q={:.4}, Q={:.4})", cfg.title, cfg.process_var, cfg.measurement_var),
            (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
        )
        .margin(15)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..closes.len() as f64, y_lo - pad..y_hi + pad)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 11).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let obs_pts: Vec<(f64, f64)> = closes
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v))
        .collect();

    chart
        .draw_series(LineSeries::new(obs_pts, cfg.theme.text().mix(0.3).stroke_width(1)))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Observed")
        .legend(|(x, y)| {
            Rectangle::new(
                [(x, y), (x + 10, y + 10)],
                cfg.theme.text().mix(0.3).filled(),
            )
        });

    let filt_pts: Vec<(f64, f64)> = filtered
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v))
        .collect();

    chart
        .draw_series(LineSeries::new(filt_pts, cfg.theme.accent().stroke_width(2)))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Filtered")
        .legend(|(x, y)| {
            Rectangle::new([(x, y), (x + 10, y + 10)], cfg.theme.accent().filled())
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

pub fn render_png(series: &OhlcvSeries, cfg: &KalmanConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &KalmanConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = KalmanConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_kalman.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
