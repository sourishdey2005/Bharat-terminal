// crates/bt-viz/src/sector_rotation.rs
// Author: Sourish Dey

//! Sector rotation momentum — visualizes relative strength of sectors
//! to identify rotation patterns and leading/lagging sectors.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct SectorRotationConfig {
    pub title: String,
    pub theme: Theme,
    pub lookback_periods: usize,
    pub show_relative_strength: bool,
}

impl Default for SectorRotationConfig {
    fn default() -> Self {
        Self {
            title: "Sector Rotation".to_string(),
            theme: Theme::Dark,
            lookback_periods: 20,
            show_relative_strength: true,
        }
    }
}

impl SectorRotationConfig {
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

    pub fn lookback_periods(mut self, p: usize) -> Self {
        self.lookback_periods = p.max(5);
        self
    }

    pub fn show_relative_strength(mut self, show: bool) -> Self {
        self.show_relative_strength = show;
        self
    }
}

#[derive(Debug, Clone)]
pub struct SectorMomentum {
    pub name: String,
    pub momentum: f64,
    pub relative_strength: f64,
    pub rank: usize,
}

fn sample_sectors() -> Vec<SectorMomentum> {
    vec![
        SectorMomentum { name: "IT".to_string(), momentum: 2.1, relative_strength: 1.08, rank: 1 },
        SectorMomentum { name: "Banking".to_string(), momentum: 1.5, relative_strength: 1.03, rank: 2 },
        SectorMomentum { name: "Auto".to_string(), momentum: 1.2, relative_strength: 1.01, rank: 3 },
        SectorMomentum { name: "Metals".to_string(), momentum: 0.8, relative_strength: 0.99, rank: 4 },
        SectorMomentum { name: "FMCG".to_string(), momentum: 0.3, relative_strength: 0.97, rank: 5 },
        SectorMomentum { name: "Pharma".to_string(), momentum: -0.2, relative_strength: 0.95, rank: 6 },
        SectorMomentum { name: "Oil & Gas".to_string(), momentum: -0.5, relative_strength: 0.93, rank: 7 },
        SectorMomentum { name: "Realty".to_string(), momentum: -1.1, relative_strength: 0.90, rank: 8 },
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    sectors: &[SectorMomentum],
    cfg: &SectorRotationConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    fill_background(&root, cfg.theme)?;

    let n = sectors.len();
    let max_mom = sectors
        .iter()
        .map(|s| s.momentum)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_mom = sectors
        .iter()
        .map(|s| s.momentum)
        .fold(f64::INFINITY, f64::min);
    let mom_range = (max_mom - min_mom).max(1.0);

    let (chart_area, rs_area) = if cfg.show_relative_strength {
        let split = root.split_horizontally((70).percent());
        (split.0, Some(split.1))
    } else {
        (root.clone(), None)
    };

    let mut chart = ChartBuilder::on(&chart_area)
        .caption(
            format!("{} — {}P Lookback", cfg.title, cfg.lookback_periods),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(120)
        .build_cartesian_2d(
            0.0..n as f64,
            (min_mom - mom_range * 0.1)..(max_mom + mom_range * 0.1),
        )
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .x_labels(n)
        .x_label_formatter(&|i| {
            sectors
                .get(*i as usize)
                .map(|s| s.name.clone())
                .unwrap_or_default()
        })
        .x_label_offset(10)
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(LineSeries::new(
            vec![(0.0, 0.0), ((n - 1) as f64, 0.0)],
            cfg.theme.border().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    let bar_width = 0.6;
    for (i, sector) in sectors.iter().enumerate() {
        let color = if sector.momentum > 0.5 {
            cfg.theme.profit().mix(1.0)
        } else if sector.momentum < -0.5 {
            cfg.theme.loss().mix(1.0)
        } else {
            cfg.theme.border().mix(0.7)
        };

        chart
            .draw_series(std::iter::once(Rectangle::new(
                [
                    (i as f64 - bar_width / 2.0, 0.0),
                    (i as f64 + bar_width / 2.0, sector.momentum),
                ],
                color.mix(0.7).filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;

        let label_y = sector.momentum
            + if sector.momentum >= 0.0 {
                mom_range * 0.03
            } else {
                -mom_range * 0.05
            };
        chart
            .draw_series(std::iter::once(Text::new(
                format!("{:+.1}%", sector.momentum),
                (i as f64, label_y),
                (LABEL_FONT, 11).into_font().color(&cfg.theme.text()),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;

        chart
            .draw_series(std::iter::once(Text::new(
                format!("#{}", sector.rank),
                (i as f64, min_mom - mom_range * 0.06),
                (LABEL_FONT, 9).into_font().color(&cfg.theme.accent()),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    if let Some(rs_area) = rs_area {
        let max_rs = sectors
            .iter()
            .map(|s| s.relative_strength)
            .fold(f64::NEG_INFINITY, f64::max);
        let min_rs = sectors
            .iter()
            .map(|s| s.relative_strength)
            .fold(f64::INFINITY, f64::min);
        let rs_range = (max_rs - min_rs).max(0.01);

        let mut rs_chart = ChartBuilder::on(&rs_area)
            .caption(
                "Relative Strength",
                (TITLE_FONT, 16).into_font().color(&cfg.theme.text()),
            )
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(60)
            .build_cartesian_2d(
                0.0..n as f64,
                (min_rs - rs_range * 0.1)..(max_rs + rs_range * 0.1),
            )
            .map_err(|e| BtError::Render(e.to_string()))?;

        rs_chart
            .configure_mesh()
            .label_style((LABEL_FONT, 10).into_font().color(&cfg.theme.text()))
            .axis_style(&cfg.theme.border())
            .light_line_style(cfg.theme.border().mix(0.3))
            .x_labels(n)
            .x_label_formatter(&|i| {
                sectors
                    .get(*i as usize)
                    .map(|s| s.name.clone())
                    .unwrap_or_default()
            })
            .draw()
            .map_err(|e| BtError::Render(e.to_string()))?;

        rs_chart
            .draw_series(LineSeries::new(
                vec![(0.0, 1.0), ((n - 1) as f64, 1.0)],
                cfg.theme.border().mix(0.5).stroke_width(1),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;

        rs_chart
            .draw_series(LineSeries::new(
                sectors
                    .iter()
                    .enumerate()
                    .map(|(i, s)| (i as f64, s.relative_strength)),
                cfg.theme.accent().stroke_width(2),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;

        for (i, sector) in sectors.iter().enumerate() {
            let color = if sector.relative_strength > 1.0 {
                cfg.theme.profit()
            } else {
                cfg.theme.loss()
            };
            rs_chart
                .draw_series(std::iter::once(Circle::new(
                    (i as f64, sector.relative_strength),
                    4,
                    color.filled(),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(sectors: &[SectorMomentum], cfg: &SectorRotationConfig, path: &str) -> Result<()> {
    render(png_root(path)?, sectors, cfg)
}

pub fn render_svg(sectors: &[SectorMomentum], cfg: &SectorRotationConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, sectors, cfg)
}

pub fn render_sample_png(cfg: &SectorRotationConfig, path: &str) -> Result<()> {
    let data = sample_sectors();
    render_png(&data, cfg, path)
}

pub fn render_sample_svg(cfg: &SectorRotationConfig, path: &str) -> Result<()> {
    let data = sample_sectors();
    render_svg(&data, cfg, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let data = sample_sectors();
        let cfg = SectorRotationConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_sector_rotation.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&data, &cfg, &path).unwrap();
    }
}
