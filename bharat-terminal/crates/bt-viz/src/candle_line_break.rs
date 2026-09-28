// crates/bt-viz/src/candle_line_break.rs
// Author: Sourish Dey

//! Three-line-break with candle bodies. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CandleLineBreakConfig {
    pub title: String,
    pub theme: Theme,
    pub lines: usize,
}

impl Default for CandleLineBreakConfig {
    fn default() -> Self {
        Self {
            title: "Line Break Candles".to_string(),
            theme: Theme::Dark,
            lines: 3,
        }
    }
}

impl CandleLineBreakConfig {
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

    pub fn lines(mut self, n: usize) -> Self {
        self.lines = n.max(2);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum LbDir {
    Up,
    Down,
}

struct LbBar {
    x: f64,
    open: f64,
    close: f64,
    dir: LbDir,
}

fn compute_line_break(series: &OhlcvSeries, lines: usize) -> Vec<LbBar> {
    if series.candles.is_empty() {
        return Vec::new();
    }

    let mut bars = Vec::new();
    let mut dir = LbDir::Up;
    let mut last_high = series.candles[0].high;
    let mut last_low = series.candles[0].low;
    let mut last_close = series.candles[0].close;

    bars.push(LbBar {
        x: series.candles[0].t,
        open: series.candles[0].open,
        close: series.candles[0].close,
        dir: LbDir::Up,
    });

    for i in 1..series.candles.len() {
        let c = &series.candles[i];
        let mut confirmed = false;

        match dir {
            LbDir::Up => {
                if c.high >= last_close {
                    bars.push(LbBar {
                        x: c.t,
                        open: last_close,
                        close: c.high,
                        dir: LbDir::Up,
                    });
                    last_high = c.high;
                    last_close = c.high;
                    confirmed = true;
                }
            }
            LbDir::Down => {
                if c.low <= last_close {
                    bars.push(LbBar {
                        x: c.t,
                        open: last_close,
                        close: c.low,
                        dir: LbDir::Down,
                    });
                    last_low = c.low;
                    last_close = c.low;
                    confirmed = true;
                }
            }
        }

        if !confirmed {
            let reversal_needed = lines as f64;
            match dir {
                LbDir::Up => {
                    let drop = last_high - c.low;
                    if drop >= reversal_needed * (last_high - last_low).max(1.0) * 0.1 {
                        dir = LbDir::Down;
                        bars.push(LbBar {
                            x: c.t,
                            open: last_high,
                            close: c.low,
                            dir: LbDir::Down,
                        });
                        last_low = c.low;
                        last_close = c.low;
                    }
                }
                LbDir::Down => {
                    let rise = c.high - last_low;
                    if rise >= reversal_needed * (last_high - last_low).max(1.0) * 0.1 {
                        dir = LbDir::Up;
                        bars.push(LbBar {
                            x: c.t,
                            open: last_low,
                            close: c.high,
                            dir: LbDir::Up,
                        });
                        last_high = c.high;
                        last_close = c.high;
                    }
                }
            }
        }
    }

    bars
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &CandleLineBreakConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;
    let bars = compute_line_break(series, cfg.lines);

    if bars.is_empty() {
        let mut chart = ChartBuilder::on(&root)
            .caption(
                format!("{} — {} (No breaks)", cfg.title, series.symbol),
                (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
            )
            .margin(10)
            .build_cartesian_2d(t_min..t_max, 0.0..1.0)
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

    let low = bars
        .iter()
        .map(|b| b.open.min(b.close))
        .fold(f64::MAX, f64::min);
    let high = bars
        .iter()
        .map(|b| b.open.max(b.close))
        .fold(f64::MIN, f64::max);
    let pad = (high - low).max(1.0) * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!(
                "{} — {} ({}-line break)",
                cfg.title, series.symbol, cfg.lines
            ),
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

    chart
        .draw_series(bars.iter().map(|b| {
            let color = if b.dir == LbDir::Up {
                cfg.theme.profit()
            } else {
                cfg.theme.loss()
            };
            CandleStick::new(
                b.x,
                b.open,
                b.open.max(b.close),
                b.open.min(b.close),
                b.close,
                color.filled(),
                color.filled(),
                (candle_width * 10.0) as u32,
            )
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &CandleLineBreakConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &CandleLineBreakConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = CandleLineBreakConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_candle_line_break.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
