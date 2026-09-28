// crates/bt-viz/src/algo_feed.rs
// Author: Sourish Dey

//! Algo-ready data feed status (latency, uptime, symbols). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct AlgoFeedConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for AlgoFeedConfig {
    fn default() -> Self {
        Self {
            title: "Algo Feed Status".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl AlgoFeedConfig {
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

fn sample_feeds() -> Vec<(String, f64, f64, String)> {
    vec![
        ("NSE Equity".to_string(), 12.5, 99.98, "Live".to_string()),
        ("NSE FO".to_string(), 15.2, 99.95, "Live".to_string()),
        ("BSE Equity".to_string(), 18.3, 99.92, "Live".to_string()),
        ("MCX".to_string(), 22.1, 99.88, "Live".to_string()),
        ("Currency".to_string(), 14.8, 99.97, "Live".to_string()),
        ("Corp Bonds".to_string(), 45.0, 99.50, "Delayed".to_string()),
        ("News Feed".to_string(), 120.0, 98.50, "Delayed".to_string()),
        (
            "Fundamentals".to_string(),
            250.0,
            97.00,
            "Batch".to_string(),
        ),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &AlgoFeedConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_feeds();
    let (w, h) = root.dim_in_pixel();

    root.draw(&Text::new(
        format!("{} — {}", cfg.title, series.symbol),
        (10, 10),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let row_h = (h as i32 - 100) / (data.len() as i32 + 1);
    let col_px = [10, 250, 450, 650, 850];

    for (x, label) in col_px
        .iter()
        .zip(["Feed", "Latency (ms)", "Uptime %", "Status", "Health"].iter())
    {
        root.draw(&Text::new(
            label.to_string(),
            (*x, 45),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.accent()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    for (i, (name, latency, uptime, status)) in data.iter().enumerate() {
        let y = 45 + (i as i32 + 1) * row_h;
        root.draw(&Text::new(
            name.clone(),
            (col_px[0], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        let lat_color = if *latency < 20.0 {
            cfg.theme.profit()
        } else if *latency < 50.0 {
            cfg.theme.accent()
        } else {
            cfg.theme.loss()
        };
        root.draw(&Text::new(
            format!("{:.1}", latency),
            (col_px[1], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&lat_color),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        let up_color = if *uptime >= 99.9 {
            cfg.theme.profit()
        } else if *uptime >= 99.0 {
            cfg.theme.accent()
        } else {
            cfg.theme.loss()
        };
        root.draw(&Text::new(
            format!("{:.2}", uptime),
            (col_px[2], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&up_color),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        let status_color = if status == "Live" {
            cfg.theme.profit()
        } else if status == "Delayed" {
            cfg.theme.accent()
        } else {
            cfg.theme.info()
        };
        root.draw(&Text::new(
            status.clone(),
            (col_px[3], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&status_color),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        let health = if *uptime >= 99.9 && *latency < 20.0 {
            "Excellent"
        } else if *uptime >= 99.0 {
            "Good"
        } else {
            "Fair"
        };
        root.draw(&Text::new(
            health.to_string(),
            (col_px[4], y + row_h / 2 - 6),
            (LABEL_FONT, 12)
                .into_font()
                .color(&cfg.theme.text().mix(0.7)),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
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

pub fn render_png(series: &OhlcvSeries, cfg: &AlgoFeedConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &AlgoFeedConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("ALGO", 100, 1, 100.0);
        let cfg = AlgoFeedConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_algo_feed.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
