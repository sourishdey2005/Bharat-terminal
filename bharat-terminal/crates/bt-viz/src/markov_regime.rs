// crates/bt-viz/src/markov_regime.rs
// Author: Sourish Dey

//! Markov regime-switching diagram. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

/// Two-state Markov regime classification: Bull (1) and Bear (-1).
/// Uses a simple threshold on smoothed returns with persistence.
pub fn classify_regimes(returns: &[f64], smooth_window: usize) -> Vec<i8> {
    let mut regimes = vec![1i8; returns.len()];
    if returns.is_empty() {
        return regimes;
    }

    let mut smoothed = vec![0.0; returns.len()];
    for i in 0..returns.len() {
        let lo = i.saturating_sub(smooth_window - 1);
        let slice = &returns[lo..=i];
        smoothed[i] = slice.iter().sum::<f64>() / slice.len() as f64;
    }

    let mut state = if smoothed[0] >= 0.0 { 1i8 } else { -1i8 };
    regimes[0] = state;

    for i in 1..smoothed.len() {
        let threshold = 0.002_f64;
        if state == 1 && smoothed[i] < -threshold {
            state = -1;
        } else if state == -1 && smoothed[i] > threshold {
            state = 1;
        }
        regimes[i] = state;
    }

    regimes
}

#[derive(Debug, Clone)]
pub struct MarkovRegimeConfig {
    pub title: String,
    pub theme: Theme,
    pub smooth_window: usize,
}

impl Default for MarkovRegimeConfig {
    fn default() -> Self {
        Self {
            title: "Markov Regimes".to_string(),
            theme: Theme::Dark,
            smooth_window: 5,
        }
    }
}

impl MarkovRegimeConfig {
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
    pub fn smooth_window(mut self, w: usize) -> Self {
        self.smooth_window = w.max(2);
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &MarkovRegimeConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    let returns = series.returns();
    if returns.len() < 3 {
        return Err(BtError::InvalidInput("series too short".into()));
    }
    fill_background(&root, cfg.theme)?;

    let regimes = classify_regimes(&returns, cfg.smooth_window);
    let (w, h) = root.dim_in_pixel();
    let top_pad = 50;
    let bottom_pad = 50;
    let chart_h = h as f64 - top_pad as f64 - bottom_pad as f64;

    // Cumulative returns for y-scale
    let mut cum = vec![0.0_f64; returns.len()];
    let mut acc = 0.0;
    for (i, &r) in returns.iter().enumerate() {
        acc += r;
        cum[i] = acc;
    }
    let y_lo = cum.iter().cloned().fold(f64::INFINITY, f64::min);
    let y_hi = cum.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let pad = (y_hi - y_lo).abs() * 0.1 + 0.01;

    root.draw(&Text::new(
        cfg.title.as_str(),
        (w as i32 / 2, 20),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let n = returns.len();
    let mut i = 0;
    while i < n {
        let regime = regimes[i];
        let mut j = i;
        while j + 1 < n && regimes[j + 1] == regime {
            j += 1;
        }

        let x0 = i as f64 / n as f64 * w as f64;
        let x1 = (j + 1) as f64 / n as f64 * w as f64;
        let color = if regime == 1 {
            cfg.theme.profit().mix(0.15).filled()
        } else {
            cfg.theme.loss().mix(0.15).filled()
        };

        root.draw(&Rectangle::new(
            [(x0 as i32, top_pad as i32), (x1 as i32, (top_pad as f64 + chart_h) as i32)],
            color,
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        i = j + 1;
    }

    // Cumulative return line
    let pts: Vec<(f64, f64)> = cum
        .iter()
        .enumerate()
        .map(|(idx, &v)| (idx as f64, v))
        .collect();

    let chart_area = root.margin(top_pad as u32, bottom_pad as u32, 0, 0);
    let mut chart = ChartBuilder::on(&chart_area)
        .build_cartesian_2d(0f64..n as f64, y_lo - pad..y_hi + pad)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(LineSeries::new(pts, cfg.theme.accent().stroke_width(2)))
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Zero line
    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(0.0, 0.0), (n as f64, 0.0)],
            cfg.theme.border().stroke_width(1),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    let bull_count = regimes.iter().filter(|&&r| r == 1).count();
    let bear_count = n - bull_count;
    root.draw(&Text::new(
        format!("Bull: {} bars  |  Bear: {} bars", bull_count, bear_count),
        (10, 45),
        (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &MarkovRegimeConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &MarkovRegimeConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = MarkovRegimeConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_markov.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
