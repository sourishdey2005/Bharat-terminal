// crates/bt-viz/src/3d_risk_landscape.rs
// Author: Sourish Dey

//! 3D risk landscape — VaR surface across confidence × horizons.
//! Made by Sourish Dey.

use bt_analytics::var_historical;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct RiskLandscape3DConfig {
    pub title: String,
    pub theme: Theme,
    pub max_horizon: usize,
}

impl Default for RiskLandscape3DConfig {
    fn default() -> Self {
        Self {
            title: "3D Risk Landscape".to_string(),
            theme: Theme::Dark,
            max_horizon: 20,
        }
    }
}

impl RiskLandscape3DConfig {
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
    pub fn max_horizon(mut self, h: usize) -> Self {
        self.max_horizon = h.max(5);
        self
    }
}

fn project(x: f64, y: f64, z: f64, cx: f64, cy: f64) -> (i32, i32) {
    let focal = 500.0;
    let s = focal / (focal + z);
    ((cx + x * s) as i32, (cy - y * s) as i32)
}

fn shade(c: RGBAColor, f: f64) -> RGBColor {
    let f = f.clamp(0.15, 1.2);
    RGBColor(
        (c.0 as f64 * f).min(255.0) as u8,
        (c.1 as f64 * f).min(255.0) as u8,
        (c.2 as f64 * f).min(255.0) as u8,
    )
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &RiskLandscape3DConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let (w, h) = root.dim_in_pixel();
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0 + 40.0;

    root.draw(&Text::new(
        format!("{} — {}", cfg.title, series.symbol),
        (w as i32 / 2 - 130, 14),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let returns = series.returns();
    if returns.len() < 20 {
        return Err(BtError::InvalidInput(
            "series too short for VaR surface".into(),
        ));
    }

    let n_conf = 10;
    let n_h = cfg.max_horizon.min(20);
    let x_span = 360.0;
    let z_span = 240.0;
    let y_span = 160.0;

    let mut grid: Vec<Vec<(i32, i32)>> = Vec::with_capacity(n_h);
    let mut vals: Vec<Vec<f64>> = Vec::with_capacity(n_h);

    for j in 0..n_h {
        let horizon = j + 1;
        let scale = (horizon as f64).sqrt();
        let mut row = Vec::with_capacity(n_conf);
        let mut vrow = Vec::with_capacity(n_conf);
        for i in 0..n_conf {
            let conf = 0.90 + (i as f64 / (n_conf - 1) as f64) * 0.095;
            let daily_var = var_historical(&returns, conf).abs();
            let var = daily_var * scale;
            let x = (i as f64 / (n_conf - 1) as f64 - 0.5) * x_span;
            let z = (j as f64 / (n_h - 1) as f64 - 0.5) * z_span;
            let py = var * y_span * 40.0;
            row.push(project(x, py, z, cx, cy));
            vrow.push(var);
        }
        grid.push(row);
        vals.push(vrow);
    }

    let max_var = vals
        .iter()
        .flat_map(|r| r.iter().copied())
        .fold(0.0_f64, f64::max)
        .max(1e-9);

    for j in (0..n_h.saturating_sub(1)).rev() {
        for i in (0..n_conf.saturating_sub(1)).rev() {
            let v_avg = (vals[j][i] + vals[j][i + 1] + vals[j + 1][i] + vals[j + 1][i + 1]) / 4.0;
            let t = (v_avg / max_var).clamp(0.0, 1.0);
            let color = if t > 0.6 {
                cfg.theme.loss().mix(0.75)
            } else if t > 0.3 {
                cfg.theme.accent().mix(0.7)
            } else {
                cfg.theme.profit().mix(0.7)
            };
            let depth_f = 0.4 + 0.6 * (j as f64 / n_h as f64);
            let pts = vec![
                grid[j][i],
                grid[j][i + 1],
                grid[j + 1][i + 1],
                grid[j + 1][i],
            ];
            root.draw(&Polygon::new(pts, shade(color, depth_f).filled()))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    for j in 0..n_h {
        let pts: Vec<(i32, i32)> = (0..n_conf).map(|i| grid[j][i]).collect();
        root.draw(&PathElement::new(pts, cfg.theme.border().stroke_width(1)))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }
    for i in 0..n_conf {
        let pts: Vec<(i32, i32)> = (0..n_h).map(|j| grid[j][i]).collect();
        root.draw(&PathElement::new(pts, cfg.theme.border().stroke_width(1)))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    root.draw(&Text::new(
        "X: confidence  Y: VaR  Z: horizon (days)",
        (w as i32 / 2 - 120, h as i32 - 40),
        (LABEL_FONT, 11)
            .into_font()
            .color(&cfg.theme.text().mix(0.6)),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &RiskLandscape3DConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &RiskLandscape3DConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = RiskLandscape3DConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_3d_risk_landscape.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
