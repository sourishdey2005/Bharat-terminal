// crates/bt-viz/src/earnings_calendar.rs
// Author: Sourish Dey

//! Earnings calendar heatmap. Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

/// Earnings event data.
#[derive(Debug, Clone)]
pub struct EarningsEvent {
    pub symbol: String,
    pub date: String,
    pub eps_estimate: f64,
    pub eps_actual: Option<f64>,
    pub surprise_pct: Option<f64>,
}

/// Sample earnings calendar data.
fn sample_earnings() -> Vec<EarningsEvent> {
    vec![
        EarningsEvent { symbol: "TCS".to_string(), date: "10 Oct".to_string(), eps_estimate: 38.50, eps_actual: Some(39.20), surprise_pct: Some(1.82) },
        EarningsEvent { symbol: "INFY".to_string(), date: "10 Oct".to_string(), eps_estimate: 16.80, eps_actual: Some(16.50), surprise_pct: Some(-1.79) },
        EarningsEvent { symbol: "HDFCBANK".to_string(), date: "11 Oct".to_string(), eps_estimate: 21.20, eps_actual: Some(21.80), surprise_pct: Some(2.83) },
        EarningsEvent { symbol: "RELIANCE".to_string(), date: "11 Oct".to_string(), eps_estimate: 28.40, eps_actual: Some(27.90), surprise_pct: Some(-1.76) },
        EarningsEvent { symbol: "ICICIBANK".to_string(), date: "12 Oct".to_string(), eps_estimate: 10.50, eps_actual: Some(10.85), surprise_pct: Some(3.33) },
        EarningsEvent { symbol: "SBIN".to_string(), date: "12 Oct".to_string(), eps_estimate: 7.80, eps_actual: Some(7.65), surprise_pct: Some(-1.92) },
        EarningsEvent { symbol: "WIPRO".to_string(), date: "14 Oct".to_string(), eps_estimate: 5.20, eps_actual: Some(5.45), surprise_pct: Some(4.81) },
        EarningsEvent { symbol: "HCLTECH".to_string(), date: "14 Oct".to_string(), eps_estimate: 15.60, eps_actual: Some(15.30), surprise_pct: Some(-1.92) },
        EarningsEvent { symbol: "BHARTIARTL".to_string(), date: "15 Oct".to_string(), eps_estimate: 8.90, eps_actual: Some(9.25), surprise_pct: Some(3.93) },
        EarningsEvent { symbol: "ITC".to_string(), date: "15 Oct".to_string(), eps_estimate: 4.85, eps_actual: Some(4.90), surprise_pct: Some(1.03) },
        EarningsEvent { symbol: "LT".to_string(), date: "16 Oct".to_string(), eps_estimate: 32.40, eps_actual: Some(33.10), surprise_pct: Some(2.16) },
        EarningsEvent { symbol: "KOTAKBANK".to_string(), date: "16 Oct".to_string(), eps_estimate: 12.30, eps_actual: Some(12.10), surprise_pct: Some(-1.63) },
        EarningsEvent { symbol: "MARUTI".to_string(), date: "17 Oct".to_string(), eps_estimate: 42.80, eps_actual: Some(44.50), surprise_pct: Some(3.97) },
        EarningsEvent { symbol: "SUNPHARMA".to_string(), date: "17 Oct".to_string(), eps_estimate: 14.20, eps_actual: Some(13.80), surprise_pct: Some(-2.82) },
        EarningsEvent { symbol: "TATAMOTORS".to_string(), date: "18 Oct".to_string(), eps_estimate: 8.50, eps_actual: Some(9.10), surprise_pct: Some(7.06) },
        EarningsEvent { symbol: "ULTRACEMCO".to_string(), date: "18 Oct".to_string(), eps_estimate: 72.30, eps_actual: Some(70.50), surprise_pct: Some(-2.49) },
        EarningsEvent { symbol: "NTPC".to_string(), date: "21 Oct".to_string(), eps_estimate: 5.60, eps_actual: Some(5.75), surprise_pct: Some(2.68) },
        EarningsEvent { symbol: "TATASTEEL".to_string(), date: "21 Oct".to_string(), eps_estimate: 3.20, eps_actual: Some(2.95), surprise_pct: Some(-7.81) },
        EarningsEvent { symbol: "BAJFINANCE".to_string(), date: "22 Oct".to_string(), eps_estimate: 68.40, eps_actual: Some(71.20), surprise_pct: Some(4.09) },
        EarningsEvent { symbol: "ADANIENT".to_string(), date: "22 Oct".to_string(), eps_estimate: 22.50, eps_actual: Some(21.80), surprise_pct: Some(-3.11) },
    ]
}

