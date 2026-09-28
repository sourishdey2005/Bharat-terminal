// crates/bt-viz/src/market_profile.rs
// Author: Sourish Dey

//! TPO market profile letters. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct MarketProfileConfig {
    pub title: String,
    pub theme: Theme,
    pub num_bins: usize,
}

impl Default for MarketProfileConfig {
    fn default() -> Self {
        Self {
            title: "Market Profile".to_string(),
            theme: Theme::Dark,
            num_bins: 30,
        }
    }
}

impl MarketProfileConfig {
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

    pub fn num_bins(mut self, bins: usize) -> Self {
        self.num_bins = bins.max(10);
        self
    }
}

fn compute_tpo_letters(
    series: &OhlcvSeries,
    num_bins: usize,
) -> (Vec<Vec<char>>, f64, f64, Vec<f64>) {
    let high = series
        .candles
        .iter()
        .map(|c| c.high)
        .fold(f64::NEG_INFINITY, f64::max);
    let low = series
        .candles
        .iter()
        .map(|c| c.low)
        .fold(f64::INFINITY, f64::min);

    let bin_size = (high - low) / num_bins as f64;
    let letters = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut bins: Vec<Vec<char>> = vec![Vec::new(); num_bins];

    for (i, candle) in series.candles.iter().enumerate() {
        let letter = letters.chars().nth(i % 26).unwrap_or('Z');
        let low_bin = ((candle.low - low) / bin_size).floor() as isize;
        let high_bin = ((candle.high - low) / bin_size).floor() as isize;

        for bin_idx in low_bin..=high_bin {
            let idx = bin_idx.clamp(0, num_bins as isize - 1) as usize;
            bins[idx].push(letter);
        }
    }

    let prices: Vec<f64> = (0..num_bins)
        .map(|i| low + i as f64 * bin_size + bin_size / 2.0)
        .collect();

    (bins, low, high, prices)
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &MarketProfileConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let (bins, low, high, prices) = compute_tpo_letters(series, cfg.num_bins);
    let bin_size = (high - low) / cfg.num_bins as f64;
    let max_count = bins.iter().map(|b| b.len()).max().unwrap_or(1);

    let profile_area = root.clone();
    let mut chart = ChartBuilder::on(&profile_area)
        .caption(
            format!("{} — {}", cfg.title, series.symbol),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..(max_count as f64 * 1.2), low..high)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    for (i, bin) in bins.iter().enumerate() {
        if bin.is_empty() {
            continue;
        }
        let price = prices[i];
        let count = bin.len() as f64;
        let intensity = count / max_count as f64;

        chart
            .draw_series(std::iter::once(Rectangle::new(
                [
                    (0.0, price - bin_size / 2.0),
                    (count, price + bin_size / 2.0),
                ],
                cfg.theme.accent().mix(intensity * 0.7).filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;

        for (j, ch) in bin.iter().enumerate() {
            chart
                .draw_series(std::iter::once(Text::new(
                    ch.to_string(),
                    (j as f64 + 0.5, price),
                    (LABEL_FONT, 8).into_font().color(&cfg.theme.text()),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &MarketProfileConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &MarketProfileConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = MarketProfileConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_market_profile.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
