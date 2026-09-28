// crates/bt-viz/src/hp_history.rs
// Author: Sourish Dey

//! HP: Historical price & volume table.
//! Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct HpHistoryConfig {
    pub title: String,
    pub theme: Theme,
    pub rows: usize,
    pub columns: usize,
}

impl Default for HpHistoryConfig {
    fn default() -> Self {
        Self {
            title: "Historical Price & Volume".to_string(),
            theme: Theme::Dark,
            rows: 20,
            columns: 6,
        }
    }
}

impl HpHistoryConfig {
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
    pub fn rows(mut self, r: usize) -> Self {
        self.rows = r.max(5).min(100);
        self
    }
    pub fn columns(mut self, c: usize) -> Self {
        self.columns = c.max(3).min(10);
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &HpHistoryConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let (title_area, body_area) = root.split_vertically(50);
    title_area
        .draw(&Text::new(
            format!("{} — {}", cfg.title, series.symbol),
            (15, 10),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    let (w, h) = body_area.dim_in_pixel();
    let headers = ["Date", "Open", "High", "Low", "Close", "Volume"];
    let col_w = w as i32 / cfg.columns as i32;
    let row_h = (h as usize - 30) / cfg.rows;

    for (ci, header) in headers.iter().enumerate().take(cfg.columns) {
        body_area
            .draw(&Text::new(
                header.to_string(),
                (ci as i32 * col_w + 5, 5),
                (LABEL_FONT, 12).into_font().color(&cfg.theme.accent()),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    let start_idx = series.candles.len().saturating_sub(cfg.rows);
    for (ri, ci) in series.candles[start_idx..]
        .iter()
        .enumerate()
        .take(cfg.rows)
    {
        let c = series.candles[start_idx + ri];
        let y = 25 + ri * row_h;
        let row_color = if c.is_bullish() {
            cfg.theme.profit().mix(0.1)
        } else {
            cfg.theme.loss().mix(0.1)
        };
        body_area
            .draw(&Rectangle::new(
                [(0, y as i32), (w as i32, (y + row_h) as i32)],
                row_color.filled(),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;

        let vals = [
            format!("{:.0}", c.t),
            format!("{:.2}", c.open),
            format!("{:.2}", c.high),
            format!("{:.2}", c.low),
            format!("{:.2}", c.close),
            format!("{:.0}", c.volume),
        ];
        for (ci, val) in vals.iter().enumerate().take(cfg.columns) {
            body_area
                .draw(&Text::new(
                    val.clone(),
                    (ci as i32 * col_w + 5, (y + 2) as i32),
                    (LABEL_FONT, 11).into_font().color(&cfg.theme.text()),
                ))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &HpHistoryConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &HpHistoryConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = HpHistoryConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_hp_history.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
