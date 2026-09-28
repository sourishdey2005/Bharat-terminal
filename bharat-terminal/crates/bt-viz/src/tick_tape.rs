// crates/bt-viz/src/tick_tape.rs
// Author: Sourish Dey

//! Tick tape heatmap. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct TickTapeConfig {
    pub title: String,
    pub theme: Theme,
    pub rows: usize,
}

impl Default for TickTapeConfig {
    fn default() -> Self {
        Self {
            title: "Tick Tape".to_string(),
            theme: Theme::Dark,
            rows: 20,
        }
    }
}

impl TickTapeConfig {
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

    pub fn rows(mut self, r: usize) -> Self {
        self.rows = r.max(5);
        self
    }
}

#[derive(Debug, Clone)]
struct Tick {
    t: f64,
    price: f64,
    volume: f64,
    is_buy: bool,
}

fn generate_ticks(series: &OhlcvSeries, rows: usize) -> Vec<Tick> {
    let mut ticks = Vec::new();
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
    let range = (high - low).max(1e-9);

    for candle in &series.candles {
        let num_ticks = (candle.volume / 100.0).max(1.0) as usize;
        for j in 0..num_ticks {
            let frac = j as f64 / num_ticks as f64;
            let price = candle.low + frac * (candle.high - candle.low);
            let is_buy = price >= (candle.open + candle.close) / 2.0;
            let vol = candle.volume / num_ticks as f64;
            ticks.push(Tick {
                t: candle.t,
                price,
                volume: vol,
                is_buy,
            });
        }
    }

    let _ = range;
    let _ = rows;
    ticks
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &TickTapeConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let ticks = generate_ticks(series, cfg.rows);
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
    let pad = (high - low).max(1.0) * 0.05;

    let max_vol = ticks
        .iter()
        .map(|t| t.volume)
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
    let cell_h = ((high + pad) - (low - pad)) / cfg.rows as f64;

    for tick in &ticks {
        let row_idx = ((tick.price - (low - pad)) / cell_h).floor() as usize;
        let row_idx = row_idx.min(cfg.rows - 1);
        let y = low - pad + row_idx as f64 * cell_h;
        let intensity = (tick.volume / max_vol).min(1.0);
        let color = if tick.is_buy {
            cfg.theme.profit().mix(intensity * 0.8)
        } else {
            cfg.theme.loss().mix(intensity * 0.8)
        };

        chart
            .draw_series(std::iter::once(Rectangle::new(
                [(tick.t, y), (tick.t + cell_w, y + cell_h)],
                color.filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &TickTapeConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &TickTapeConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = TickTapeConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_tick_tape.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
