// crates/bt-viz/src/trix.rs
// Author: Sourish Dey

//! TRIX indicator chart. Made by Sourish Dey.

use bt_analytics::ema;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct TrixConfig {
    pub title: String,
    pub theme: Theme,
    pub period: usize,
}

impl Default for TrixConfig {
    fn default() -> Self {
        Self {
            title: "TRIX".to_string(),
            theme: Theme::Dark,
            period: 15,
        }
    }
}

impl TrixConfig {
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
    pub fn period(mut self, p: usize) -> Self {
        self.period = p.max(2);
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &TrixConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let ema1 = ema(series, cfg.period);
    let n = series.candles.len();
    let mut ema2 = vec![f64::NAN; n];
    let mut ema3 = vec![f64::NAN; n];
    let mut trix = vec![f64::NAN; n];

    for i in 0..n {
        if !ema1[i].is_nan() {
            ema2[i] = ema1[i];
            for j in (i + 1)..n {
                if !ema1[j].is_nan() {
                    ema2[j] = ema1[j];
                    break;
                }
            }
        }
    }

    for i in 0..n {
        if !ema2[i].is_nan() {
            ema3[i] = ema2[i];
            for j in (i + 1)..n {
                if !ema2[j].is_nan() {
                    ema3[j] = ema2[j];
                    break;
                }
            }
        }
    }

    for i in 1..n {
        if !ema3[i].is_nan() && !ema3[i - 1].is_nan() && ema3[i - 1] != 0.0 {
            trix[i] = (ema3[i] - ema3[i - 1]) / ema3[i - 1] * 100.0;
        }
    }

    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            &cfg.title,
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(50)
        .build_cartesian_2d(t_min..t_max, -5.0..5.0)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !trix[i].is_nan() {
                    Some((c.t, trix[i]))
                } else {
                    None
                }
            }),
            cfg.theme.accent().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(LineSeries::new(
            vec![(t_min, 0.0), (t_max, 0.0)],
            cfg.theme.border().stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &TrixConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &TrixConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = TrixConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_trix.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
