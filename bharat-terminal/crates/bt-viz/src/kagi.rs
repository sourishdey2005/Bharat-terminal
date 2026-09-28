// crates/bt-viz/src/kagi.rs
// Author: Sourish Dey

//! Kagi Chart (reversal lines). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct KagiConfig {
    pub title: String,
    pub theme: Theme,
    pub reversal_pct: f64,
}

impl Default for KagiConfig {
    fn default() -> Self {
        Self {
            title: "Kagi".to_string(),
            theme: Theme::Dark,
            reversal_pct: 4.0,
        }
    }
}

impl KagiConfig {
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

    pub fn reversal_pct(mut self, pct: f64) -> Self {
        self.reversal_pct = pct.max(0.1);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum KagiDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy)]
struct KagiPoint {
    x: f64,
    price: f64,
    kagi_dir: KagiDirection,
}

fn compute_kagi(series: &OhlcvSeries, reversal_pct: f64) -> Vec<KagiPoint> {
    if series.candles.is_empty() {
        return Vec::new();
    }

    let t_min = series.candles[0].t;
    let t_max = series.candles[series.candles.len() - 1].t;
    let n = series.candles.len();

    let mut points = Vec::new();
    let mut direction = KagiDirection::Up;
    let mut extreme = series.candles[0].close;
    let mut prev_price = series.candles[0].close;

    points.push(KagiPoint {
        x: t_min,
        price: series.candles[0].close,
        kagi_dir: KagiDirection::Up,
    });

    let reversal_factor = reversal_pct / 100.0;

    for i in 1..n {
        let close = series.candles[i].close;
        let x = series.candles[i].t;

        match direction {
            KagiDirection::Up => {
                if close > extreme {
                    extreme = close;
                    points.push(KagiPoint { x, price: close, kagi_dir: direction });
                } else if (extreme - close) / extreme > reversal_factor {
                    direction = KagiDirection::Down;
                    points.push(KagiPoint { x, price: extreme, kagi_dir: KagiDirection::Up });
                    prev_price = extreme;
                    extreme = close;
                    points.push(KagiPoint { x, price: close, kagi_dir: KagiDirection::Down });
                }
            }
            KagiDirection::Down => {
                if close < extreme {
                    extreme = close;
                    points.push(KagiPoint { x, price: close, kagi_dir: direction });
                } else if (close - prev_price) / prev_price > reversal_factor {
                    direction = KagiDirection::Up;
                    points.push(KagiPoint { x, price: extreme, kagi_dir: KagiDirection::Down });
                    prev_price = extreme;
                    extreme = close;
                    points.push(KagiPoint { x, price: close, kagi_dir: KagiDirection::Up });
                }
            }
        }
    }

    let _ = t_max;
    points
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &KagiConfig,
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
            format!("{} — {} (Reversal: {:.1}%)", cfg.title, series.symbol, cfg.reversal_pct),
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

    for i in 0..kagi.len() - 1 {
        let p1 = &kagi[i];
        let p2 = &kagi[i + 1];
        let color = if p2.kagi_dir == KagiDirection::Up {
            cfg.theme.profit()
        } else {
            cfg.theme.loss()
        };
        let width = if p2.kagi_dir == KagiDirection::Up { 1 } else { 3 };

        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![(p1.x, p1.price), (p2.x, p2.price)],
                color.stroke_width(width),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    chart
        .draw_series(
            kagi.iter()
                .enumerate()
                .filter(|(i, _)| *i % 5 == 0 || *i == kagi.len() - 1)
                .map(|(i, p)| {
                    let _ = i;
                    Circle::new((p.x, p.price), 3, cfg.theme.text().filled())
                }),
        )
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &KagiConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &KagiConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = KagiConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_kagi.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
