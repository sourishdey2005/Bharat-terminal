// crates/bt-viz/src/candle_kagi.rs
// Author: Sourish Dey

//! Kagi reversals as candles: thick=bullish, thin=bearish. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CandleKagiConfig {
    pub title: String,
    pub theme: Theme,
    pub reversal_pct: f64,
}

impl Default for CandleKagiConfig {
    fn default() -> Self {
        Self {
            title: "Kagi Candles".to_string(),
            theme: Theme::Dark,
            reversal_pct: 4.0,
        }
    }
}

impl CandleKagiConfig {
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

    pub fn reversal_pct(mut self, p: f64) -> Self {
        self.reversal_pct = p.max(0.1);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum KagiDir {
    Up,
    Down,
}

struct KagiPoint {
    x: f64,
    price: f64,
    dir: KagiDir,
}

fn compute_kagi(series: &OhlcvSeries, reversal_pct: f64) -> Vec<KagiPoint> {
    if series.candles.is_empty() {
        return Vec::new();
    }

    let mut points = Vec::new();
    let mut direction = KagiDir::Up;
    let mut extreme = series.candles[0].close;
    let mut prev_price = series.candles[0].close;

    points.push(KagiPoint {
        x: series.candles[0].t,
        price: series.candles[0].close,
        dir: KagiDir::Up,
    });

    let reversal_factor = reversal_pct / 100.0;

    for i in 1..series.candles.len() {
        let close = series.candles[i].close;
        let x = series.candles[i].t;

        match direction {
            KagiDir::Up => {
                if close > extreme {
                    extreme = close;
                    points.push(KagiPoint {
                        x,
                        price: close,
                        dir: direction,
                    });
                } else if (extreme - close) / extreme > reversal_factor {
                    direction = KagiDir::Down;
                    points.push(KagiPoint {
                        x,
                        price: extreme,
                        dir: KagiDir::Up,
                    });
                    prev_price = extreme;
                    extreme = close;
                    points.push(KagiPoint {
                        x,
                        price: close,
                        dir: KagiDir::Down,
                    });
                }
            }
            KagiDir::Down => {
                if close < extreme {
                    extreme = close;
                    points.push(KagiPoint {
                        x,
                        price: close,
                        dir: direction,
                    });
                } else if (close - prev_price) / prev_price > reversal_factor {
                    direction = KagiDir::Up;
                    points.push(KagiPoint {
                        x,
                        price: extreme,
                        dir: KagiDir::Down,
                    });
                    prev_price = extreme;
                    extreme = close;
                    points.push(KagiPoint {
                        x,
                        price: close,
                        dir: KagiDir::Up,
                    });
                }
            }
        }
    }

    points
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &CandleKagiConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;
    let kagi = compute_kagi(series, cfg.reversal_pct);

    if kagi.len() < 2 {
        let mut chart = ChartBuilder::on(&root)
            .caption(
                format!("{} — {} (No reversals)", cfg.title, series.symbol),
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

    let low = kagi.iter().map(|p| p.price).fold(f64::MAX, f64::min);
    let high = kagi.iter().map(|p| p.price).fold(f64::MIN, f64::max);
    let pad = (high - low).max(1.0) * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!(
                "{} — {} (Reversal: {:.1}%)",
                cfg.title, series.symbol, cfg.reversal_pct
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

    let slot = (t_max - t_min) / kagi.len() as f64;

    for i in 0..kagi.len() - 1 {
        let p1 = &kagi[i];
        let p2 = &kagi[i + 1];
        let is_bull = p2.dir == KagiDir::Up;
        let color = if is_bull {
            cfg.theme.profit()
        } else {
            cfg.theme.loss()
        };
        let w = if is_bull { 4.0 } else { 1.5 };

        chart
            .draw_series(std::iter::once(Rectangle::new(
                [
                    (p1.x, p1.price.min(p2.price)),
                    (p2.x, p1.price.max(p2.price) + w),
                ],
                color.filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    let _ = slot;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &CandleKagiConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &CandleKagiConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = CandleKagiConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_candle_kagi.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
