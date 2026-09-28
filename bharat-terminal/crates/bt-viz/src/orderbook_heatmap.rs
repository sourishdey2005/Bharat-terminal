// crates/bt-viz/src/orderbook_heatmap.rs
// Author: Sourish Dey

//! Order book depth heatmap. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct OrderbookHeatmapConfig {
    pub title: String,
    pub theme: Theme,
    pub depth_levels: usize,
}

impl Default for OrderbookHeatmapConfig {
    fn default() -> Self {
        Self {
            title: "Order Book Heatmap".to_string(),
            theme: Theme::Dark,
            depth_levels: 20,
        }
    }
}

impl OrderbookHeatmapConfig {
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

    pub fn depth_levels(mut self, levels: usize) -> Self {
        self.depth_levels = levels.max(5);
        self
    }
}

#[derive(Debug, Clone)]
struct DepthLevel {
    price: f64,
    bid_size: f64,
    ask_size: f64,
}

fn compute_depth(series: &OhlcvSeries, depth_levels: usize) -> Vec<(f64, Vec<DepthLevel>)> {
    let mut result = Vec::new();

    for candle in &series.candles {
        let typical = (candle.high + candle.low + candle.close) / 3.0;
        let spread = (candle.high - candle.low).max(1e-9);
        let step = spread / depth_levels as f64;
        let mut levels = Vec::with_capacity(depth_levels);

        for i in 0..depth_levels {
            let price = typical - spread / 2.0 + i as f64 * step;
            let dist = (price - typical).abs() / (spread / 2.0);
            let decay = (-dist * 2.0).exp();
            let is_bid = price < typical;
            let bias = if is_bid {
                candle.close >= candle.open
            } else {
                candle.close < candle.open
            };
            let size = candle.volume * decay * if bias { 0.7 } else { 0.3 };
            levels.push(DepthLevel {
                price,
                bid_size: if is_bid { size } else { 0.0 },
                ask_size: if !is_bid { size } else { 0.0 },
            });
        }

        result.push((candle.t, levels));
    }

    result
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &OrderbookHeatmapConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;
    let depth = compute_depth(series, cfg.depth_levels);

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
    let pad = (high - low).max(1.0) * 0.1;

    let max_size = depth
        .iter()
        .flat_map(|(_, levels)| levels.iter().map(|l| l.bid_size.max(l.ask_size)))
        .fold(0.0_f64, f64::max);

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

    let cell_w = (t_max - t_min) / series.candles.len() as f64;
    let cell_h = ((high + pad) - (low - pad)) / cfg.depth_levels as f64;

    for (t, levels) in &depth {
        for (i, level) in levels.iter().enumerate() {
            let y = low - pad + i as f64 * cell_h;

            if level.bid_size > 0.0 {
                let intensity = (level.bid_size / max_size).min(1.0);
                chart
                    .draw_series(std::iter::once(Rectangle::new(
                        [(*t, y), (*t + cell_w, y + cell_h)],
                        cfg.theme.profit().mix(intensity * 0.8).filled(),
                    )))
                    .map_err(|e| BtError::Render(e.to_string()))?;
            }

            if level.ask_size > 0.0 {
                let intensity = (level.ask_size / max_size).min(1.0);
                chart
                    .draw_series(std::iter::once(Rectangle::new(
                        [(*t, y), (*t + cell_w, y + cell_h)],
                        cfg.theme.loss().mix(intensity * 0.8).filled(),
                    )))
                    .map_err(|e| BtError::Render(e.to_string()))?;
            }
        }
    }

    let last_candle = series.candles.last().unwrap();
    chart
        .draw_series(std::iter::once(Circle::new(
            (last_candle.t, last_candle.close),
            5,
            cfg.theme.accent().filled(),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &OrderbookHeatmapConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &OrderbookHeatmapConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = OrderbookHeatmapConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_orderbook_heatmap.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
