// crates/bt-viz/src/fo_chain.rs
// Author: Sourish Dey

//! NSE F&O options chain table. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::coord::Shift;
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct FoChainConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for FoChainConfig {
    fn default() -> Self {
        Self { title: "NSE F&O Options Chain".to_string(), theme: Theme::Dark }
    }
}

impl FoChainConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
}

#[derive(Debug, Clone)]
pub struct OptionRow {
    pub strike: f64,
    pub call_oi: f64,
    pub call_chng_oi: f64,
    pub call_iv: f64,
    pub call_ltp: f64,
    pub put_oi: f64,
    pub put_chng_oi: f64,
    pub put_iv: f64,
    pub put_ltp: f64,
}

fn sample_chain(symbol: &str, spot: f64) -> Vec<OptionRow> {
    let base = symbol.parse::<f64>().unwrap_or(spot);
    let atm = (base / 50.0).round() * 50.0;
    let mut rows = Vec::new();
    for i in -5i64..=5 {
        let k = atm + i as f64 * 50.0;
        let m = (k - atm).abs() / atm;
        rows.push(OptionRow {
            strike: k,
            call_oi: 100000.0 + (5 - i).abs() as f64 * 30000.0,
            call_chng_oi: if i < 0 { 5000.0 * i as f64 } else { 8000.0 * i as f64 },
            call_iv: 0.14 + 0.10 * m + 0.02 * i as f64 / 10.0,
            call_ltp: (base - k).max(2.0) * 0.9,
            put_oi: 90000.0 + (5 + i).abs() as f64 * 28000.0,
            put_chng_oi: if i > 0 { -6000.0 * i as f64 } else { -7000.0 * i as f64 },
            put_iv: 0.15 + 0.09 * m - 0.01 * i as f64 / 10.0,
            put_ltp: (k - base).max(2.0) * 0.9,
        });
    }
    rows
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &FoChainConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let (w, h) = root.dim_in_pixel();
    let spot = series.candles.last().map(|c| c.close).unwrap_or(24500.0);
    let rows = sample_chain(&series.symbol, spot);

    let cols = [
        ("STRIKE", 0.06), ("CALL OI", 0.17), ("CHNG OI", 0.27), ("IV%", 0.38),
        ("LTP", 0.47), ("PUT OI", 0.58), ("CHNG OI", 0.68), ("IV%", 0.78), ("LTP", 0.87),
    ];

    root.draw(&Text::new(
        format!("{} — {} (Spot {:.2})", cfg.title, series.symbol, spot),
        (10, 10),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let table_y = 40;
    let row_h = (h as i32 - table_y - 60) / (rows.len() as i32 + 1);

    for (label, fx) in cols {
        root.draw(&Text::new(
            label.to_string(),
            (10 + (w as f64 * fx) as i32, table_y),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.accent()),
        ))
    .map_err(|e| BtError::Render(e.to_string()))?;
    }

    for (r, row) in rows.iter().enumerate() {
        let y = table_y + (r as i32 + 1) * row_h;
        let draw_str = |root: &DrawingArea<DB, Shift>, txt: &str, x: i32, y: i32, color: RGBColor| {
            root.draw(&Text::new(txt.to_string(), (x, y), (LABEL_FONT, 11).into_font().color(&color)))
                .map_err(|e| BtError::Render(e.to_string()))
        };

        let x = 10 + (w as f64 * cols[0].1) as i32;
        draw_str(&root, &format!("{:.0}", row.strike), x, y + row_h / 2 - 6, cfg.theme.text())?;
        let x = 10 + (w as f64 * cols[1].1) as i32;
        let call_chg_color = if row.call_chng_oi >= 0.0 { cfg.theme.profit() } else { cfg.theme.loss() };
        draw_str(&root, &format!("{:+.0} / {:.1}%", row.call_chng_oi, row.call_iv * 100.0), x, y + row_h / 2 - 6, call_chg_color)?;
        let x = 10 + (w as f64 * cols[4].1) as i32;
        draw_str(&root, &format!("{:.2}", row.call_ltp), x, y + row_h / 2 - 6, cfg.theme.text())?;
        let x = 10 + (w as f64 * cols[5].1) as i32;
        let put_chg_color = if row.put_chng_oi >= 0.0 { cfg.theme.profit() } else { cfg.theme.loss() };
        draw_str(&root, &format!("{:+.0} / {:.1}%", row.put_chng_oi, row.put_iv * 100.0), x, y + row_h / 2 - 6, put_chg_color)?;
        let x = 10 + (w as f64 * cols[8].1) as i32;
        draw_str(&root, &format!("{:.2}", row.put_ltp), x, y + row_h / 2 - 6, cfg.theme.text())?;

        root.draw(&PathElement::new(
            vec![(0, y + row_h), (w as i32, y + row_h)],
            cfg.theme.border().stroke_width(1),
        ))
    .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &FoChainConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &FoChainConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("NIFTY", 100, 1, 100.0);
        let cfg = FoChainConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_fo_chain.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
