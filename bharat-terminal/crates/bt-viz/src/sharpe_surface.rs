// crates/bt-viz/src/sharpe_surface.rs
// Author: Sourish Dey

//! Sharpe ratio surface. Made by Sourish Dey.

use bt_analytics::sharpe;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct SharpeSurfaceConfig {
    pub title: String,
    pub theme: Theme,
    pub windows: Vec<usize>,
    pub risk_free: f64,
    pub periods_per_year: usize,
}

impl Default for SharpeSurfaceConfig {
    fn default() -> Self {
        Self {
            title: "Sharpe Ratio Surface".to_string(),
            theme: Theme::Dark,
            windows: vec![10, 20, 30, 60, 120],
            risk_free: 0.05,
            periods_per_year: 252,
        }
    }
}

impl SharpeSurfaceConfig {
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
    pub fn windows(mut self, w: Vec<usize>) -> Self {
        self.windows = w;
        self
    }
    pub fn risk_free(mut self, r: f64) -> Self {
        self.risk_free = r;
        self
    }
    pub fn periods_per_year(mut self, p: usize) -> Self {
        self.periods_per_year = p;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &SharpeSurfaceConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let returns = series.returns();
    if returns.is_empty() {
        return Err(BtError::EmptySeries("returns".into()));
    }

    let n = returns.len();
    let mut surface: Vec<Vec<f64>> = Vec::with_capacity(cfg.windows.len());
    for &w in &cfg.windows {
        let mut row = Vec::with_capacity(n);
        for i in 0..n {
            if i + 1 >= w {
                let window = &returns[i + 1 - w..=i];
                row.push(sharpe(window, cfg.risk_free, cfg.periods_per_year));
            } else {
                row.push(f64::NAN);
            }
        }
        surface.push(row);
    }

    let valid: Vec<f64> = surface
        .iter()
        .flat_map(|r| r.iter().cloned())
        .filter(|v| v.is_finite())
        .collect();
    if valid.is_empty() {
        return Err(BtError::EmptySeries("sharpe surface".into()));
    }
    let v_min = valid.iter().cloned().fold(f64::MAX, f64::min);
    let v_max = valid.iter().cloned().fold(f64::MIN, f64::max);

    let n_windows = cfg.windows.len();
    let mut chart = ChartBuilder::on(&root)
        .caption(
            &cfg.title,
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(15)
        .x_label_area_size(35)
        .y_label_area_size(60)
        .build_cartesian_2d(0..n, 0..n_windows)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .x_labels(5)
        .y_labels(n_windows)
        .y_label_formatter(&|idx| {
            cfg.windows
                .get(*idx)
                .map(|w| format!("{}d", w))
                .unwrap_or_default()
        })
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    for (j, row) in surface.iter().enumerate() {
        for (i, &v) in row.iter().enumerate() {
            if !v.is_finite() {
                continue;
            }
            let t = if v_max > v_min {
                (v - v_min) / (v_max - v_min)
            } else {
                0.5
            };
            let color = blend_colors(cfg.theme.loss(), cfg.theme.profit(), t);
            chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(i, n_windows - 1 - j), (i + 1, n_windows - j)],
                    color.filled(),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

fn blend_colors(a: RGBColor, b: RGBColor, t: f64) -> RGBColor {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: u8, y: u8| (x as f64 + (y as f64 - x as f64) * t).round() as u8;
    RGBColor(lerp(a.0, b.0), lerp(a.1, b.1), lerp(a.2, b.2))
}

pub fn render_png(series: &OhlcvSeries, cfg: &SharpeSurfaceConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &SharpeSurfaceConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 200, 1, 100.0);
        let cfg = SharpeSurfaceConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_sharpe_surface.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
