// crates/bt-viz/src/footprint.rs
// Author: Sourish Dey

//! Footprint bid/ask chart. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct FootprintConfig {
    pub title: String,
    pub theme: Theme,
    pub num_rows: usize,
}

impl Default for FootprintConfig {
    fn default() -> Self {
        Self {
            title: "Footprint".to_string(),
            theme: Theme::Dark,
            num_rows: 20,
        }
    }
}

impl FootprintConfig {
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

    pub fn num_rows(mut self, rows: usize) -> Self {
        self.num_rows = rows.max(5);
        self
    }
}

#[derive(Debug, Clone)]
struct FootprintRow {
    price: f64,
    bid: f64,
    ask: f64,
}

fn compute_footprint(
    series: &OhlcvSeries,
    num_rows: usize,
) -> Vec<(f64, Vec<FootprintRow>)> {
    let mut result = Vec::new();

    for candle in &series.candles {
        let range = (candle.high - candle.low).max(1e-9);
        let step = range / num_rows as f64;
        let mut rows = Vec::with_capacity(num_rows);

        for i in 0..num_rows {
            let price = candle.low + (i as f64 + 0.5) * step;
            let dist_from_close = (price - candle.close).abs() / range;
            let intensity = 1.0 - dist_from_close;
            let is_buy = candle.close >= candle.open;
            let bias = if is_buy { 0.6 } else { 0.4 };
            let bid = candle.volume * intensity * bias;
            let ask = candle.volume * intensity * (1.0 - bias);
            rows.push(FootprintRow { price, bid, ask });
        }

        result.push((candle.t, rows));
    }

    result
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &FootprintConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;
    let footprint = compute_footprint(series, cfg.num_rows);

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
    let pad = (high - low).max(1.0) * 0.05;

    let max_delta = footprint
        .iter()
        .flat_map(|(_, rows)| rows.iter().map(|r| r.bid.max(r.ask)))
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

    let bar_width = ((t_max - t_min) / series.candles.len() as f64).max(0.3) * 0.6;

    for (t, rows) in &footprint {
        for row in rows {
            let total = row.bid + row.ask;
            if total <= 0.0 {
                continue;
            }
            let bid_frac = row.bid / total;

            let bid_rect = Rectangle::new(
                [
                    (*t - bar_width, row.price - (high - low) / cfg.num_rows as f64 * 0.4),
                    (*t, row.price + (high - low) / cfg.num_rows as f64 * 0.4),
                ],
                cfg.theme.profit().mix(bid_frac * 0.7).filled(),
            );

            let ask_rect = Rectangle::new(
                [
                    (*t, row.price - (high - low) / cfg.num_rows as f64 * 0.4),
                    (*t + bar_width, row.price + (high - low) / cfg.num_rows as f64 * 0.4),
                ],
                cfg.theme.loss().mix((1.0 - bid_frac) * 0.7).filled(),
            );

            chart
                .draw_series(std::iter::once(bid_rect))
                .map_err(|e| BtError::Render(e.to_string()))?;
            chart
                .draw_series(std::iter::once(ask_rect))
                .map_err(|e| BtError::Render(e.to_string()))?;

            if total > max_delta * 0.3 {
                chart
                    .draw_series(std::iter::once(Text::new(
                        format!("{:.0}/{:.0}", row.bid, row.ask),
                        (*t - bar_width * 0.5, row.price),
                        (LABEL_FONT, 8).into_font().color(&cfg.theme.text()),
                    )))
                    .map_err(|e| BtError::Render(e.to_string()))?;
            }
        }
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &FootprintConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &FootprintConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = FootprintConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_footprint.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
