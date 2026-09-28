// crates/bt-viz/src/aroon.rs
// Author: Sourish Dey

//! Aroon indicator chart. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct AroonConfig {
    pub title: String,
    pub theme: Theme,
    pub period: usize,
}

impl Default for AroonConfig {
    fn default() -> Self {
        Self { title: "Aroon".to_string(), theme: Theme::Dark, period: 25 }
    }
}

impl AroonConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn period(mut self, p: usize) -> Self { self.period = p.max(2); self }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &AroonConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let n = series.candles.len();
    let mut aroon_up = vec![f64::NAN; n];
    let mut aroon_down = vec![f64::NAN; n];

    for i in (cfg.period - 1)..n {
        let window = &series.candles[i + 1 - cfg.period..=i];
        let high_idx = window.iter().enumerate().max_by(|(_, a), (_, b)| a.high.partial_cmp(&b.high).unwrap()).map(|(i, _)| i).unwrap_or(0);
        let low_idx = window.iter().enumerate().min_by(|(_, a), (_, b)| a.low.partial_cmp(&b.low).unwrap()).map(|(i, _)| i).unwrap_or(0);
        aroon_up[i] = 100.0 * (cfg.period - high_idx) as f64 / cfg.period as f64;
        aroon_down[i] = 100.0 * (cfg.period - low_idx) as f64 / cfg.period as f64;
    }

    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;

    let mut chart = ChartBuilder::on(&root)
        .caption(&cfg.title, (TITLE_FONT, 22).into_font().color(&cfg.theme.text()))
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(50)
        .build_cartesian_2d(t_min..t_max, 0.0..100.0)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart.configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart.draw_series(LineSeries::new(
        series.candles.iter().enumerate().filter_map(|(i, c)| if !aroon_up[i].is_nan() { Some((c.t, aroon_up[i])) } else { None }),
        cfg.theme.profit().stroke_width(2),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    chart.draw_series(LineSeries::new(
        series.candles.iter().enumerate().filter_map(|(i, c)| if !aroon_down[i].is_nan() { Some((c.t, aroon_down[i])) } else { None }),
        cfg.theme.loss().stroke_width(2),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &AroonConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &AroonConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = AroonConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_aroon.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}