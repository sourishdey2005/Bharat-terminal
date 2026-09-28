// crates/bt-viz/src/comp_return.rs
// Author: Sourish Dey

//! COMP: Normalized comparative return from base date.
//! Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CompReturnConfig {
    pub title: String,
    pub theme: Theme,
    pub base_date: f64,
    pub show_benchmark: bool,
}

impl Default for CompReturnConfig {
    fn default() -> Self {
        Self {
            title: "Comparative Return".to_string(),
            theme: Theme::Dark,
            base_date: 0.0,
            show_benchmark: true,
        }
    }
}

impl CompReturnConfig {
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
    pub fn base_date(mut self, d: f64) -> Self {
        self.base_date = d.max(0.0);
        self
    }
    pub fn show_benchmark(mut self, s: bool) -> Self {
        self.show_benchmark = s;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &CompReturnConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let base_idx = series
        .candles
        .iter()
        .position(|c| c.t >= cfg.base_date)
        .unwrap_or(0);
    let base_close = series.candles.get(base_idx).map(|c| c.close).unwrap_or(1.0);

    let returns: Vec<f64> = series
        .candles
        .iter()
        .map(|c| (c.close - base_close) / base_close * 100.0)
        .collect();

    let t_min = series.candles.first().map(|c| c.t).unwrap_or(0.0);
    let t_max = series.candles.last().map(|c| c.t).unwrap_or(1.0);
    let r_min = returns.iter().cloned().fold(f64::INFINITY, f64::min);
    let r_max = returns.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let pad = (r_max - r_min).max(1.0) * 0.05;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!(
                "{} — {} (Base: {:.0})",
                cfg.title, series.symbol, cfg.base_date
            ),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, (r_min - pad)..(r_max + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .y_desc("Return (%)")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(LineSeries::new(
            vec![(t_min, 0.0), (t_max, 0.0)],
            cfg.theme.border().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(LineSeries::new(
            series
                .candles
                .iter()
                .enumerate()
                .map(|(i, c)| (c.t, returns[i])),
            cfg.theme.accent().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label(&series.symbol)
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                cfg.theme.accent().stroke_width(2),
            )
        });

    if cfg.show_benchmark {
        let bench_returns: Vec<f64> = series
            .candles
            .iter()
            .map(|c| (c.close - base_close) / base_close * 100.0 * 0.5)
            .collect();
        chart
            .draw_series(LineSeries::new(
                series
                    .candles
                    .iter()
                    .enumerate()
                    .map(|(i, c)| (c.t, bench_returns[i])),
                cfg.theme.info().mix(0.5).stroke_width(1),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?
            .label("Benchmark (50%)")
            .legend(|(x, y)| {
                PathElement::new(
                    vec![(x, y), (x + 20, y)],
                    cfg.theme.info().mix(0.5).stroke_width(1),
                )
            });
    }

    chart
        .configure_series_labels()
        .background_style(cfg.theme.background().mix(0.9))
        .border_style(cfg.theme.border())
        .label_font((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &CompReturnConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &CompReturnConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = CompReturnConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_comp_return.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
