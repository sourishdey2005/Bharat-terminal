// crates/bt-viz/src/rbi_policy.rs
// Author: Sourish Dey

//! RBI monetary policy chart (repo, reverse repo, CRR, SDF). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct RbiPolicyConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for RbiPolicyConfig {
    fn default() -> Self {
        Self {
            title: "RBI Monetary Policy".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl RbiPolicyConfig {
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
}

fn sample_policy() -> Vec<(String, f64, f64, f64, f64)> {
    vec![
        ("Repo".to_string(), 6.50, 6.50, 6.50, 6.50),
        ("Reverse Repo".to_string(), 3.35, 3.35, 3.35, 3.35),
        ("SDF".to_string(), 6.25, 6.25, 6.25, 6.25),
        ("MSF".to_string(), 6.75, 6.75, 6.75, 6.75),
        ("CRR".to_string(), 4.50, 4.50, 4.50, 4.50),
        ("SLR".to_string(), 18.0, 18.0, 18.0, 18.0),
        ("Bank Rate".to_string(), 6.75, 6.75, 6.75, 6.75),
        ("Base Rate".to_string(), 9.45, 9.45, 9.45, 9.45),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &RbiPolicyConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_policy();
    let (w, h) = root.dim_in_pixel();

    root.draw(&Text::new(
        format!("{} — {}", cfg.title, series.symbol),
        (10, 10),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let chart = ChartBuilder::on(&root)
        .margin(15)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0f64..data.len() as f64, 0f64..20f64)
        .map_err(|e| BtError::Render(e.to_string()))?;

    let mut chart = chart;
    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 11).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .x_labels(data.len())
        .x_label_formatter(&|x| {
            let idx = *x as usize;
            if idx < data.len() {
                data[idx].0.clone()
            } else {
                String::new()
            }
        })
        .y_desc("Rate (%)")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let palette = cfg.theme.categorical(0);
    for (i, (name, r1, r2, r3, r4)) in data.iter().enumerate() {
        let color = palette[i % palette.len()];
        let vals = [*r1, *r2, *r3, *r4];
        let bar_w = 0.6 / vals.len() as f64;
        for (j, &v) in vals.iter().enumerate() {
            let x0 = i as f64 + j as f64 * bar_w;
            chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(x0, 0.0), (x0 + bar_w * 0.9, v)],
                    color.filled(),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
        root.draw(&Text::new(
            format!("{:.2}", r1),
            (60 + i as i32 * ((w as i32 - 120) / data.len() as i32), 50),
            (LABEL_FONT, 10).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        let _ = name;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &RbiPolicyConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &RbiPolicyConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("RBI", 100, 1, 100.0);
        let cfg = RbiPolicyConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_rbi_policy.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
