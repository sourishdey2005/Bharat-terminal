// crates/bt-viz/src/candle_range_bars.rs
// Author: Sourish Dey

//! Fixed-range bars (e.g., ₹10 range per bar). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CandleRangeBarsConfig {
    pub title: String,
    pub theme: Theme,
    pub range: f64,
}

impl Default for CandleRangeBarsConfig {
    fn default() -> Self {
        Self {
            title: "Range Bars".to_string(),
            theme: Theme::Dark,
            range: 10.0,
        }
    }
}

impl CandleRangeBarsConfig {
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

    pub fn range(mut self, r: f64) -> Self {
        self.range = r.max(0.01);
        self
    }
}

fn build_range_bars(series: &OhlcvSeries, range: f64) -> Vec<(f64, f64, f64, f64, f64, f64)> {
    let mut bars = Vec::new();
    let mut cur: Option<(f64, f64, f64, f64, f64, f64)> = None;

    for c in &series.candles {
        match cur {
            None => {
                cur = Some((c.t, c.open, c.high, c.low, c.close, c.volume));
            }
            Some((t0, o, h, l, _cl, v)) => {
                let new_h = h.max(c.high);
                let new_l = l.min(c.low);
                if new_h - new_l >= range {
                    bars.push((t0, o, h, l, c.close, v + c.volume));
                    cur = Some((c.t, c.close, c.close, c.close, c.close, 0.0));
                } else {
                    cur = Some((t0, o, new_h, new_l, c.close, v + c.volume));
                }
            }
        }
    }

    if let Some((t0, o, h, l, cl, v)) = cur {
        bars.push((t0, o, h, l, cl, v));
    }

    bars
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &CandleRangeBarsConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let bars = build_range_bars(series, cfg.range);

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
                "{} — {} (Range: ₹{:.2})",
                cfg.title, series.symbol, cfg.range
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
        .draw_series(bars.iter().map(|(t, o, h, l, c, _v)| {
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

pub fn render_png(series: &OhlcvSeries, cfg: &CandleRangeBarsConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &CandleRangeBarsConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = CandleRangeBarsConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_candle_range_bars.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
