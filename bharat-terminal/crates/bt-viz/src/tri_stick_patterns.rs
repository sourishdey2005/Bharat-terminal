// crates/bt-viz/src/tri_stick_patterns.rs
// Author: Sourish Dey

//! Triangle-stick chart with auto-annotated hammer, doji and engulfing
//! candlestick patterns. Made by Sourish Dey.

use bt_analytics::sma;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct TriStickPatternsConfig {
    pub title: String,
    pub theme: Theme,
    pub show_hammer: bool,
    pub show_doji: bool,
    pub show_engulfing: bool,
    pub body_tolerance: f64,
    pub ma_period: usize,
}

impl Default for TriStickPatternsConfig {
    fn default() -> Self {
        Self {
            title: "Tri-Stick Patterns".to_string(),
            theme: Theme::Dark,
            show_hammer: true,
            show_doji: true,
            show_engulfing: true,
            body_tolerance: 0.1,
            ma_period: 20,
        }
    }
}

impl TriStickPatternsConfig {
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

    pub fn show_hammer(mut self, show: bool) -> Self {
        self.show_hammer = show;
        self
    }

    pub fn show_doji(mut self, show: bool) -> Self {
        self.show_doji = show;
        self
    }

    pub fn show_engulfing(mut self, show: bool) -> Self {
        self.show_engulfing = show;
        self
    }

    pub fn body_tolerance(mut self, tol: f64) -> Self {
        self.body_tolerance = tol.clamp(0.01, 0.9);
        self
    }

    pub fn ma_period(mut self, p: usize) -> Self {
        self.ma_period = p.max(2);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pattern {
    Hammer,
    Doji,
    EngulfingBull,
    EngulfingBear,
}

fn detect_patterns(series: &OhlcvSeries, tol: f64) -> Vec<Option<Pattern>> {
    let candles = &series.candles;
    let mut out = vec![None; candles.len()];

    for i in 0..candles.len() {
        let c = &candles[i];
        let range = (c.high - c.low).max(1e-9);
        let body = (c.close - c.open).abs();
        let lower_wick = c.open.min(c.close) - c.low;
        let upper_wick = c.high - c.open.max(c.close);

        if body / range < tol {
            out[i] = Some(Pattern::Doji);
            continue;
        }

        if lower_wick >= 2.0 * body && upper_wick <= body && body / range < 0.5 {
            out[i] = Some(Pattern::Hammer);
            continue;
        }

        if i > 0 {
            let p = &candles[i - 1];
            if c.is_bullish() && !p.is_bullish() && c.open <= p.close && c.close >= p.open {
                out[i] = Some(Pattern::EngulfingBull);
            } else if !c.is_bullish() && p.is_bullish() && c.open >= p.close && c.close <= p.open {
                out[i] = Some(Pattern::EngulfingBear);
            }
        }
    }

    out
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &TriStickPatternsConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;
    let low = series
        .candles
        .iter()
        .map(|c| c.low)
        .fold(f64::MAX, f64::min);
    let high = series
        .candles
        .iter()
        .map(|c| c.high)
        .fold(f64::MIN, f64::max);
    let pad = (high - low).max(1.0) * 0.08;

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

    let bar_width = ((t_max - t_min) / series.candles.len() as f64).max(0.3) * 0.35;

    for c in &series.candles {
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

    let ma = sma(series, cfg.ma_period);
    chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if i < ma.len() && !ma[i].is_nan() {
                    Some((c.t, ma[i]))
                } else {
                    None
                }
            }),
            cfg.theme.info().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label(format!("SMA({})", cfg.ma_period))
        .legend(move |(x, y)| {
            PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.info().stroke_width(2))
        });

    let patterns = detect_patterns(series, cfg.body_tolerance);
    let mut n_hammer = 0usize;
    let mut n_doji = 0usize;
    let mut n_engulf = 0usize;

    for (i, pat) in patterns.iter().enumerate() {
        let pat = match pat {
            Some(p) => *p,
            None => continue,
        };
        let c = &series.candles[i];
        let range = (c.high - c.low).max(1e-9);

        let (label, below, color) = match pat {
            Pattern::Hammer if cfg.show_hammer => {
                n_hammer += 1;
                ("HAMMER", true, cfg.theme.profit())
            }
            Pattern::Doji if cfg.show_doji => {
                n_doji += 1;
                ("DOJI", false, cfg.theme.info())
            }
            Pattern::EngulfingBull if cfg.show_engulfing => {
                n_engulf += 1;
                ("ENG↑", false, cfg.theme.profit())
            }
            Pattern::EngulfingBear if cfg.show_engulfing => {
                n_engulf += 1;
                ("ENG↓", true, cfg.theme.loss())
            }
            _ => continue,
        };

        let y = if below {
            c.low - range * 0.06
        } else {
            c.high + range * 0.06
        };

        chart
            .draw_series(std::iter::once(Text::new(
                label.to_string(),
                (c.t, y),
                (LABEL_FONT, 9).into_font().color(&color),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    chart
        .configure_series_labels()
        .background_style(cfg.theme.background().mix(0.9))
        .border_style(cfg.theme.border())
        .label_font((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    root.draw(&Text::new(
        format!(
            "Hammer: {}   Doji: {}   Engulfing: {}",
            n_hammer, n_doji, n_engulf
        ),
        (15, 40),
        (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &TriStickPatternsConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &TriStickPatternsConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = TriStickPatternsConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_tri_stick_patterns.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
