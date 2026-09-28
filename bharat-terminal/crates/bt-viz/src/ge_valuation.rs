// crates/bt-viz/src/ge_valuation.rs
// Author: Sourish Dey

//! GE: Valuation multiple bands.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct GeValuationConfig {
    pub title: String,
    pub theme: Theme,
    pub multiple: String,
    pub current_value: f64,
    pub band_low: f64,
    pub band_mid: f64,
    pub band_high: f64,
    pub historical_values: Vec<f64>,
}

impl Default for GeValuationConfig {
    fn default() -> Self {
        Self {
            title: "Valuation Multiple Bands".to_string(),
            theme: Theme::Dark,
            multiple: "P/E".into(),
            current_value: 20.0,
            band_low: 15.0,
            band_mid: 20.0,
            band_high: 25.0,
            historical_values: (0..20)
                .map(|i| 15.0 + (i as f64 * 0.5) + (i as f64 * 0.1).sin() * 3.0)
                .collect(),
        }
    }
}

impl GeValuationConfig {
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
    pub fn multiple(mut self, m: impl Into<String>) -> Self {
        self.multiple = m.into();
        self
    }
    pub fn current_value(mut self, v: f64) -> Self {
        self.current_value = v.max(0.0);
        self
    }
    pub fn band_low(mut self, v: f64) -> Self {
        self.band_low = v.max(0.0);
        self
    }
    pub fn band_mid(mut self, v: f64) -> Self {
        self.band_mid = v.max(0.0);
        self
    }
    pub fn band_high(mut self, v: f64) -> Self {
        self.band_high = v.max(0.0);
        self
    }
    pub fn historical_values(mut self, v: Vec<f64>) -> Self {
        self.historical_values = v;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    cfg: &GeValuationConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    fill_background(&root, cfg.theme)?;

    let mut all_vals = cfg.historical_values.clone();
    all_vals.push(cfg.current_value);
    let v_min = all_vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let v_max = all_vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let pad = (v_max - v_min).max(1.0) * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — {} Multiple", cfg.title, cfg.multiple),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(
            0.0..(cfg.historical_values.len() + 1) as f64,
            (v_min - pad)..(v_max + pad),
        )
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
            vec![
                (0.0, cfg.band_low),
                ((cfg.historical_values.len() + 1) as f64, cfg.band_low),
            ],
            cfg.theme.loss().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Band Low")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                cfg.theme.loss().mix(0.5).stroke_width(1),
            )
        });

    chart
        .draw_series(LineSeries::new(
            vec![
                (0.0, cfg.band_high),
                ((cfg.historical_values.len() + 1) as f64, cfg.band_high),
            ],
            cfg.theme.profit().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Band High")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                cfg.theme.profit().mix(0.5).stroke_width(1),
            )
        });

    chart
        .draw_series(LineSeries::new(
            vec![
                (0.0, cfg.band_mid),
                ((cfg.historical_values.len() + 1) as f64, cfg.band_mid),
            ],
            cfg.theme.accent().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Band Mid")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                cfg.theme.accent().mix(0.5).stroke_width(1),
            )
        });

    chart
        .draw_series(LineSeries::new(
            cfg.historical_values
                .iter()
                .enumerate()
                .map(|(i, &v)| (i as f64 + 1.0, v)),
            cfg.theme.info().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Historical")
        .legend(|(x, y)| {
            PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.info().stroke_width(2))
        });

    chart
        .draw_series(std::iter::once(Circle::new(
            (cfg.historical_values.len() as f64 + 1.0, cfg.current_value),
            6,
            cfg.theme.accent().filled(),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Current")
        .legend(|(x, y)| Circle::new((x + 10, y), 5, cfg.theme.accent().filled()));

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

pub fn render_png(cfg: &GeValuationConfig, path: &str) -> Result<()> {
    render(png_root(path)?, cfg)
}

pub fn render_svg(cfg: &GeValuationConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let cfg = GeValuationConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_ge_valuation.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&cfg, &path).unwrap();
    }
}
