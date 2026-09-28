// crates/bt-viz/src/price_momentum.rs
// Author: Sourish Dey

//! Price momentum heatmap. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;
use plotters::style::RGBColor;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct PriceMomentumConfig {
    pub title: String,
    pub theme: Theme,
    pub lookback: usize,
}

impl Default for PriceMomentumConfig {
    fn default() -> Self {
        Self {
            title: "Price Momentum".to_string(),
            theme: Theme::Dark,
            lookback: 10,
        }
    }
}

impl PriceMomentumConfig {
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

    pub fn lookback(mut self, lb: usize) -> Self {
        self.lookback = lb.max(2);
        self
    }
}

fn compute_momentum(series: &OhlcvSeries, lookback: usize) -> Vec<f64> {
    let n = series.candles.len();
    let mut momentum = vec![f64::NAN; n];

    for i in lookback..n {
        let prev = series.candles[i - lookback].close;
        if prev != 0.0 {
            momentum[i] = (series.candles[i].close - prev) / prev * 100.0;
        }
    }

    momentum
}

fn momentum_to_color(value: f64, theme: Theme) -> RGBAColor {
    if value.is_nan() {
        return theme.border().mix(0.3);
    }
    if value > 5.0 {
        theme.profit().mix(0.9)
    } else if value > 2.0 {
        theme.profit().mix(0.5)
    } else if value > 0.0 {
        theme.profit().mix(0.2)
    } else if value > -2.0 {
        theme.loss().mix(0.2)
    } else if value > -5.0 {
        theme.loss().mix(0.5)
    } else {
        theme.loss().mix(0.9)
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &PriceMomentumConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let momentum = compute_momentum(series, cfg.lookback);
    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;
    let n = series.candles.len();

    let mom_min = momentum
        .iter()
        .filter(|v| !v.is_nan())
        .fold(f64::INFINITY, |a, &b| a.min(b));
    let mom_max = momentum
        .iter()
        .filter(|v| !v.is_nan())
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let range = (mom_max - mom_min).max(1.0);

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — {} (Lookback: {})", cfg.title, series.symbol, cfg.lookback),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, (mom_min - range * 0.1)..(mom_max + range * 0.1))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(LineSeries::new(
            vec![(t_min, 0.0), (t_max, 0.0)],
            cfg.theme.border().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    let cell_w = (t_max - t_min) / n as f64;

    for i in 0..n {
        if momentum[i].is_nan() {
            continue;
        }
        let color = momentum_to_color(momentum[i], cfg.theme);
        let y = if momentum[i] >= 0.0 { 0.0 } else { momentum[i] };
        let h = momentum[i].abs();

        chart
            .draw_series(std::iter::once(Rectangle::new(
                [
                    (series.candles[i].t, y),
                    (series.candles[i].t + cell_w, y + h),
                ],
                color.filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;

        if n <= 40 {
            chart
                .draw_series(std::iter::once(Text::new(
                    format!("{:.1}", momentum[i]),
                    (series.candles[i].t + cell_w * 0.5, y + h * 0.5),
                    (LABEL_FONT, 8).into_font().color(&cfg.theme.text()),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &PriceMomentumConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &PriceMomentumConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = PriceMomentumConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_price_momentum.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
