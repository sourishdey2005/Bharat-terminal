// crates/bt-viz/src/sector_performance.rs
// Author: Sourish Dey

//! Tier 7 #69 â€” Sector Performance Bar Chart (ranked NSE sectors).
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

/// Sector performance data.
#[derive(Debug, Clone)]
pub struct SectorData {
    pub name: String,
    pub change_pct: f64,   // period change %
    pub market_cap: f64,   // total sector market cap (crores)
    pub num_stocks: usize, // number of stocks in sector
    pub advance: usize,    // advancing stocks
    pub decline: usize,    // declining stocks
}

/// Configuration for sector performance chart.
#[derive(Debug, Clone)]
pub struct SectorPerformanceConfig {
    pub title: String,
    pub theme: Theme,
    pub show_advance_decline: bool,
}

impl Default for SectorPerformanceConfig {
    fn default() -> Self {
        Self {
            title: "Sector Performance".to_string(),
            theme: Theme::Dark,
            show_advance_decline: true,
        }
    }
}

impl SectorPerformanceConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn show_advance_decline(mut self, show: bool) -> Self {
        self.show_advance_decline = show;
        self
    }
}

/// Sample NSE sector data (replace with real data from bt-data)
fn sample_sectors() -> Vec<SectorData> {
    vec![
        SectorData {
            name: "IT".to_string(),
            change_pct: -0.5,
            market_cap: 2500000.0,
            num_stocks: 10,
            advance: 3,
            decline: 7,
        },
        SectorData {
            name: "Banking".to_string(),
            change_pct: 1.8,
            market_cap: 3200000.0,
            num_stocks: 12,
            advance: 10,
            decline: 2,
        },
        SectorData {
            name: "Oil & Gas".to_string(),
            change_pct: 1.2,
            market_cap: 2000000.0,
            num_stocks: 4,
            advance: 3,
            decline: 1,
        },
        SectorData {
            name: "FMCG".to_string(),
            change_pct: -0.3,
            market_cap: 1500000.0,
            num_stocks: 8,
            advance: 2,
            decline: 6,
        },
        SectorData {
            name: "Auto".to_string(),
            change_pct: 1.4,
            market_cap: 1000000.0,
            num_stocks: 8,
            advance: 6,
            decline: 2,
        },
        SectorData {
            name: "Pharma".to_string(),
            change_pct: -0.8,
            market_cap: 800000.0,
            num_stocks: 6,
            advance: 1,
            decline: 5,
        },
        SectorData {
            name: "Metals".to_string(),
            change_pct: 1.5,
            market_cap: 600000.0,
            num_stocks: 5,
            advance: 4,
            decline: 1,
        },
        SectorData {
            name: "Consumer Durables".to_string(),
            change_pct: 0.7,
            market_cap: 400000.0,
            num_stocks: 3,
            advance: 2,
            decline: 1,
        },
        SectorData {
            name: "Cement".to_string(),
            change_pct: 1.0,
            market_cap: 350000.0,
            num_stocks: 4,
            advance: 3,
            decline: 1,
        },
        SectorData {
            name: "Telecom".to_string(),
            change_pct: 2.1,
            market_cap: 600000.0,
            num_stocks: 2,
            advance: 2,
            decline: 0,
        },
        SectorData {
            name: "Power".to_string(),
            change_pct: 0.4,
            market_cap: 500000.0,
            num_stocks: 4,
            advance: 2,
            decline: 2,
        },
        SectorData {
            name: "Financial Services".to_string(),
            change_pct: 1.1,
            market_cap: 700000.0,
            num_stocks: 5,
            advance: 4,
            decline: 1,
        },
        SectorData {
            name: "Chemicals".to_string(),
            change_pct: 0.5,
            market_cap: 300000.0,
            num_stocks: 3,
            advance: 2,
            decline: 1,
        },
        SectorData {
            name: "Construction".to_string(),
            change_pct: 1.8,
            market_cap: 450000.0,
            num_stocks: 2,
            advance: 2,
            decline: 0,
        },
        SectorData {
            name: "Realty".to_string(),
            change_pct: 0.9,
            market_cap: 200000.0,
            num_stocks: 3,
            advance: 2,
            decline: 1,
        },
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    sectors: &[SectorData],
    cfg: &SectorPerformanceConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    fill_background(&root, cfg.theme)?;

    // Sort by change_pct descending
    let mut sorted = sectors.to_vec();
    sorted.sort_by(|a, b| b.change_pct.partial_cmp(&a.change_pct).unwrap());

    let n = sorted.len();
    let (w, h) = root.dim_in_pixel();
    let canvas_w = w as f64;
    let canvas_h = h as f64 - 80.0;

    // Chart area
    let mut chart = ChartBuilder::on(&root)
        .caption(
            cfg.title.clone(),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(60)
        .y_label_area_size(140)
        .build_cartesian_2d(
            0.0..n as f64,
            sorted
                .iter()
                .map(|s| s.change_pct)
                .fold(f64::INFINITY, f64::min)
                - 0.5
                ..sorted
                    .iter()
                    .map(|s| s.change_pct)
                    .fold(f64::NEG_INFINITY, f64::max)
                    + 1.0,
        )
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .x_labels(n)
        .x_label_formatter(&|i| {
            sorted
                .get(*i as usize)
                .map(|s| s.name.clone())
                .unwrap_or_default()
        })
        .x_label_offset(10)
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Zero line
    chart
        .draw_series(LineSeries::new(
            vec![(0.0, 0.0), ((n - 1) as f64, 0.0)],
            cfg.theme.border().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Bars
    let bar_width = 0.6;
    for (i, sector) in sorted.iter().enumerate() {
        let color = if sector.change_pct >= 0.0 {
            cfg.theme.profit()
        } else {
            cfg.theme.loss()
        };

        chart
            .draw_series(std::iter::once(Rectangle::new(
                [
                    (i as f64 - bar_width / 2.0, 0.0),
                    (i as f64 + bar_width / 2.0, sector.change_pct),
                ],
                color.mix(0.7).filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;

        // Value label on bar
        let label_y = sector.change_pct
            + if sector.change_pct >= 0.0 {
                0.15
            } else {
                -0.25
            };
        chart
            .draw_series(std::iter::once(Text::new(
                format!("{:+.1}%", sector.change_pct),
                (i as f64, label_y),
                (LABEL_FONT, 11).into_font().color(&cfg.theme.text()),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    // Advance/Decline labels at bottom
    if cfg.show_advance_decline {
        for (i, sector) in sorted.iter().enumerate() {
            let ad_text = format!("A:{}/D:{}", sector.advance, sector.decline);
            chart
                .draw_series(std::iter::once(Text::new(
                    ad_text,
                    (
                        i as f64,
                        sorted
                            .iter()
                            .map(|s| s.change_pct)
                            .fold(f64::INFINITY, f64::min)
                            - 0.3,
                    ),
                    (LABEL_FONT, 9)
                        .into_font()
                        .color(&cfg.theme.text().mix(0.7)),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    // Market cap reference line (secondary y-axis would be better but complex)
    // Instead, add as tooltip-style text
    for (i, sector) in sorted.iter().enumerate() {
        if i < 5 {
            // Top 5 sectors
            let mc_text = format!("â‚¹{:.0}k Cr", sector.market_cap / 1000.0);
            chart
                .draw_series(std::iter::once(Text::new(
                    mc_text,
                    (
                        i as f64,
                        sorted
                            .iter()
                            .map(|s| s.change_pct)
                            .fold(f64::NEG_INFINITY, f64::max)
                            + 0.3,
                    ),
                    (LABEL_FONT, 9).into_font().color(&cfg.theme.accent()),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(sectors: &[SectorData], cfg: &SectorPerformanceConfig, path: &str) -> Result<()> {
    render(png_root(path)?, sectors, cfg)
}

pub fn render_svg(sectors: &[SectorData], cfg: &SectorPerformanceConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, sectors, cfg)
}

pub fn render_sample_png(cfg: &SectorPerformanceConfig, path: &str) -> Result<()> {
    let data = sample_sectors();
    render_png(&data, cfg, path)
}

pub fn render_sample_svg(cfg: &SectorPerformanceConfig, path: &str) -> Result<()> {
    let data = sample_sectors();
    render_svg(&data, cfg, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_constructs() {
        let cfg = SectorPerformanceConfig::new();
        let _ = cfg;
    }
}
