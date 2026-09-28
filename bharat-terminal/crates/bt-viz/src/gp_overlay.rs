// crates/bt-viz/src/gp_overlay.rs
// Author: Sourish Dey

//! GP: Multi-asset price overlay with split/dividend adjustments.
//! Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct GpOverlayConfig {
    pub title: String,
    pub theme: Theme,
    pub base_value: f64,
    pub adjustment_factor: f64,
}

impl Default for GpOverlayConfig {
    fn default() -> Self {
        Self {
            title: "Multi-Asset Price Overlay".to_string(),
            theme: Theme::Dark,
            base_value: 100.0,
            adjustment_factor: 1.0,
        }
    }
}

impl GpOverlayConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn base_value(mut self, v: f64) -> Self { self.base_value = v.max(1.0); self }
    pub fn adjustment_factor(mut self, f: f64) -> Self { self.adjustment_factor = f.max(0.0001); self }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series_list: &[OhlcvSeries],
    cfg: &GpOverlayConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    for s in series_list { s.validate()?; }
    fill_background(&root, cfg.theme)?;

    let colors = [
        cfg.theme.profit(),
        cfg.theme.info(),
        cfg.theme.accent(),
        cfg.theme.loss(),
        RGBColor(0xB0, 0x66, 0xFF),
        RGBColor(0xFF, 0x8A, 0x00),
    ];

    let mut all_normalized: Vec<Vec<f64>> = Vec::new();
    let mut global_min = f64::INFINITY;
    let mut global_max = f64::NEG_INFINITY;

    for series in series_list {
        let first_close = series.candles.first().map(|c| c.close).unwrap_or(1.0);
        let normalized: Vec<f64> = series
            .candles
            .iter()
            .map(|c| c.close / first_close * cfg.base_value * cfg.adjustment_factor)
            .collect();
        for &v in &normalized {
            global_min = global_min.min(v);
            global_max = global_max.max(v);
        }
        all_normalized.push(normalized);
    }

    let n_max = all_normalized.iter().map(|v| v.len()).max().unwrap_or(0);
    let t_min = 0.0;
    let t_max = n_max as f64;
    let pad = (global_max - global_min).max(1.0) * 0.05;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — Base {:.0} Adj {:.2}", cfg.title, cfg.base_value, cfg.adjustment_factor),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, (global_min - pad)..(global_max + pad))
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
            vec![(t_min, cfg.base_value), (t_max, cfg.base_value)],
            cfg.theme.border().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    for (idx, (series, normalized)) in series_list.iter().zip(all_normalized.iter()).enumerate() {
        let color = colors[idx % colors.len()];
        chart
            .draw_series(LineSeries::new(
                normalized.iter().enumerate().map(|(i, &v)| (i as f64, v)),
                color.stroke_width(2),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?
            .label(&series.symbol)
            .legend(move |(x, y)| {
                PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2))
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

pub fn render_png(series_list: &[OhlcvSeries], cfg: &GpOverlayConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series_list, cfg)
}

pub fn render_svg(series_list: &[OhlcvSeries], cfg: &GpOverlayConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series_list, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let a = synthetic_ohlcv("AAA", 100, 1, 100.0);
        let b = synthetic_ohlcv("BBB", 100, 2, 100.0);
        let series_list = vec![a, b];
        let cfg = GpOverlayConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_gp_overlay.png").to_str().unwrap().to_string();
        render_png(&series_list, &cfg, &path).unwrap();
    }
}
