// crates/bt-viz/src/multi_timeframe.rs
// Author: Sourish Dey

//! Multi-timeframe comparison chart. Made by Sourish Dey.

use bt_analytics::sma;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct MultiTimeframeConfig {
    pub title: String,
    pub theme: Theme,
    pub timeframes: Vec<usize>,
    pub ma_period: usize,
}

impl Default for MultiTimeframeConfig {
    fn default() -> Self {
        Self {
            title: "Multi-Timeframe".to_string(),
            theme: Theme::Dark,
            timeframes: vec![5, 10, 20],
            ma_period: 10,
        }
    }
}

impl MultiTimeframeConfig {
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

    pub fn timeframes(mut self, tf: Vec<usize>) -> Self {
        self.timeframes = if tf.is_empty() { vec![10] } else { tf };
        self
    }

    pub fn ma_period(mut self, p: usize) -> Self {
        self.ma_period = p.max(2);
        self
    }
}

fn resample_series(series: &OhlcvSeries, factor: usize) -> OhlcvSeries {
    if factor <= 1 {
        return series.clone();
    }

    let mut candles = Vec::new();
    let mut i = 0;
    while i < series.candles.len() {
        let end = (i + factor).min(series.candles.len());
        let chunk = &series.candles[i..end];
        let first = &chunk[0];
        let last = &chunk[chunk.len() - 1];
        let high = chunk
            .iter()
            .map(|c| c.high)
            .fold(f64::NEG_INFINITY, f64::max);
        let low = chunk.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
        let volume: f64 = chunk.iter().map(|c| c.volume).sum();

        candles.push(bt_core::Candle::new(
            first.t, first.open, high, low, last.close, volume,
        ));

        i += factor;
    }

    OhlcvSeries::new(&series.symbol, candles)
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &MultiTimeframeConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let num_tf = cfg.timeframes.len();
    let mut resampled: Vec<OhlcvSeries> = Vec::new();
    let mut ma_values: Vec<Vec<f64>> = Vec::new();

    for &tf in &cfg.timeframes {
        let rs = resample_series(series, tf);
        let ma = sma(&rs, cfg.ma_period);
        resampled.push(rs);
        ma_values.push(ma);
    }

    let mut all_high = f64::NEG_INFINITY;
    let mut all_low = f64::INFINITY;
    for rs in &resampled {
        for c in &rs.candles {
            all_high = all_high.max(c.high);
            all_low = all_low.min(c.low);
        }
    }
    let pad = (all_high - all_low).max(1.0) * 0.05;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — {}", cfg.title, series.symbol),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(
            0.0..(series.candles.len() as f64),
            (all_low - pad)..(all_high + pad),
        )
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let colors = [
        cfg.theme.profit(),
        cfg.theme.info(),
        cfg.theme.accent(),
        cfg.theme.loss(),
    ];

    for (tf_idx, rs) in resampled.iter().enumerate() {
        let color = colors[tf_idx % colors.len()].clone();
        let offset = tf_idx as f64 * 0.1;

        chart
            .draw_series(LineSeries::new(
                rs.candles
                    .iter()
                    .enumerate()
                    .map(|(i, c)| (i as f64 * cfg.timeframes[tf_idx] as f64 + offset, c.close)),
                color.stroke_width(2),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?
            .label(format!("TF{}", cfg.timeframes[tf_idx]))
            .legend(move |(x, y)| {
                PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2))
            });

        chart
            .draw_series(LineSeries::new(
                rs.candles.iter().enumerate().filter_map(|(i, c)| {
                    let ma_idx = i;
                    if ma_idx < ma_values[tf_idx].len() && !ma_values[tf_idx][ma_idx].is_nan() {
                        Some((
                            i as f64 * cfg.timeframes[tf_idx] as f64 + offset,
                            ma_values[tf_idx][ma_idx],
                        ))
                    } else {
                        None
                    }
                }),
                color.mix(0.5).stroke_width(1),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    chart
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

pub fn render_png(series: &OhlcvSeries, cfg: &MultiTimeframeConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &MultiTimeframeConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = MultiTimeframeConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_multi_timeframe.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
