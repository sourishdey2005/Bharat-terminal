// crates/bt-viz/src/candlestick_bollinger.rs
// Author: Sourish Dey

//! Tier 1 #3 â€” Candlestick + Bollinger Bands (3 bands shaded).
//! Made by Sourish Dey.

use bt_analytics::bollinger;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CandlestickBollingerConfig {
    pub title: String,
    pub theme: Theme,
    pub show_volume: bool,
    pub period: usize,
    pub std_dev: f64,
}

impl Default for CandlestickBollingerConfig {
    fn default() -> Self {
        Self {
            title: "Candlestick + Bollinger".to_string(),
            theme: Theme::Dark,
            show_volume: true,
            period: 20,
            std_dev: 2.0,
        }
    }
}

impl CandlestickBollingerConfig {
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

    pub fn show_volume(mut self, show: bool) -> Self {
        self.show_volume = show;
        self
    }

    pub fn period(mut self, period: usize) -> Self {
        self.period = period.max(2);
        self
    }

    pub fn std_dev(mut self, k: f64) -> Self {
        self.std_dev = k.max(0.5);
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &CandlestickBollingerConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let (chart_area, volume_area) = if cfg.show_volume {
        let split = root.split_vertically((70).percent());
        (split.0, Some(split.1))
    } else {
        (root.clone(), None)
    };

    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;
    let low = series.candles.iter().map(|c| c.low).fold(f64::MAX, f64::min);
    let high = series.candles.iter().map(|c| c.high).fold(f64::MIN, f64::max);
    let pad = (high - low) * 0.05;

    let (middle, upper, lower) = bollinger(series, cfg.period, cfg.std_dev);

    // Extend y-range to include bands
    let band_high = upper.iter().filter(|v| !v.is_nan()).fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let band_low = lower.iter().filter(|v| !v.is_nan()).fold(f64::INFINITY, |a, &b| a.min(b));
    let y_min = low.min(band_low) - pad;
    let y_max = high.max(band_high) + pad;

    let mut chart = ChartBuilder::on(&chart_area)
        .caption(
            format!("{} â€” {}", cfg.title, series.symbol),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, y_min..y_max)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Shaded Bollinger Bands area
    chart
        .draw_series(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !upper[i].is_nan() && !lower[i].is_nan() {
                    Some(Polygon::new(
                        vec![
                            (c.t, upper[i]),
                            (c.t, lower[i]),
                        ],
                        cfg.theme.info().mix(0.15).filled(),
                    ))
                } else {
                    None
                }
            }),
        )
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Upper band line
    chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !upper[i].is_nan() { Some((c.t, upper[i])) } else { None }
            }),
            cfg.theme.info().mix(0.7).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label(format!("Upper ({})", cfg.std_dev))
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.info().stroke_width(1)));

    // Middle band (SMA)
    chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !middle[i].is_nan() { Some((c.t, middle[i])) } else { None }
            }),
            cfg.theme.accent().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label(format!("SMA{}", cfg.period))
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.accent().stroke_width(2)));

    // Lower band line
    chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !lower[i].is_nan() { Some((c.t, lower[i])) } else { None }
            }),
            cfg.theme.info().mix(0.7).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label(format!("Lower ({})", cfg.std_dev))
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.info().stroke_width(1)));

    let candle_width = ((t_max - t_min) / series.candles.len() as f64).max(0.3) * 0.4;

    chart
        .draw_series(series.candles.iter().map(|c| {
            let color = if c.is_bullish() {
                cfg.theme.profit()
            } else {
                cfg.theme.loss()
            };
            CandleStick::new(
                c.t, c.open, c.high, c.low, c.close,
                color.filled(), color.filled(), (candle_width * 10.0) as u32,
            )
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_series_labels()
        .background_style(cfg.theme.background().mix(0.9))
        .border_style(cfg.theme.border())
        .label_font((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    if let Some(vol_area) = volume_area {
        let max_vol = series.candles.iter().map(|c| c.volume).fold(0.0_f64, f64::max);
        let mut vol_chart = ChartBuilder::on(&vol_area)
            .margin(10)
            .x_label_area_size(20)
            .y_label_area_size(60)
            .build_cartesian_2d(t_min..t_max, 0.0..(max_vol * 1.1))
            .map_err(|e| BtError::Render(e.to_string()))?;

        vol_chart
            .configure_mesh()
            .label_style((LABEL_FONT, 10).into_font().color(&cfg.theme.text()))
            .axis_style(&cfg.theme.border())
            .disable_x_mesh()
            .draw()
            .map_err(|e| BtError::Render(e.to_string()))?;

        vol_chart
            .draw_series(series.candles.iter().map(|c| {
                let color = if c.is_bullish() {
                    cfg.theme.profit()
                } else {
                    cfg.theme.loss()
                };
                Rectangle::new(
                    [(c.t - candle_width, 0.0), (c.t + candle_width, c.volume)],
                    color.mix(0.6).filled(),
                )
            }))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &CandlestickBollingerConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &CandlestickBollingerConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_constructs() {
        let cfg = CandlestickBollingerConfig::new();
        let _ = cfg;
    }
}