#[derive(Debug, Clone)]
pub struct EarningsCalendarConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for EarningsCalendarConfig {
    fn default() -> Self {
        Self {
            title: "Earnings Calendar".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl EarningsCalendarConfig {
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

fn lerp(a: u8, b: u8, t: f64) -> u8 {
    (a as f64 + (b as f64 - a as f64) * t.clamp(0.0, 1.0)).round() as u8
}

fn surprise_color(theme: Theme, v: f64) -> RGBColor {
    let bg = theme.background();
    let target = if v >= 0.0 { theme.profit() } else { theme.loss() };
    let t = (v.abs() / 10.0).min(1.0);
    RGBColor(
        lerp(bg.0, target.0, t),
        lerp(bg.1, target.1, t),
        lerp(bg.2, target.2, t),
    )
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    data: &[EarningsEvent],
    cfg: &EarningsCalendarConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if data.is_empty() {
        return Err(BtError::EmptySeries("earnings data".into()));
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

    for (i, event) in data.iter().enumerate() {
        let y = top_pad as f64 + i as f64 * row_h;

        // Symbol
        root.draw(&Text::new(
            event.symbol.as_str(),
            (10, (y + row_h / 2.0) as i32),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        // Date
        root.draw(&Text::new(
            event.date.as_str(),
            (130, (y + row_h / 2.0) as i32),
            (LABEL_FONT, 11)
                .into_font()
                .color(&cfg.theme.text().mix(0.6)),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        // EPS estimate
        root.draw(&Text::new(
            format!("Est: {:.2}", event.eps_estimate),
            (220, (y + row_h / 2.0) as i32),
            (LABEL_FONT, 11)
                .into_font()
                .color(&cfg.theme.text().mix(0.6)),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        // Surprise cell
        if let (Some(actual), Some(surprise)) = (event.eps_actual, event.surprise_pct) {
            let color = surprise_color(cfg.theme, surprise);
            root.draw(&Rectangle::new(
                [(350, y as i32), (w as i32 - 20, (y + row_h) as i32)],
                color.filled(),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;

            root.draw(&Text::new(
                format!("Act: {:.2} ({:+.2}%)", actual, surprise),
                (360, (y + row_h / 2.0) as i32),
                (LABEL_FONT, 11).into_font().color(&cfg.theme.text()),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
        } else {
            root.draw(&Text::new(
                "Pending".to_string(),
                (360, (y + row_h / 2.0) as i32),
                (LABEL_FONT, 11)
                    .into_font()
                    .color(&cfg.theme.text().mix(0.4)),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(data: &[EarningsEvent], cfg: &EarningsCalendarConfig, path: &str) -> Result<()> {
    render(png_root(path)?, data, cfg)
}

pub fn render_svg(data: &[EarningsEvent], cfg: &EarningsCalendarConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, data, cfg)
}

pub fn render_sample_png(cfg: &EarningsCalendarConfig, path: &str) -> Result<()> {
    let data = sample_earnings();
    render_png(&data, cfg, path)
}

pub fn render_sample_svg(cfg: &EarningsCalendarConfig, path: &str) -> Result<()> {
    let data = sample_earnings();
    render_svg(&data, cfg, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let data = sample_earnings();
        let cfg = EarningsCalendarConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_earnings.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&data, &cfg, &path).unwrap();
    }
}
