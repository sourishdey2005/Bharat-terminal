//! Tier 1 #4 â€” Tick delta / cumulative delta (order-flow bars). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CumulativeDeltaConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for CumulativeDeltaConfig {
    fn default() -> Self {
        Self {
            title: "Cumulative Delta".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl CumulativeDeltaConfig {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

/// Approximates per-bar aggressor delta from OHLCV: buy volume estimated by
/// how much of the bar's range closed above the midpoint (a common proxy
/// when true bid/ask tick data isn't available), then accumulates it.
pub fn compute_deltas(series: &OhlcvSeries) -> (Vec<f64>, Vec<f64>) {
    let mut per_bar = Vec::with_capacity(series.candles.len());
    let mut cumulative = Vec::with_capacity(series.candles.len());
    let mut running = 0.0;
    for c in &series.candles {
        let range = (c.high - c.low).max(1e-9);
        let close_pos = (c.close - c.low) / range; // 0..1
        let buy_fraction = close_pos;
        let delta = c.volume * (2.0 * buy_fraction - 1.0);
        running += delta;
        per_bar.push(delta);
        cumulative.push(running);
    }
    (per_bar, cumulative)
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &CumulativeDeltaConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let (per_bar, cumulative) = compute_deltas(series);
    let n = series.candles.len() as f64;

    let (bar_area, cum_area) = root.split_vertically((45).percent());

    let max_abs_bar = per_bar.iter().cloned().fold(0.0_f64, |a, b| a.max(b.abs()));
    let mut bar_chart = ChartBuilder::on(&bar_area)
        .caption(
            &cfg.title,
            (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(20)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..n, -max_abs_bar * 1.1..max_abs_bar * 1.1)
        .map_err(|e| BtError::Render(e.to_string()))?;

    bar_chart
        .configure_mesh()
        .label_style((LABEL_FONT, 11).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .y_desc("Delta / bar")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    bar_chart
        .draw_series(per_bar.iter().enumerate().map(|(i, &d)| {
            let color = if d >= 0.0 {
                cfg.theme.profit()
            } else {
                cfg.theme.loss()
            };
            Rectangle::new([(i as f64, 0.0), (i as f64 + 0.8, d)], color.filled())
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    let cum_min = cumulative.iter().cloned().fold(f64::MAX, f64::min);
    let cum_max = cumulative.iter().cloned().fold(f64::MIN, f64::max);
    let pad = (cum_max - cum_min).max(1.0) * 0.1;

    let mut cum_chart = ChartBuilder::on(&cum_area)
        .margin(10)
        .x_label_area_size(25)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..n, (cum_min - pad)..(cum_max + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    cum_chart
        .configure_mesh()
        .label_style((LABEL_FONT, 11).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .y_desc("Cumulative Delta")
        .x_desc("Bar")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    cum_chart
        .draw_series(LineSeries::new(
            cumulative.iter().enumerate().map(|(i, &v)| (i as f64, v)),
            cfg.theme.accent().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &CumulativeDeltaConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &CumulativeDeltaConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_constructs() {
        let cfg = CumulativeDeltaConfig::new();
        let _ = cfg;
    }
}
