// crates/bt-viz/src/williams_r.rs
// Author: Sourish Dey

//! Williams %R. Made by Sourish Dey.

use bt_analytics::williams_r;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct WilliamsRConfig {
    pub title: String,
    pub theme: Theme,
    pub period: usize,
}

impl Default for WilliamsRConfig {
    fn default() -> Self {
        Self {
            title: "Williams %R".to_string(),
            theme: Theme::Dark,
            period: 14,
        }
    }
}

impl WilliamsRConfig {
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
    cfg: &WilliamsRConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let wr_vals = williams_r(series, cfg.period);
    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — {} (Period: {})", cfg.title, series.symbol, cfg.period),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, -100.0..0.0)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(LineSeries::new(
            vec![(t_min, -20.0), (t_max, -20.0)],
            cfg.theme.loss().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(LineSeries::new(
            vec![(t_min, -80.0), (t_max, -80.0)],
            cfg.theme.profit().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(LineSeries::new(
            vec![(t_min, -50.0), (t_max, -50.0)],
            cfg.theme.border().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !wr_vals[i].is_nan() {
                    Some((c.t, wr_vals[i]))
                } else {
                    None
                }
            }),
            cfg.theme.info().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    for i in 0..series.candles.len() - 1 {
        if !wr_vals[i].is_nan() && !wr_vals[i + 1].is_nan() {
            if wr_vals[i] >= -20.0 && wr_vals[i + 1] >= -20.0 {
                chart
                    .draw_series(std::iter::once(Polygon::new(
                        vec![
                            (series.candles[i].t, -20.0),
                            (series.candles[i + 1].t, -20.0),
                            (series.candles[i + 1].t, wr_vals[i + 1].min(0.0)),
                            (series.candles[i].t, wr_vals[i].min(0.0)),
                        ],
                        cfg.theme.loss().mix(0.1).filled(),
                    )))
                    .map_err(|e| BtError::Render(e.to_string()))?;
            } else if wr_vals[i] <= -80.0 && wr_vals[i + 1] <= -80.0 {
                chart
                    .draw_series(std::iter::once(Polygon::new(
                        vec![
                            (series.candles[i].t, -80.0),
                            (series.candles[i + 1].t, -80.0),
                            (series.candles[i + 1].t, wr_vals[i + 1].max(-100.0)),
                            (series.candles[i].t, wr_vals[i].max(-100.0)),
                        ],
                        cfg.theme.profit().mix(0.1).filled(),
                    )))
                    .map_err(|e| BtError::Render(e.to_string()))?;
            }
        }
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &WilliamsRConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &WilliamsRConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = WilliamsRConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_williams_r.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
