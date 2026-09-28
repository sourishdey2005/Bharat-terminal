// crates/bt-viz/src/candlestick_ichimoku.rs
// Author: Sourish Dey

//! Candlestick + Ichimoku cloud. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CandlestickIchimokuConfig {
    pub title: String,
    pub theme: Theme,
    pub tenkan: usize,
    pub kijun: usize,
    pub senkou_b: usize,
}

impl Default for CandlestickIchimokuConfig {
    fn default() -> Self {
        Self {
            title: "Candlestick + Ichimoku".to_string(),
            theme: Theme::Dark,
            tenkan: 9,
            kijun: 26,
            senkou_b: 52,
        }
    }
}

impl CandlestickIchimokuConfig {
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

    pub fn tenkan(mut self, v: usize) -> Self {
        self.tenkan = v.max(2);
        self
    }

    pub fn kijun(mut self, v: usize) -> Self {
        self.kijun = v.max(2);
        self
    }

    pub fn senkou_b(mut self, v: usize) -> Self {
        self.senkou_b = v.max(2);
        self
    }
}

fn ichimoku(series: &OhlcvSeries, tenkan: usize, kijun: usize, senkou_b_period: usize) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let n = series.candles.len();
    let mut tenkan_sen = vec![f64::NAN; n];
    let mut kijun_sen = vec![f64::NAN; n];
    let mut senkou_a = vec![f64::NAN; n];
    let mut senkou_b = vec![f64::NAN; n];
    let mut chikou = vec![f64::NAN; n];

    for i in 0..n {
        if i >= tenkan - 1 {
            let window = &series.candles[i + 1 - tenkan..=i];
            let hh = window.iter().map(|c| c.high).fold(f64::NEG_INFINITY, f64::max);
            let ll = window.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
            tenkan_sen[i] = (hh + ll) / 2.0;
        }
        if i >= kijun - 1 {
            let window = &series.candles[i + 1 - kijun..=i];
            let hh = window.iter().map(|c| c.high).fold(f64::NEG_INFINITY, f64::max);
            let ll = window.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
            kijun_sen[i] = (hh + ll) / 2.0;
        }
        if i >= senkou_b_period - 1 {
            let window = &series.candles[i + 1 - senkou_b_period..=i];
            let hh = window.iter().map(|c| c.high).fold(f64::NEG_INFINITY, f64::max);
            let ll = window.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
            senkou_b[i] = (hh + ll) / 2.0;
        }
        if !tenkan_sen[i].is_nan() && !kijun_sen[i].is_nan() {
            senkou_a[i] = (tenkan_sen[i] + kijun_sen[i]) / 2.0;
        }
        if i + kijun < n {
            chikou[i] = series.candles[i + kijun].close;
        }
    }

    (tenkan_sen, kijun_sen, senkou_a, senkou_b, chikou)
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &CandlestickIchimokuConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;
    let low = series.candles.iter().map(|c| c.low).fold(f64::MAX, f64::min);
    let high = series.candles.iter().map(|c| c.high).fold(f64::MIN, f64::max);
    let pad = (high - low) * 0.05;

    let (tenkan, kijun, senkou_a, senkou_b, _chikou) = ichimoku(series, cfg.tenkan, cfg.kijun, cfg.senkou_b);

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — {}", cfg.title, series.symbol),
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
                c.t, c.open, c.high, c.low, c.close,
                color.filled(), color.filled(), (candle_width * 10.0) as u32,
            )
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    for i in 0..series.candles.len().saturating_sub(1) {
        if !senkou_a[i].is_nan() && !senkou_b[i].is_nan() && !senkou_a[i + 1].is_nan() && !senkou_b[i + 1].is_nan() {
            let c1 = &series.candles[i];
            let c2 = &series.candles[i + 1];
            let fill_color = if senkou_a[i] > senkou_b[i] {
                cfg.theme.profit().mix(0.15)
            } else {
                cfg.theme.loss().mix(0.15)
            };
            chart
                .draw_series(std::iter::once(Polygon::new(
                    vec![
                        (c1.t, senkou_a[i]),
                        (c2.t, senkou_a[i + 1]),
                        (c2.t, senkou_b[i + 1]),
                        (c1.t, senkou_b[i]),
                    ],
                    fill_color.filled(),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !tenkan[i].is_nan() { Some((c.t, tenkan[i])) } else { None }
            }),
            cfg.theme.info().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Tenkan-sen")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.info().stroke_width(2)));

    chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !kijun[i].is_nan() { Some((c.t, kijun[i])) } else { None }
            }),
            cfg.theme.loss().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Kijun-sen")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.loss().stroke_width(2)));

    chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !senkou_a[i].is_nan() { Some((c.t, senkou_a[i])) } else { None }
            }),
            cfg.theme.profit().stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Senkou A")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.profit().stroke_width(1)));

    chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !senkou_b[i].is_nan() { Some((c.t, senkou_b[i])) } else { None }
            }),
            cfg.theme.accent().stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Senkou B")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.accent().stroke_width(1)));

    chart
        .configure_series_labels()
        .border_style(&cfg.theme.border())
        .background_style(cfg.theme.background().mix(0.8))
        .label_font((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &CandlestickIchimokuConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &CandlestickIchimokuConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = CandlestickIchimokuConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_candlestick_ichimoku.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
