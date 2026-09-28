// crates/bt-viz/src/vortex.rs
// Author: Sourish Dey

//! Vortex indicator chart. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct VortexConfig {
    pub title: String,
    pub theme: Theme,
    pub period: usize,
}

impl Default for VortexConfig {
    fn default() -> Self {
        Self { title: "Vortex".to_string(), theme: Theme::Dark, period: 14 }
    }
}

impl VortexConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn period(mut self, p: usize) -> Self { self.period = p.max(2); self }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &VortexConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let n = series.candles.len();
    let mut vi_plus = vec![f64::NAN; n];
    let mut vi_minus = vec![f64::NAN; n];

    for i in 1..n {
        let vm_plus = (series.candles[i].high - series.candles[i - 1].low).abs();
        let vm_minus = (series.candles[i].low - series.candles[i - 1].high).abs();

        let tr = (series.candles[i].high - series.candles[i].low)
            .max((series.candles[i].high - series.candles[i - 1].close).abs())
            .max((series.candles[i].low - series.candles[i - 1].close).abs());

        if i >= cfg.period {
            let start = i - cfg.period + 1;
            let sum_vm_plus: f64 = (start..=i).map(|j| (series.candles[j].high - series.candles[j - 1].low).abs()).sum();
            let sum_vm_minus: f64 = (start..=i).map(|j| (series.candles[j].low - series.candles[j - 1].high).abs()).sum();
            let sum_tr: f64 = (start..=i).map(|j| {
                (series.candles[j].high - series.candles[j].low)
                    .max((series.candles[j].high - series.candles[j - 1].close).abs())
                    .max((series.candles[j].low - series.candles[j - 1].close).abs())
            }).sum();

            if sum_tr > 0.0 {
                vi_plus[i] = sum_vm_plus / sum_tr;
                vi_minus[i] = sum_vm_minus / sum_tr;
            }
        }
    }

    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;

    let mut chart = ChartBuilder::on(&root)
        .caption(&cfg.title, (TITLE_FONT, 22).into_font().color(&cfg.theme.text()))
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(50)
        .build_cartesian_2d(t_min..t_max, 0.5..1.5)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart.configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart.draw_series(LineSeries::new(
        series.candles.iter().enumerate().filter_map(|(i, c)| if !vi_plus[i].is_nan() { Some((c.t, vi_plus[i])) } else { None }),
        cfg.theme.profit().stroke_width(2),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    chart.draw_series(LineSeries::new(
        series.candles.iter().enumerate().filter_map(|(i, c)| if !vi_minus[i].is_nan() { Some((c.t, vi_minus[i])) } else { None }),
        cfg.theme.loss().stroke_width(2),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &VortexConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &VortexConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = VortexConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_vortex.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}