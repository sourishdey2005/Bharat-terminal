// crates/bt-viz/src/sensex_heatmap.rs
// Author: Sourish Dey

//! Tier 7 #67 — Sensex Heatmap (30 stocks grid heatmap).
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

/// Sensex constituent data for heatmap.
#[derive(Debug, Clone)]
pub struct HeatmapStock {
    pub symbol: String,
    pub name: String,
    pub sector: String,
    pub change_pct: f64, // daily change %
    pub volume: f64,
    pub market_cap: f64,
}

/// Configuration for the heatmap.
#[derive(Debug, Clone)]
pub struct SensexHeatmapConfig {
    pub title: String,
    pub theme: Theme,
    pub cols: usize,
}

impl Default for SensexHeatmapConfig {
    fn default() -> Self {
        Self {
            title: "Sensex Heatmap".to_string(),
            theme: Theme::Dark,
            cols: 6,
        }
    }
}

impl SensexHeatmapConfig {
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

    pub fn cols(mut self, cols: usize) -> Self {
        self.cols = cols.max(2);
        self
    }
}

/// Sample Sensex 30 data (replace with real data from bt-data)
fn sample_sensex30() -> Vec<HeatmapStock> {
    vec![
        HeatmapStock {
            symbol: "RELIANCE.BO".to_string(),
            name: "Reliance".to_string(),
            sector: "Oil & Gas".to_string(),
            change_pct: 1.2,
            volume: 1500000.0,
            market_cap: 1700000.0,
        },
        HeatmapStock {
            symbol: "HDFCBANK.BO".to_string(),
            name: "HDFC Bank".to_string(),
            sector: "Banking".to_string(),
            change_pct: 0.8,
            volume: 2000000.0,
            market_cap: 1100000.0,
        },
        HeatmapStock {
            symbol: "ICICIBANK.BO".to_string(),
            name: "ICICI Bank".to_string(),
            sector: "Banking".to_string(),
            change_pct: 1.5,
            volume: 1800000.0,
            market_cap: 750000.0,
        },
        HeatmapStock {
            symbol: "INFY.BO".to_string(),
            name: "Infosys".to_string(),
            sector: "IT".to_string(),
            change_pct: -0.5,
            volume: 1200000.0,
            market_cap: 650000.0,
        },
        HeatmapStock {
            symbol: "TCS.BO".to_string(),
            name: "TCS".to_string(),
            sector: "IT".to_string(),
            change_pct: 0.3,
            volume: 900000.0,
            market_cap: 1300000.0,
        },
        HeatmapStock {
            symbol: "BHARTIARTL.BO".to_string(),
            name: "Bharti Airtel".to_string(),
            sector: "Telecom".to_string(),
            change_pct: 2.1,
            volume: 3000000.0,
            market_cap: 550000.0,
        },
        HeatmapStock {
            symbol: "ITC.BO".to_string(),
            name: "ITC".to_string(),
            sector: "FMCG".to_string(),
            change_pct: -0.2,
            volume: 2500000.0,
            market_cap: 500000.0,
        },
        HeatmapStock {
            symbol: "LT.BO".to_string(),
            name: "L&T".to_string(),
            sector: "Construction".to_string(),
            change_pct: 1.8,
            volume: 800000.0,
            market_cap: 450000.0,
        },
        HeatmapStock {
            symbol: "SBIN.BO".to_string(),
            name: "SBI".to_string(),
            sector: "Banking".to_string(),
            change_pct: 2.5,
            volume: 4000000.0,
            market_cap: 550000.0,
        },
        HeatmapStock {
            symbol: "KOTAKBANK.BO".to_string(),
            name: "Kotak Bank".to_string(),
            sector: "Banking".to_string(),
            change_pct: 0.9,
            volume: 1100000.0,
            market_cap: 380000.0,
        },
        HeatmapStock {
            symbol: "HINDUNILVR.BO".to_string(),
            name: "HUL".to_string(),
            sector: "FMCG".to_string(),
            change_pct: -0.8,
            volume: 900000.0,
            market_cap: 580000.0,
        },
        HeatmapStock {
            symbol: "BAJFINANCE.BO".to_string(),
            name: "Bajaj Fin".to_string(),
            sector: "Financial".to_string(),
            change_pct: 1.1,
            volume: 700000.0,
            market_cap: 420000.0,
        },
        HeatmapStock {
            symbol: "ASIANPAINT.BO".to_string(),
            name: "Asian Paints".to_string(),
            sector: "Consumer".to_string(),
            change_pct: -0.3,
            volume: 500000.0,
            market_cap: 280000.0,
        },
        HeatmapStock {
            symbol: "MARUTI.BO".to_string(),
            name: "Maruti".to_string(),
            sector: "Auto".to_string(),
            change_pct: 1.4,
            volume: 600000.0,
            market_cap: 320000.0,
        },
        HeatmapStock {
            symbol: "SUNPHARMA.BO".to_string(),
            name: "Sun Pharma".to_string(),
            sector: "Pharma".to_string(),
            change_pct: -1.2,
            volume: 800000.0,
            market_cap: 290000.0,
        },
        HeatmapStock {
            symbol: "TITAN.BO".to_string(),
            name: "Titan".to_string(),
            sector: "Consumer".to_string(),
            change_pct: 0.7,
            volume: 600000.0,
            market_cap: 260000.0,
        },
        HeatmapStock {
            symbol: "ULTRACEMCO.BO".to_string(),
            name: "UltraTech".to_string(),
            sector: "Cement".to_string(),
            change_pct: 1.0,
            volume: 400000.0,
            market_cap: 240000.0,
        },
        HeatmapStock {
            symbol: "NESTLEIND.BO".to_string(),
            name: "Nestle".to_string(),
            sector: "FMCG".to_string(),
            change_pct: -0.4,
            volume: 300000.0,
            market_cap: 220000.0,
        },
        HeatmapStock {
            symbol: "WIPRO.BO".to_string(),
            name: "Wipro".to_string(),
            sector: "IT".to_string(),
            change_pct: -0.6,
            volume: 1000000.0,
            market_cap: 250000.0,
        },
        HeatmapStock {
            symbol: "HCLTECH.BO".to_string(),
            name: "HCL Tech".to_string(),
            sector: "IT".to_string(),
            change_pct: 0.5,
            volume: 900000.0,
            market_cap: 320000.0,
        },
        HeatmapStock {
            symbol: "AXISBANK.BO".to_string(),
            name: "Axis Bank".to_string(),
            sector: "Banking".to_string(),
            change_pct: 1.3,
            volume: 1500000.0,
            market_cap: 280000.0,
        },
        HeatmapStock {
            symbol: "TATAMOTORS.BO".to_string(),
            name: "Tata Motors".to_string(),
            sector: "Auto".to_string(),
            change_pct: 2.0,
            volume: 2000000.0,
            market_cap: 250000.0,
        },
        HeatmapStock {
            symbol: "TATASTEEL.BO".to_string(),
            name: "Tata Steel".to_string(),
            sector: "Metals".to_string(),
            change_pct: 1.5,
            volume: 1200000.0,
            market_cap: 180000.0,
        },
        HeatmapStock {
            symbol: "ADANIENT.BO".to_string(),
            name: "Adani Ent".to_string(),
            sector: "Conglomerate".to_string(),
            change_pct: 3.2,
            volume: 1500000.0,
            market_cap: 280000.0,
        },
        HeatmapStock {
            symbol: "ADANIPORTS.BO".to_string(),
            name: "Adani Ports".to_string(),
            sector: "Infra".to_string(),
            change_pct: 1.8,
            volume: 800000.0,
            market_cap: 190000.0,
        },
        HeatmapStock {
            symbol: "BAJAJ-AUTO.BO".to_string(),
            name: "Bajaj Auto".to_string(),
            sector: "Auto".to_string(),
            change_pct: 0.6,
            volume: 400000.0,
            market_cap: 160000.0,
        },
        HeatmapStock {
            symbol: "COALINDIA.BO".to_string(),
            name: "Coal India".to_string(),
            sector: "Mining".to_string(),
            change_pct: -0.5,
            volume: 2000000.0,
            market_cap: 140000.0,
        },
        HeatmapStock {
            symbol: "NTPC.BO".to_string(),
            name: "NTPC".to_string(),
            sector: "Power".to_string(),
            change_pct: 0.4,
            volume: 1500000.0,
            market_cap: 170000.0,
        },
        HeatmapStock {
            symbol: "POWERGRID.BO".to_string(),
            name: "Power Grid".to_string(),
            sector: "Power".to_string(),
            change_pct: 0.3,
            volume: 1000000.0,
            market_cap: 150000.0,
        },
        HeatmapStock {
            symbol: "TECHM.BO".to_string(),
            name: "Tech M".to_string(),
            sector: "IT".to_string(),
            change_pct: -0.4,
            volume: 800000.0,
            market_cap: 110000.0,
        },
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    stocks: &[HeatmapStock],
    cfg: &SensexHeatmapConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    fill_background(&root, cfg.theme)?;

    let (w, h) = root.dim_in_pixel();
    let canvas_w = w as f64;
    let canvas_h = h as f64 - 80.0; // Title + footer

    let cols = cfg.cols;
    let rows = (stocks.len() + cols - 1) / cols;

    let cell_w = canvas_w / cols as f64;
    let cell_h = canvas_h / rows as f64;
    let padding = 4.0;

    // Find max absolute change for color scaling
    let max_abs_change = stocks
        .iter()
        .map(|s| s.change_pct.abs())
        .fold(0.0_f64, f64::max)
        .max(0.5);

    for (idx, stock) in stocks.iter().enumerate() {
        let col = idx % cols;
        let row = idx / cols;

        let x = col as f64 * cell_w + padding;
        let y = row as f64 * cell_h + padding + 40.0; // Title offset
        let cw = cell_w - 2.0 * padding;
        let ch = cell_h - 2.0 * padding;

        // Color based on change %
        let intensity = (stock.change_pct / max_abs_change).clamp(-1.0, 1.0);
        let color = if intensity >= 0.0 {
            cfg.theme.profit().mix(0.3 + 0.7 * intensity)
        } else {
            cfg.theme.loss().mix(0.3 + 0.7 * intensity.abs())
        };

        // Draw cell background
        root.draw(&Rectangle::new(
            [(x as i32, y as i32), ((x + cw) as i32, (y + ch) as i32)],
            color.filled(),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        // Border
        root.draw(&Rectangle::new(
            [(x as i32, y as i32), ((x + cw) as i32, (y + ch) as i32)],
            cfg.theme.border().stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        // Symbol
        root.draw(&Text::new(
            stock.symbol.replace(".BO", ""),
            (x as i32 + 4, y as i32 + 16),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        // Change %
        let change_str = format!("{:+.1}%", stock.change_pct);
        root.draw(&Text::new(
            change_str,
            (x as i32 + 4, y as i32 + 32),
            (LABEL_FONT, 14).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        // Sector (small)
        let sector_short = if stock.sector.len() > 12 {
            &stock.sector[..12]
        } else {
            &stock.sector
        };
        root.draw(&Text::new(
            sector_short,
            (x as i32 + 4, (y + ch - 16.0) as i32),
            (LABEL_FONT, 9)
                .into_font()
                .color(&cfg.theme.text().mix(0.7)),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    // Title
    root.draw(&Text::new(
        format!("{} — SENSEX 30 ({} stocks)", cfg.title, stocks.len()),
        (w as i32 / 2, 30),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    // Legend
    let legend_y = h as i32 - 30;
    // Green to red gradient
    for i in 0..100 {
        let intensity = (i as f64 / 99.0 - 0.5) * 2.0;
        let color = if intensity >= 0.0 {
            cfg.theme.profit().mix(0.3 + 0.7 * intensity)
        } else {
            cfg.theme.loss().mix(0.3 + 0.7 * intensity.abs())
        };
        root.draw(&Rectangle::new(
            [(50 + i * 10, legend_y), (60 + i * 10, legend_y + 15)],
            color.filled(),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }
    root.draw(&Text::new(
        "-5%".to_string(),
        (50, legend_y + 30),
        (LABEL_FONT, 10).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;
    root.draw(&Text::new(
        "0%".to_string(),
        (550, legend_y + 30),
        (LABEL_FONT, 10).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;
    root.draw(&Text::new(
        "+5%".to_string(),
        (1050, legend_y + 30),
        (LABEL_FONT, 10).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(stocks: &[HeatmapStock], cfg: &SensexHeatmapConfig, path: &str) -> Result<()> {
    render(png_root(path)?, stocks, cfg)
}

pub fn render_svg(stocks: &[HeatmapStock], cfg: &SensexHeatmapConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, stocks, cfg)
}

pub fn render_sample_png(cfg: &SensexHeatmapConfig, path: &str) -> Result<()> {
    let data = sample_sensex30();
    render_png(&data, cfg, path)
}

pub fn render_sample_svg(cfg: &SensexHeatmapConfig, path: &str) -> Result<()> {
    let data = sample_sensex30();
    render_svg(&data, cfg, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_constructs() {
        let cfg = SensexHeatmapConfig::new();
        let _ = cfg;
    }
}
