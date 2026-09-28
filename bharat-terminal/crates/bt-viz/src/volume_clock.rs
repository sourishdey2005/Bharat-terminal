// crates/bt-viz/src/volume_clock.rs
// Author: Sourish Dey

//! Volume-sampled bars. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct VolumeClockConfig {
    pub title: String,
    pub theme: Theme,
    pub target_bars: usize,
}

impl Default for VolumeClockConfig {
    fn default() -> Self {
        Self {
            title: "Volume Clock".to_string(),
            theme: Theme::Dark,
            target_bars: 50,
        }
    }
}

impl VolumeClockConfig {
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

    pub fn target_bars(mut self, bars: usize) -> Self {
        self.target_bars = bars.max(10);
        self
    }
}

#[derive(Debug, Clone)]
struct VolumeBar {
    t: f64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

fn resample_by_volume(series: &OhlcvSeries, target_bars: usize) -> Vec<VolumeBar> {
    let total_vol: f64 = series.candles.iter().map(|c| c.volume).sum();
    let vol_per_bar = total_vol / target_bars as f64;

    let mut bars = Vec::new();
    let mut cur = VolumeBar {
        t: series.candles[0].t,
        open: series.candles[0].open,
        high: series.candles[0].high,
        low: series.candles[0].low,
        close: series.candles[0].close,
        volume: 0.0,
    };

    for candle in &series.candles {
        cur.high = cur.high.max(candle.high);
        cur.low = cur.low.min(candle.low);
        cur.close = candle.close;
        cur.volume += candle.volume;
        cur.t = candle.t;

        if cur.volume >= vol_per_bar {
            bars.push(cur.clone());
            cur = VolumeBar {
                t: candle.t,
                open: candle.open,
                high: candle.high,
                low: candle.low,
                close: candle.close,
                volume: 0.0,
            };
        }
    }

    if cur.volume > 0.0 && (bars.is_empty() || cur.t != bars.last().unwrap().t) {
        bars.push(cur);
    }

    bars
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &VolumeClockConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let bars = resample_by_volume(series, cfg.target_bars);

    if bars.is_empty() {
        let mut chart = ChartBuilder::on(&root)
            .caption(
                format!("{} — {} (No data)", cfg.title, series.symbol),
                (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
            )
            .margin(10)
            .build_cartesian_2d(0.0..1.0, 0.0..1.0)
            .map_err(|e| BtError::Render(e.to_string()))?;

        chart
            .configure_mesh()
            .disable_mesh()
            .draw()
            .map_err(|e| BtError::Render(e.to_string()))?;

        draw_footer(&root, cfg.theme)?;
        root.present().map_err(|e| BtError::Render(e.to_string()))?;
        return Ok(());
    }

    let t_min = bars.first().unwrap().t;
    let t_max = bars.last().unwrap().t;
    let low = bars.iter().map(|b| b.low).fold(f64::MAX, f64::min);
    let high = bars.iter().map(|b| b.high).fold(f64::MIN, f64::max);
    let pad = (high - low).max(1.0) * 0.05;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — {} ({} bars)", cfg.title, series.symbol, bars.len()),
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

    let candle_width = ((t_max - t_min) / bars.len() as f64).max(0.3) * 0.4;

    for bar in &bars {
        let color = if bar.close >= bar.open {
            cfg.theme.profit()
        } else {
            cfg.theme.loss()
        };

        chart
            .draw_series(std::iter::once(CandleStick::new(
                bar.t,
                bar.open,
                bar.high,
                bar.low,
                bar.close,
                color.filled(),
                color.filled(),
                (candle_width * 10.0) as u32,
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;

        chart
            .draw_series(std::iter::once(Rectangle::new(
                [
                    (bar.t - candle_width, low - pad),
                    (
                        bar.t + candle_width,
                        low - pad
                            + bar.volume / bars.iter().map(|b| b.volume).fold(0.0_f64, f64::max)
                                * pad,
                    ),
                ],
                color.mix(0.3).filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &VolumeClockConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &VolumeClockConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = VolumeClockConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_volume_clock.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
