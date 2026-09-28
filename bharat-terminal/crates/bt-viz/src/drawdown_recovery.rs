// crates/bt-viz/src/drawdown_recovery.rs
// Author: Sourish Dey

//! Drawdown recovery analysis. Made by Sourish Dey.

use bt_analytics::drawdown_series;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct DrawdownRecoveryConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for DrawdownRecoveryConfig {
    fn default() -> Self {
        Self {
            title: "Drawdown Recovery Analysis".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl DrawdownRecoveryConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &DrawdownRecoveryConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let dd = drawdown_series(series);
    if dd.is_empty() {
        return Err(BtError::EmptySeries("drawdown series".into()));
    }

    let min_dd = dd.iter().cloned().fold(0.0_f64, f64::min);
    let n = dd.len() as f64;

    let mut chart = ChartBuilder::on(&root)
        .caption(&cfg.title, (TITLE_FONT, 22).into_font().color(&cfg.theme.text()))
        .margin(15)
        .x_label_area_size(35)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..n, (min_dd * 1.1).min(-1.0)..0.5f64)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .x_desc("Bar")
        .y_desc("Drawdown (%)")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(AreaSeries::new(
            dd.iter().enumerate().map(|(i, &v)| (i as f64, v)),
            0.0,
            cfg.theme.loss().mix(0.35),
        ).border_style(cfg.theme.loss().stroke_width(2)))
        .map_err(|e| BtError::Render(e.to_string()))?;

    let mut in_dd = false;
    let mut dd_start = 0usize;
    let mut max_dd = 0.0_f64;
    let mut max_dd_idx = 0usize;
    let mut recovery_points = Vec::new();

    for (i, &v) in dd.iter().enumerate() {
        if v < -0.01 && !in_dd {
            in_dd = true;
            dd_start = i;
            max_dd = v;
            max_dd_idx = i;
        } else if in_dd {
            if v < max_dd {
                max_dd = v;
                max_dd_idx = i;
            }
            if v >= -0.01 {
                recovery_points.push((dd_start, max_dd_idx, i, max_dd));
                in_dd = false;
            }
        }
    }
    if in_dd {
        recovery_points.push((dd_start, max_dd_idx, dd.len() - 1, max_dd));
    }

    for (start, trough, recovery, depth) in &recovery_points {
        chart
            .draw_series(std::iter::once(Circle::new(
                (*trough as f64, *depth),
                5,
                cfg.theme.loss().filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;

        chart
            .draw_series(std::iter::once(Circle::new(
                (*recovery as f64, 0.0),
                5,
                cfg.theme.profit().filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;

        chart
            .draw_series(std::iter::once(Text::new(
                format!("{:.1}%", depth),
                (*trough as f64, *depth - 1.0),
                (LABEL_FONT, 11).into_font().color(&cfg.theme.loss()),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &DrawdownRecoveryConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &DrawdownRecoveryConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = DrawdownRecoveryConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_drawdown_recovery.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
