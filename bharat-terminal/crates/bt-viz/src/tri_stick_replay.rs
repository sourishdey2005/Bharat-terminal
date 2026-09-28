// crates/bt-viz/src/tri_stick_replay.rs
// Author: Sourish Dey

//! Bar-by-bar market replay with triangle sticks building up over the
//! session. Made by Sourish Dey.

use bt_analytics::vwap;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct TriStickReplayConfig {
    pub title: String,
    pub theme: Theme,
    pub bars_to_show: usize,
    pub show_vwap: bool,
}

impl Default for TriStickReplayConfig {
    fn default() -> Self {
        Self {
            title: "Tri-Stick Replay".to_string(),
            theme: Theme::Dark,
            bars_to_show: 40,
            show_vwap: true,
        }
    }
}

impl TriStickReplayConfig {
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

    pub fn bars_to_show(mut self, n: usize) -> Self {
        self.bars_to_show = n.max(1);
        self
    }

    pub fn show_vwap(mut self, show: bool) -> Self {
        self.show_vwap = show;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &TriStickReplayConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let total = series.candles.len();
    let visible = cfg.bars_to_show.min(total);
    let sub = OhlcvSeries::new(&series.symbol, series.candles[..visible].to_vec());

    let t_min = sub.candles.first().unwrap().t;
    let t_max = sub.candles.last().unwrap().t;
    let low = sub.candles.iter().map(|c| c.low).fold(f64::MAX, f64::min);
    let high = sub.candles.iter().map(|c| c.high).fold(f64::MIN, f64::max);
    let pad = (high - low).max(1.0) * 0.08;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!(
                "{} — {} (bar {}/{})",
                cfg.title, series.symbol, visible, total
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

    let bar_width = ((t_max - t_min) / visible as f64).max(0.3) * 0.35;

    for c in &sub.candles {
        let color = if c.is_bullish() {
            cfg.theme.profit()
        } else {
            cfg.theme.loss()
        };

        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![(c.t, c.high), (c.t, c.low)],
                color.stroke_width(1),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;

        chart
            .draw_series(std::iter::once(Polygon::new(
                vec![
                    (c.t, c.close),
                    (c.t - bar_width, c.open),
                    (c.t + bar_width, c.open),
                ],
                color.filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    if cfg.show_vwap {
        let vwap_vals = vwap(&sub);
        chart
            .draw_series(LineSeries::new(
                sub.candles.iter().enumerate().filter_map(|(i, c)| {
                    if i < vwap_vals.len() && !vwap_vals[i].is_nan() {
                        Some((c.t, vwap_vals[i]))
                    } else {
                        None
                    }
                }),
                cfg.theme.info().stroke_width(2),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?
            .label("VWAP")
            .legend(move |(x, y)| {
                PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.info().stroke_width(2))
            });

        chart
            .configure_series_labels()
            .background_style(cfg.theme.background().mix(0.9))
            .border_style(cfg.theme.border())
            .label_font((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
            .draw()
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    let (w, h) = root.dim_in_pixel();
    root.draw(&Text::new(
        "REPLAY".to_string(),
        (w as i32 - 90, 40),
        (LABEL_FONT, 12).into_font().color(&cfg.theme.accent()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let bar_y = h as i32 - 40;
    let bar_x0 = 10i32;
    let bar_x1 = w as i32 - 10;
    let frac = visible as f64 / total as f64;
    let fill_x = bar_x0 + ((bar_x1 - bar_x0) as f64 * frac) as i32;

    root.draw(&Rectangle::new(
        [(bar_x0, bar_y), (bar_x1, bar_y + 8)],
        cfg.theme.border().filled(),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    root.draw(&Rectangle::new(
        [(bar_x0, bar_y), (fill_x, bar_y + 8)],
        cfg.theme.accent().filled(),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &TriStickReplayConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &TriStickReplayConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = TriStickReplayConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_tri_stick_replay.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
