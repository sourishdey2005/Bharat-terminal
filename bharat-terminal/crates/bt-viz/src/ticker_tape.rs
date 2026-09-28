// crates/bt-viz/src/ticker_tape.rs
// Author: Sourish Dey

//! Scrolling ticker tape visualization. Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

/// Ticker tape entry.
#[derive(Debug, Clone)]
pub struct TickerEntry {
    pub symbol: String,
    pub price: f64,
    pub change_pct: f64,
    pub volume: f64,
}

/// Sample ticker tape data.
fn sample_tape() -> Vec<TickerEntry> {
    vec![
        TickerEntry {
            symbol: "RELIANCE".to_string(),
            price: 2985.50,
            change_pct: 1.25,
            volume: 15_200_000.0,
        },
        TickerEntry {
            symbol: "HDFCBANK".to_string(),
            price: 1720.30,
            change_pct: 0.82,
            volume: 12_800_000.0,
        },
        TickerEntry {
            symbol: "INFY".to_string(),
            price: 1545.75,
            change_pct: -0.45,
            volume: 8_500_000.0,
        },
        TickerEntry {
            symbol: "TCS".to_string(),
            price: 3890.00,
            change_pct: 0.35,
            volume: 3_200_000.0,
        },
        TickerEntry {
            symbol: "ICICIBANK".to_string(),
            price: 1085.60,
            change_pct: 1.55,
            volume: 18_400_000.0,
        },
        TickerEntry {
            symbol: "SBIN".to_string(),
            price: 625.40,
            change_pct: 2.10,
            volume: 25_600_000.0,
        },
        TickerEntry {
            symbol: "BHARTIARTL".to_string(),
            price: 1580.25,
            change_pct: 0.95,
            volume: 9_800_000.0,
        },
        TickerEntry {
            symbol: "ITC".to_string(),
            price: 435.80,
            change_pct: -0.22,
            volume: 14_200_000.0,
        },
        TickerEntry {
            symbol: "LT".to_string(),
            price: 3650.00,
            change_pct: 1.80,
            volume: 4_500_000.0,
        },
        TickerEntry {
            symbol: "KOTAKBANK".to_string(),
            price: 1785.90,
            change_pct: 0.65,
            volume: 7_800_000.0,
        },
        TickerEntry {
            symbol: "HINDUNILVR".to_string(),
            price: 2450.30,
            change_pct: -0.85,
            volume: 2_100_000.0,
        },
        TickerEntry {
            symbol: "BAJFINANCE".to_string(),
            price: 7150.00,
            change_pct: 1.10,
            volume: 1_800_000.0,
        },
        TickerEntry {
            symbol: "MARUTI".to_string(),
            price: 12450.00,
            change_pct: 0.45,
            volume: 1_200_000.0,
        },
        TickerEntry {
            symbol: "SUNPHARMA".to_string(),
            price: 1890.75,
            change_pct: -1.20,
            volume: 5_600_000.0,
        },
        TickerEntry {
            symbol: "TITAN".to_string(),
            price: 3450.00,
            change_pct: 0.70,
            volume: 3_400_000.0,
        },
        TickerEntry {
            symbol: "ULTRACEMCO".to_string(),
            price: 10850.00,
            change_pct: 1.05,
            volume: 800_000.0,
        },
        TickerEntry {
            symbol: "NTPC".to_string(),
            price: 385.50,
            change_pct: 0.30,
            volume: 11_500_000.0,
        },
        TickerEntry {
            symbol: "TATAMOTORS".to_string(),
            price: 985.25,
            change_pct: 2.50,
            volume: 22_300_000.0,
        },
        TickerEntry {
            symbol: "WIPRO".to_string(),
            price: 485.60,
            change_pct: -0.60,
            volume: 6_700_000.0,
        },
        TickerEntry {
            symbol: "ADANIENT".to_string(),
            price: 3150.00,
            change_pct: 3.20,
            volume: 16_800_000.0,
        },
    ]
}

#[derive(Debug, Clone)]
pub struct TickerTapeConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for TickerTapeConfig {
    fn default() -> Self {
        Self {
            title: "Ticker Tape".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl TickerTapeConfig {
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

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    data: &[TickerEntry],
    cfg: &TickerTapeConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if data.is_empty() {
        return Err(BtError::EmptySeries("ticker data".into()));
    }
    fill_background(&root, cfg.theme)?;

    let (w, h) = root.dim_in_pixel();
    let top_pad = 50;
    let bottom_pad = 50;
    let chart_h = h as f64 - top_pad as f64 - bottom_pad as f64;

    root.draw(&Text::new(
        cfg.title.as_str(),
        (w as i32 / 2, 20),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let n = data.len();
    let row_h = chart_h / n as f64;

    for (i, entry) in data.iter().enumerate() {
        let y = top_pad as f64 + i as f64 * row_h;
        let is_up = entry.change_pct >= 0.0;
        let color = if is_up {
            cfg.theme.profit()
        } else {
            cfg.theme.loss()
        };

        // Background stripe
        if i % 2 == 0 {
            root.draw(&Rectangle::new(
                [(0, y as i32), (w as i32, (y + row_h) as i32)],
                cfg.theme.border().mix(0.3).filled(),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
        }

        // Symbol
        root.draw(&Text::new(
            entry.symbol.as_str(),
            (10, (y + row_h / 2.0) as i32),
            (LABEL_FONT, 13).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        // Price
        root.draw(&Text::new(
            format!("{:.2}", entry.price),
            (160, (y + row_h / 2.0) as i32),
            (LABEL_FONT, 13).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        // Change
        let arrow = if is_up { "+" } else { "-" };
        root.draw(&Text::new(
            format!("{}{:.2}%", arrow, entry.change_pct.abs()),
            (300, (y + row_h / 2.0) as i32),
            (LABEL_FONT, 13).into_font().color(&color),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        // Volume bar
        let max_vol = data.iter().map(|e| e.volume).fold(0.0_f64, f64::max);
        let bar_w = (entry.volume / max_vol) * 200.0;
        root.draw(&Rectangle::new(
            [
                (450, (y + row_h * 0.25) as i32),
                ((450.0 + bar_w) as i32, (y + row_h * 0.75) as i32),
            ],
            color.mix(0.5).filled(),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        // Volume label
        let vol_str = if entry.volume >= 1_000_000.0 {
            format!("{:.1}M", entry.volume / 1_000_000.0)
        } else {
            format!("{:.0}K", entry.volume / 1_000.0)
        };
        root.draw(&Text::new(
            vol_str,
            (670, (y + row_h / 2.0) as i32),
            (LABEL_FONT, 11)
                .into_font()
                .color(&cfg.theme.text().mix(0.6)),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(data: &[TickerEntry], cfg: &TickerTapeConfig, path: &str) -> Result<()> {
    render(png_root(path)?, data, cfg)
}

pub fn render_svg(data: &[TickerEntry], cfg: &TickerTapeConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, data, cfg)
}

pub fn render_sample_png(cfg: &TickerTapeConfig, path: &str) -> Result<()> {
    let data = sample_tape();
    render_png(&data, cfg, path)
}

pub fn render_sample_svg(cfg: &TickerTapeConfig, path: &str) -> Result<()> {
    let data = sample_tape();
    render_svg(&data, cfg, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let data = sample_tape();
        let cfg = TickerTapeConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_ticker_tape.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&data, &cfg, &path).unwrap();
    }
}
