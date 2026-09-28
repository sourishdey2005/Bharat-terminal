// crates/bt-viz/src/volume_profile_advanced.rs
// Author: Sourish Dey

//! Advanced volume profile with value area, POC, volume delta, and
//! session-based volume-at-price heatmap. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct VolumeProfileAdvancedConfig {
    pub title: String,
    pub theme: Theme,
    pub num_bins: usize,
    pub value_area_pct: f64,
    pub show_delta: bool,
}

impl Default for VolumeProfileAdvancedConfig {
    fn default() -> Self {
        Self {
            title: "Advanced Volume Profile".to_string(),
            theme: Theme::Dark,
            num_bins: 60,
            value_area_pct: 0.7,
            show_delta: true,
        }
    }
}

impl VolumeProfileAdvancedConfig {
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

    pub fn num_bins(mut self, bins: usize) -> Self {
        self.num_bins = bins.max(10);
        self
    }

    pub fn value_area_pct(mut self, pct: f64) -> Self {
        self.value_area_pct = pct.clamp(0.1, 0.99);
        self
    }

    pub fn show_delta(mut self, show: bool) -> Self {
        self.show_delta = show;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &VolumeProfileAdvancedConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;

    let high = series
        .candles
        .iter()
        .map(|c| c.high)
        .fold(f64::NEG_INFINITY, f64::max);
    let low = series
        .candles
        .iter()
        .map(|c| c.low)
        .fold(f64::INFINITY, f64::min);

    let bin_size = (high - low) / cfg.num_bins as f64;
    let mut volume_at_price = vec![0.0; cfg.num_bins];
    let mut delta_at_price = vec![0.0; cfg.num_bins];

    for candle in &series.candles {
        let candle_low_bin = ((candle.low - low) / bin_size).floor() as isize;
        let candle_high_bin = ((candle.high - low) / bin_size).floor() as isize;
        let bins_covered = (candle_high_bin - candle_low_bin).max(1) as usize;

        for bin_idx in candle_low_bin..=candle_high_bin {
            let idx = bin_idx.clamp(0, cfg.num_bins as isize - 1) as usize;
            volume_at_price[idx] += candle.volume / bins_covered as f64;
            if candle.is_bullish() {
                delta_at_price[idx] += candle.volume / bins_covered as f64;
            } else {
                delta_at_price[idx] -= candle.volume / bins_covered as f64;
            }
        }
    }

    let max_vol = volume_at_price.iter().fold(0.0_f64, |a, &b| a.max(b));
    let total_vol: f64 = volume_at_price.iter().sum();

    let poc_idx = volume_at_price
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0);

    let mut va_low = poc_idx;
    let mut va_high = poc_idx;
    let mut va_vol = volume_at_price[poc_idx];

    while va_vol / total_vol < cfg.value_area_pct {
        let vol_low = if va_low > 0 {
            volume_at_price[va_low - 1]
        } else {
            0.0
        };
        let vol_high = if va_high + 1 < cfg.num_bins {
            volume_at_price[va_high + 1]
        } else {
            0.0
        };

        if vol_low >= vol_high && va_low > 0 {
            va_low -= 1;
            va_vol += vol_low;
        } else if va_high + 1 < cfg.num_bins {
            va_high += 1;
            va_vol += vol_high;
        } else {
            break;
        }
    }

    let va_price_low = low + va_low as f64 * bin_size;
    let va_price_high = low + (va_high + 1) as f64 * bin_size;
    let poc_price = low + (poc_idx as f64 + 0.5) * bin_size;

    let (profile_area, price_area) = root.split_horizontally((30).percent());

    let mut profile_chart = ChartBuilder::on(&profile_area)
        .caption(
            "Volume Profile",
            (TITLE_FONT, 16).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .y_label_area_size(60)
        .x_label_area_size(40)
        .build_cartesian_2d(0.0..(max_vol * 1.2), (low - bin_size)..(high + bin_size))
        .map_err(|e| BtError::Render(e.to_string()))?;

    profile_chart
        .configure_mesh()
        .label_style((LABEL_FONT, 10).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    for (i, &vol) in volume_at_price.iter().enumerate() {
        if vol > 0.0 {
            let price = low + i as f64 * bin_size;
            let color = if i >= va_low && i <= va_high {
                cfg.theme.accent().mix(1.0)
            } else if i == poc_idx {
                cfg.theme.info().mix(1.0)
            } else {
                cfg.theme.border().mix(0.5)
            };

            profile_chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(0.0, price), (vol, price + bin_size)],
                    color.mix(0.6).filled(),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    profile_chart
        .draw_series(LineSeries::new(
            vec![(0.0, poc_price), (max_vol * 1.2, poc_price)],
            cfg.theme.info().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    profile_chart
        .draw_series(LineSeries::new(
            vec![(0.0, va_price_low), (max_vol * 1.2, va_price_low)],
            cfg.theme.accent().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    profile_chart
        .draw_series(LineSeries::new(
            vec![(0.0, va_price_high), (max_vol * 1.2, va_price_high)],
            cfg.theme.accent().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    if cfg.show_delta {
        let max_delta = delta_at_price
            .iter()
            .map(|v| v.abs())
            .fold(0.0_f64, |a, b| a.max(b));
        if max_delta > 0.0 {
            for (i, &delta) in delta_at_price.iter().enumerate() {
                if delta.abs() > 0.0 {
                    let price = low + i as f64 * bin_size;
                    let bar_width = delta.abs() / max_delta * max_vol * 0.3;
                    let x_start = if delta > 0.0 {
                        max_vol * 1.1
                    } else {
                        max_vol * 1.1 - bar_width
                    };
                    let color = if delta > 0.0 {
                        cfg.theme.profit().mix(0.4)
                    } else {
                        cfg.theme.loss().mix(0.4)
                    };
                    profile_chart
                        .draw_series(std::iter::once(Rectangle::new(
                            [(x_start, price), (x_start + bar_width, price + bin_size)],
                            color.filled(),
                        )))
                        .map_err(|e| BtError::Render(e.to_string()))?;
                }
            }
        }
    }

    let pad = (high - low) * 0.05;
    let mut price_chart = ChartBuilder::on(&price_area)
        .caption(
            format!("{} — {}", cfg.title, series.symbol),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, (low - pad)..(high + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    price_chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let candle_width = ((t_max - t_min) / series.candles.len() as f64).max(0.3) * 0.4;

    price_chart
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

    price_chart
        .draw_series(LineSeries::new(
            vec![(t_min, poc_price), (t_max, poc_price)],
            cfg.theme.info().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("POC")
        .legend(|(x, y)| {
            PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.info().stroke_width(2))
        });

    price_chart
        .draw_series(LineSeries::new(
            vec![(t_min, va_price_low), (t_max, va_price_low)],
            cfg.theme.accent().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("VA Low")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                cfg.theme.accent().mix(0.5).stroke_width(1),
            )
        });

    price_chart
        .draw_series(LineSeries::new(
            vec![(t_min, va_price_high), (t_max, va_price_high)],
            cfg.theme.accent().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("VA High")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                cfg.theme.accent().mix(0.5).stroke_width(1),
            )
        });

    price_chart
        .configure_series_labels()
        .background_style(cfg.theme.background().mix(0.9))
        .border_style(cfg.theme.border())
        .label_font((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(
    series: &OhlcvSeries,
    cfg: &VolumeProfileAdvancedConfig,
    path: &str,
) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(
    series: &OhlcvSeries,
    cfg: &VolumeProfileAdvancedConfig,
    path: &str,
) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = VolumeProfileAdvancedConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_volume_profile_advanced.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
