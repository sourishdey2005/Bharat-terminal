//! Tier 1 #1 â€” Candlestick + Volume overlay. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

/// Builder-pattern configuration for the candlestick chart.
#[derive(Debug, Clone)]
pub struct CandlestickConfig {
    pub title: String,
    pub theme: Theme,
    pub show_volume: bool,
}

impl Default for CandlestickConfig {
    fn default() -> Self {
        Self {
            title: "Candlestick".to_string(),
            theme: Theme::Dark,
            show_volume: true,
        }
    }
}

impl CandlestickConfig {
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
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &CandlestickConfig,
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
    let low = series
        .candles
        .iter()
        .map(|c| c.low)
        .fold(f64::MAX, f64::min);
    let high = series
        .candles
        .iter()
        .map(|c| c.high)
        .fold(f64::MIN, f64::max);
    let pad = (high - low) * 0.05;

    let mut chart = ChartBuilder::on(&chart_area)
        .caption(
            format!("{} â€” {}", cfg.title, series.symbol),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, (low - pad)..(high + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let candle_width = ((t_max - t_min) / series.candles.len() as f64).max(0.3) * 0.4;

    chart
        .draw_series(series.candles.iter().map(|c| {
            let color = if c.is_bullish() {
                cfg.theme.profit()
            } else {
                cfg.theme.loss()
            };
            CandleStick::new(
                c.t,
                c.open,
                c.high,
                c.low,
                c.close,
                color.filled(),
                color.filled(),
                (candle_width * 10.0) as u32,
            )
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    if let Some(vol_area) = volume_area {
        let max_vol = series
            .candles
            .iter()
            .map(|c| c.volume)
            .fold(0.0_f64, f64::max);
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

pub fn render_png(series: &OhlcvSeries, cfg: &CandlestickConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &CandlestickConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_constructs() {
        let cfg = CandlestickConfig::new();
        let _ = cfg;
    }
}
