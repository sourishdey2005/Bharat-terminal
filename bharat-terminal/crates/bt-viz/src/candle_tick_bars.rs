// crates/bt-viz/src/candle_tick_bars.rs
// Author: Sourish Dey

//! Bars sampled by tick count. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CandleTickBarsConfig {
    pub title: String,
    pub theme: Theme,
    pub ticks_per_bar: usize,
}

impl Default for CandleTickBarsConfig {
    fn default() -> Self {
        Self {
            title: "Tick Bars".to_string(),
            theme: Theme::Dark,
            ticks_per_bar: 50,
        }
    }
}

impl CandleTickBarsConfig {
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

    pub fn ticks_per_bar(mut self, n: usize) -> Self {
        self.ticks_per_bar = n.max(1);
        self
    }
}

fn build_tick_bars(
    series: &OhlcvSeries,
    ticks_per_bar: usize,
) -> Vec<(f64, f64, f64, f64, f64, usize)> {
    let mut bars = Vec::new();
    let mut cur: Option<(f64, f64, f64, f64, f64, usize)> = None;

    for c in &series.candles {
        match cur {
            None => {
                cur = Some((c.t, c.open, c.high, c.low, c.close, 1));
            }
            Some((t0, o, h, l, _cl, count)) => {
                if count >= ticks_per_bar {
                    bars.push((t0, o, h, l, c.close, count));
                    cur = Some((c.t, c.close, c.close, c.close, c.close, 1));
                } else {
                    cur = Some((t0, o, h.max(c.high), l.min(c.low), c.close, count + 1));
                }
            }
        }
    }

    if let Some((t0, o, h, l, cl, count)) = cur {
        bars.push((t0, o, h, l, cl, count));
    }

    bars
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &CandleTickBarsConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let bars = build_tick_bars(series, cfg.ticks_per_bar);

    if bars.is_empty() {
        let mut chart = ChartBuilder::on(&root)
            .caption(
                format!("{} — {} (No bars)", cfg.title, series.symbol),
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

    let t_min = bars.first().unwrap().0;
    let t_max = bars.last().unwrap().0;
    let low = bars.iter().map(|b| b.3).fold(f64::MAX, f64::min);
    let high = bars.iter().map(|b| b.2).fold(f64::MIN, f64::max);
    let pad = (high - low) * 0.05;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!(
                "{} — {} ({} ticks/bar)",
                cfg.title, series.symbol, cfg.ticks_per_bar
            ),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..(t_max + 1.0), (low - pad)..(high + pad))
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
        .draw_series(bars.iter().map(|(t, o, h, l, c, _count)| {
            let color = if c >= o {
                cfg.theme.profit()
            } else {
                cfg.theme.loss()
            };
            CandleStick::new(
                *t,
                *o,
                *h,
                *l,
                *c,
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

pub fn render_png(series: &OhlcvSeries, cfg: &CandleTickBarsConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &CandleTickBarsConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = CandleTickBarsConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_candle_tick_bars.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
