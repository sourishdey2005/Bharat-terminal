// crates/bt-viz/src/put_call_ratio.rs
// Author: Sourish Dey

//! Put/Call ratio chart. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct PutCallRatioConfig {
    pub title: String,
    pub theme: Theme,
    pub window: usize,
}

impl Default for PutCallRatioConfig {
    fn default() -> Self {
        Self {
            title: "Put/Call Ratio".to_string(),
            theme: Theme::Dark,
            window: 10,
        }
    }
}

impl PutCallRatioConfig {
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
    pub fn window(mut self, w: usize) -> Self {
        self.window = w;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &PutCallRatioConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let volumes: Vec<f64> = series.candles.iter().map(|c| c.volume).collect();
    if volumes.len() < cfg.window {
        return Err(BtError::InvalidInput(
            "insufficient data for PCR window".into(),
        ));
    }

    let mut pcr = Vec::with_capacity(volumes.len());
    for i in 0..volumes.len() {
        if i + 1 >= cfg.window {
            let window = &volumes[i + 1 - cfg.window..=i];
            let avg = window.iter().sum::<f64>() / window.len() as f64;
            pcr.push(if avg > 0.0 { volumes[i] / avg } else { 1.0 });
        } else {
            pcr.push(1.0);
        }
    }

    let y_max = pcr.iter().cloned().fold(1.0_f64, f64::max);
    let n = pcr.len() as f64;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            &cfg.title,
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(15)
        .x_label_area_size(35)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..n, 0.0..(y_max * 1.2))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .x_desc("Bar")
        .y_desc("Put/Call Ratio")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let points: Vec<(f64, f64)> = pcr
        .iter()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect();

    chart
        .draw_series(LineSeries::new(
            points.clone(),
            cfg.theme.info().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(0.0, 1.0), (n, 1.0)],
            cfg.theme.border().stroke_width(1),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &PutCallRatioConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &PutCallRatioConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = PutCallRatioConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_put_call_ratio.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
