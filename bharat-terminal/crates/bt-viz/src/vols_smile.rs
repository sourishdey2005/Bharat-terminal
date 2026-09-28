// crates/bt-viz/src/vols_smile.rs
// Author: Sourish Dey

//! VOLS: Volatility smile slices.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct VolSmileSlice {
    pub name: String,
    pub strikes: Vec<f64>,
    pub vols: Vec<f64>,
}

impl VolSmileSlice {
    pub fn new(name: impl Into<String>, strikes: Vec<f64>, vols: Vec<f64>) -> Self {
        Self {
            name: name.into(),
            strikes,
            vols,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VolsSmileConfig {
    pub title: String,
    pub theme: Theme,
    pub show_atm: bool,
}

impl Default for VolsSmileConfig {
    fn default() -> Self {
        Self {
            title: "Volatility Smile Slices".to_string(),
            theme: Theme::Dark,
            show_atm: true,
        }
    }
}

impl VolsSmileConfig {
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
    pub fn show_atm(mut self, s: bool) -> Self {
        self.show_atm = s;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    slices: &[VolSmileSlice],
    cfg: &VolsSmileConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if slices.is_empty() {
        return Err(BtError::EmptySeries("vol smile slices".into()));
    }
    fill_background(&root, cfg.theme)?;

    let colors = [
        cfg.theme.profit(),
        cfg.theme.info(),
        cfg.theme.accent(),
        cfg.theme.loss(),
        RGBColor(0xB0, 0x66, 0xFF),
        RGBColor(0xFF, 0x8A, 0x00),
    ];

    let mut k_min = f64::INFINITY;
    let mut k_max = f64::NEG_INFINITY;
    let mut v_min = f64::INFINITY;
    let mut v_max = f64::NEG_INFINITY;

    for slice in slices {
        for &k in &slice.strikes {
            k_min = k_min.min(k);
            k_max = k_max.max(k);
        }
        for &v in &slice.vols {
            v_min = v_min.min(v);
            v_max = v_max.max(v);
        }
    }

    let k_range = (k_max - k_min).max(1.0);
    let v_range = (v_max - v_min).max(0.1);
    k_min -= k_range * 0.05;
    k_max += k_range * 0.05;
    v_min -= v_range * 0.1;
    v_max += v_range * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            cfg.title.clone(),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(k_min..k_max, v_min..v_max)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .x_desc("Strike")
        .y_desc("Implied Vol (%)")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    for (idx, slice) in slices.iter().enumerate() {
        let color = colors[idx % colors.len()];
        let points: Vec<(f64, f64)> = slice
            .strikes
            .iter()
            .zip(slice.vols.iter())
            .map(|(&k, &v)| (k, v))
            .collect();

        chart
            .draw_series(LineSeries::new(points.clone(), color.stroke_width(2)))
            .map_err(|e| BtError::Render(e.to_string()))?
            .label(&slice.name)
            .legend(move |(x, y)| {
                PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2))
            });

        chart
            .draw_series(
                points
                    .iter()
                    .map(|&(k, v)| Circle::new((k, v), 3, color.filled())),
            )
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    if cfg.show_atm {
        let atm_strike = (k_min + k_max) / 2.0;
        chart
            .draw_series(LineSeries::new(
                vec![(atm_strike, v_min), (atm_strike, v_max)],
                cfg.theme.border().mix(0.5).stroke_width(1),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?
            .label("ATM")
            .legend(|(x, y)| {
                PathElement::new(
                    vec![(x, y), (x + 20, y)],
                    cfg.theme.border().mix(0.5).stroke_width(1),
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

pub fn render_png(slices: &[VolSmileSlice], cfg: &VolsSmileConfig, path: &str) -> Result<()> {
    render(png_root(path)?, slices, cfg)
}

pub fn render_svg(slices: &[VolSmileSlice], cfg: &VolsSmileConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, slices, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let slices = vec![
            VolSmileSlice::new(
                "1M",
                vec![90.0, 95.0, 100.0, 105.0, 110.0],
                vec![25.0, 22.0, 20.0, 21.0, 24.0],
            ),
            VolSmileSlice::new(
                "3M",
                vec![90.0, 95.0, 100.0, 105.0, 110.0],
                vec![28.0, 25.0, 23.0, 24.0, 27.0],
            ),
        ];
        let cfg = VolsSmileConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_vols_smile.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&slices, &cfg, &path).unwrap();
    }
}
