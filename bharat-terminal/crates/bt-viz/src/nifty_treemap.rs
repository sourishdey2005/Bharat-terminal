// crates/bt-viz/src/nifty_treemap.rs
// Author: Sourish Dey

//! Tier 7 #66 — Nifty 50 Constituent Treemap (constituents × cap × change).
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

/// Nifty 50 constituent data for treemap.
#[derive(Debug, Clone)]
pub struct TreemapConstituent {
    pub symbol: String,
    pub name: String,
    pub sector: String,
    pub market_cap: f64,    // in crores
    pub change_pct: f64,    // daily change %
    pub weight_pct: f64,    // index weight %
}

/// Configuration for the treemap.
#[derive(Debug, Clone)]
pub struct NiftyTreemapConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for NiftyTreemapConfig {
    fn default() -> Self {
        Self {
            title: "Nifty 50 Treemap".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl NiftyTreemapConfig {
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
}

/// Sample Nifty 50 data (replace with real data from bt-data)
fn sample_nifty50() -> Vec<TreemapConstituent> {
    vec![
        TreemapConstituent { symbol: "RELIANCE.NS".to_string(), name: "Reliance Industries".to_string(), sector: "Oil & Gas".to_string(), market_cap: 1700000.0, change_pct: 1.2, weight_pct: 11.5 },
        TreemapConstituent { symbol: "HDFCBANK.NS".to_string(), name: "HDFC Bank".to_string(), sector: "Banking".to_string(), market_cap: 1100000.0, change_pct: 0.8, weight_pct: 9.2 },
        TreemapConstituent { symbol: "ICICIBANK.NS".to_string(), name: "ICICI Bank".to_string(), sector: "Banking".to_string(), market_cap: 750000.0, change_pct: 1.5, weight_pct: 7.8 },
        TreemapConstituent { symbol: "INFY.NS".to_string(), name: "Infosys".to_string(), sector: "IT".to_string(), market_cap: 650000.0, change_pct: -0.5, weight_pct: 6.5 },
        TreemapConstituent { symbol: "TCS.NS".to_string(), name: "TCS".to_string(), sector: "IT".to_string(), market_cap: 1300000.0, change_pct: 0.3, weight_pct: 6.2 },
        TreemapConstituent { symbol: "BHARTIARTL.NS".to_string(), name: "Bharti Airtel".to_string(), sector: "Telecom".to_string(), market_cap: 550000.0, change_pct: 2.1, weight_pct: 4.1 },
        TreemapConstituent { symbol: "ITC.NS".to_string(), name: "ITC".to_string(), sector: "FMCG".to_string(), market_cap: 500000.0, change_pct: -0.2, weight_pct: 3.8 },
        TreemapConstituent { symbol: "LT.NS".to_string(), name: "Larsen & Toubro".to_string(), sector: "Construction".to_string(), market_cap: 450000.0, change_pct: 1.8, weight_pct: 3.5 },
        TreemapConstituent { symbol: "SBIN.NS".to_string(), name: "State Bank of India".to_string(), sector: "Banking".to_string(), market_cap: 550000.0, change_pct: 2.5, weight_pct: 3.2 },
        TreemapConstituent { symbol: "KOTAKBANK.NS".to_string(), name: "Kotak Mahindra Bank".to_string(), sector: "Banking".to_string(), market_cap: 380000.0, change_pct: 0.9, weight_pct: 2.9 },
        TreemapConstituent { symbol: "HINDUNILVR.NS".to_string(), name: "Hindustan Unilever".to_string(), sector: "FMCG".to_string(), market_cap: 580000.0, change_pct: -0.8, weight_pct: 2.8 },
        TreemapConstituent { symbol: "BAJFINANCE.NS".to_string(), name: "Bajaj Finance".to_string(), sector: "Financial Services".to_string(), market_cap: 420000.0, change_pct: 1.1, weight_pct: 2.5 },
        TreemapConstituent { symbol: "ASIANPAINT.NS".to_string(), name: "Asian Paints".to_string(), sector: "Consumer".to_string(), market_cap: 280000.0, change_pct: -0.3, weight_pct: 1.8 },
        TreemapConstituent { symbol: "MARUTI.NS".to_string(), name: "Maruti Suzuki".to_string(), sector: "Auto".to_string(), market_cap: 320000.0, change_pct: 1.4, weight_pct: 1.7 },
        TreemapConstituent { symbol: "SUNPHARMA.NS".to_string(), name: "Sun Pharma".to_string(), sector: "Pharma".to_string(), market_cap: 290000.0, change_pct: -1.2, weight_pct: 1.6 },
        TreemapConstituent { symbol: "TITAN.NS".to_string(), name: "Titan Company".to_string(), sector: "Consumer".to_string(), market_cap: 260000.0, change_pct: 0.7, weight_pct: 1.5 },
        TreemapConstituent { symbol: "ULTRACEMCO.NS".to_string(), name: "UltraTech Cement".to_string(), sector: "Cement".to_string(), market_cap: 240000.0, change_pct: 1.0, weight_pct: 1.4 },
        TreemapConstituent { symbol: "NESTLEIND.NS".to_string(), name: "Nestle India".to_string(), sector: "FMCG".to_string(), market_cap: 220000.0, change_pct: -0.4, weight_pct: 1.3 },
        TreemapConstituent { symbol: "WIPRO.NS".to_string(), name: "Wipro".to_string(), sector: "IT".to_string(), market_cap: 250000.0, change_pct: -0.6, weight_pct: 1.2 },
        TreemapConstituent { symbol: "HCLTECH.NS".to_string(), name: "HCL Technologies".to_string(), sector: "IT".to_string(), market_cap: 320000.0, change_pct: 0.5, weight_pct: 1.1 },
        TreemapConstituent { symbol: "AXISBANK.NS".to_string(), name: "Axis Bank".to_string(), sector: "Banking".to_string(), market_cap: 280000.0, change_pct: 1.3, weight_pct: 1.0 },
        TreemapConstituent { symbol: "TATAMOTORS.NS".to_string(), name: "Tata Motors".to_string(), sector: "Auto".to_string(), market_cap: 250000.0, change_pct: 2.0, weight_pct: 1.0 },
        TreemapConstituent { symbol: "TATASTEEL.NS".to_string(), name: "Tata Steel".to_string(), sector: "Metals".to_string(), market_cap: 180000.0, change_pct: 1.5, weight_pct: 0.9 },
        TreemapConstituent { symbol: "ADANIENT.NS".to_string(), name: "Adani Enterprises".to_string(), sector: "Conglomerate".to_string(), market_cap: 280000.0, change_pct: 3.2, weight_pct: 0.9 },
        TreemapConstituent { symbol: "ADANIPORTS.NS".to_string(), name: "Adani Ports".to_string(), sector: "Infrastructure".to_string(), market_cap: 190000.0, change_pct: 1.8, weight_pct: 0.8 },
        TreemapConstituent { symbol: "BAJAJ-AUTO.NS".to_string(), name: "Bajaj Auto".to_string(), sector: "Auto".to_string(), market_cap: 160000.0, change_pct: 0.6, weight_pct: 0.8 },
        TreemapConstituent { symbol: "COALINDIA.NS".to_string(), name: "Coal India".to_string(), sector: "Metals & Mining".to_string(), market_cap: 140000.0, change_pct: -0.5, weight_pct: 0.7 },
        TreemapConstituent { symbol: "DIVISLAB.NS".to_string(), name: "Divi's Laboratories".to_string(), sector: "Pharma".to_string(), market_cap: 130000.0, change_pct: -0.8, weight_pct: 0.7 },
        TreemapConstituent { symbol: "DRREDDY.NS".to_string(), name: "Dr Reddy's Labs".to_string(), sector: "Pharma".to_string(), market_cap: 120000.0, change_pct: 0.2, weight_pct: 0.6 },
        TreemapConstituent { symbol: "EICHERMOT.NS".to_string(), name: "Eicher Motors".to_string(), sector: "Auto".to_string(), market_cap: 110000.0, change_pct: 1.1, weight_pct: 0.6 },
        TreemapConstituent { symbol: "GRASIM.NS".to_string(), name: "Grasim Industries".to_string(), sector: "Cement".to_string(), market_cap: 100000.0, change_pct: 0.9, weight_pct: 0.5 },
        TreemapConstituent { symbol: "HEROMOTOCO.NS".to_string(), name: "Hero MotoCorp".to_string(), sector: "Auto".to_string(), market_cap: 95000.0, change_pct: -0.3, weight_pct: 0.5 },
        TreemapConstituent { symbol: "HINDALCO.NS".to_string(), name: "Hindalco".to_string(), sector: "Metals".to_string(), market_cap: 115000.0, change_pct: 1.7, weight_pct: 0.5 },
        TreemapConstituent { symbol: "INDUSINDBK.NS".to_string(), name: "IndusInd Bank".to_string(), sector: "Banking".to_string(), market_cap: 105000.0, change_pct: 1.4, weight_pct: 0.5 },
        TreemapConstituent { symbol: "JSWSTEEL.NS".to_string(), name: "JSW Steel".to_string(), sector: "Metals".to_string(), market_cap: 120000.0, change_pct: 1.2, weight_pct: 0.5 },
        TreemapConstituent { symbol: "M&M.NS".to_string(), name: "Mahindra & Mahindra".to_string(), sector: "Auto".to_string(), market_cap: 180000.0, change_pct: 0.8, weight_pct: 0.5 },
        TreemapConstituent { symbol: "NTPC.NS".to_string(), name: "NTPC".to_string(), sector: "Power".to_string(), market_cap: 170000.0, change_pct: 0.4, weight_pct: 0.5 },
        TreemapConstituent { symbol: "ONGC.NS".to_string(), name: "ONGC".to_string(), sector: "Oil & Gas".to_string(), market_cap: 160000.0, change_pct: 1.0, weight_pct: 0.5 },
        TreemapConstituent { symbol: "POWERGRID.NS".to_string(), name: "Power Grid".to_string(), sector: "Power".to_string(), market_cap: 150000.0, change_pct: 0.3, weight_pct: 0.5 },
        TreemapConstituent { symbol: "TECHM.NS".to_string(), name: "Tech Mahindra".to_string(), sector: "IT".to_string(), market_cap: 110000.0, change_pct: -0.4, weight_pct: 0.4 },
        TreemapConstituent { symbol: "TATACONSUM.NS".to_string(), name: "Tata Consumer".to_string(), sector: "FMCG".to_string(), market_cap: 95000.0, change_pct: -0.1, weight_pct: 0.4 },
        TreemapConstituent { symbol: "UPL.NS".to_string(), name: "UPL".to_string(), sector: "Chemicals".to_string(), market_cap: 90000.0, change_pct: 0.5, weight_pct: 0.4 },
    ]
}

/// Render a squarified treemap.
fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    constituents: &[TreemapConstituent],
    cfg: &NiftyTreemapConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    fill_background(&root, cfg.theme)?;

    let (w, h) = root.dim_in_pixel();
    let canvas_w = w as f64;
    let canvas_h = h as f64 - 50.0; // Reserve space for title/footer

    // Total weight
    let total_weight: f64 = constituents.iter().map(|c| c.weight_pct).sum();

    // Normalize weights to canvas area
    let total_area = canvas_w * canvas_h;
    let mut rectangles = Vec::new();
    let mut x = 0.0;
    let mut y = 0.0;
    let mut row_height = 0.0_f64;

    // Simple row-based layout (squarified would be better but more complex)
    for c in constituents {
        let area = total_area * (c.weight_pct / total_weight);
        let rect_w = (area / 20.0).sqrt() * 20.0; // Aspect ratio ~1:1
        let rect_h = area / rect_w;

        if x + rect_w > canvas_w {
            x = 0.0;
            y += row_height + 5.0;
            row_height = 0.0;
        }

        rectangles.push((x, y, rect_w, rect_h, c.clone()));
        x += rect_w + 2.0;
        row_height = row_height.max(rect_h);
    }

    // Draw rectangles
    for (rx, ry, rw, rh, c) in rectangles {
        let color = if c.change_pct >= 0.0 {
            cfg.theme.profit().mix(0.7 + 0.3 * (c.change_pct / 5.0).min(1.0))
        } else {
            cfg.theme.loss().mix(0.7 + 0.3 * (c.change_pct.abs() / 5.0).min(1.0))
        };

        root.draw(&Rectangle::new(
            [(rx as i32, ry as i32 + 40), ((rx + rw) as i32, (ry + rh) as i32 + 40)],
            color.filled(),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        // Border
        root.draw(&Rectangle::new(
            [(rx as i32, ry as i32 + 40), ((rx + rw) as i32, (ry + rh) as i32 + 40)],
            cfg.theme.border().stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        // Label
        let font_size = (rw.min(rh) / 8.0).max(8.0) as u32;
        if rw > 40.0 && rh > 20.0 {
            root.draw(&Text::new(
                format!("{}\n{:+.1}%", c.symbol, c.change_pct),
                (rx as i32 + 4, ry as i32 + 44),
                (LABEL_FONT, font_size).into_font().color(&cfg.theme.text()),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    // Title
    root.draw(&Text::new(
        format!("{} — NIFTY 50 ({} constituents)", cfg.title, constituents.len()),
        (w as i32 / 2, 30),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(constituents: &[TreemapConstituent], cfg: &NiftyTreemapConfig, path: &str) -> Result<()> {
    render(png_root(path)?, constituents, cfg)
}

pub fn render_svg(constituents: &[TreemapConstituent], cfg: &NiftyTreemapConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, constituents, cfg)
}

/// Render with sample data.
pub fn render_sample_png(cfg: &NiftyTreemapConfig, path: &str) -> Result<()> {
    let data = sample_nifty50();
    render_png(&data, cfg, path)
}

pub fn render_sample_svg(cfg: &NiftyTreemapConfig, path: &str) -> Result<()> {
    let data = sample_nifty50();
    render_svg(&data, cfg, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_constructs() {
        let cfg = NiftyTreemapConfig::new();
        let _ = cfg;
    }
}
