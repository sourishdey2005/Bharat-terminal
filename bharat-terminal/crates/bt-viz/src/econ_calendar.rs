// crates/bt-viz/src/econ_calendar.rs
// Author: Sourish Dey

//! Economic calendar timeline. Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

/// Economic event data.
#[derive(Debug, Clone)]
pub struct EconEvent {
    pub date: String,
    pub time: String,
    pub event: String,
    pub country: String,
    pub impact: Impact,
    pub actual: Option<String>,
    pub forecast: Option<String>,
    pub previous: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Impact {
    High,
    Medium,
    Low,
}

/// Sample economic calendar data.
fn sample_econ() -> Vec<EconEvent> {
    vec![
        EconEvent { date: "28 Sep".to_string(), time: "10:00".to_string(), event: "GDP Growth Rate".to_string(), country: "IN".to_string(), impact: Impact::High, actual: Some("7.8%".to_string()), forecast: Some("7.5%".to_string()), previous: Some("7.2%".to_string()) },
        EconEvent { date: "28 Sep".to_string(), time: "14:30".to_string(), event: "CPI Inflation".to_string(), country: "IN".to_string(), impact: Impact::High, actual: Some("3.6%".to_string()), forecast: Some("3.8%".to_string()), previous: Some("3.5%".to_string()) },
        EconEvent { date: "29 Sep".to_string(), time: "09:00".to_string(), event: "Interest Rate Decision".to_string(), country: "US".to_string(), impact: Impact::High, actual: Some("5.25%".to_string()), forecast: Some("5.25%".to_string()), previous: Some("5.50%".to_string()) },
        EconEvent { date: "29 Sep".to_string(), time: "18:00".to_string(), event: "FOMC Statement".to_string(), country: "US".to_string(), impact: Impact::High, actual: None, forecast: None, previous: None },
        EconEvent { date: "30 Sep".to_string(), time: "08:30".to_string(), event: "IIP Data".to_string(), country: "IN".to_string(), impact: Impact::Medium, actual: Some("5.2%".to_string()), forecast: Some("4.8%".to_string()), previous: Some("4.5%".to_string()) },
        EconEvent { date: "30 Sep".to_string(), time: "11:00".to_string(), event: "Trade Balance".to_string(), country: "IN".to_string(), impact: Impact::Medium, actual: Some("-$24B".to_string()), forecast: Some("-$22B".to_string()), previous: Some("-$20B".to_string()) },
        EconEvent { date: "01 Oct".to_string(), time: "07:30".to_string(), event: "PMI Manufacturing".to_string(), country: "IN".to_string(), impact: Impact::Medium, actual: Some("57.5".to_string()), forecast: Some("57.0".to_string()), previous: Some("56.8".to_string()) },
        EconEvent { date: "01 Oct".to_string(), time: "12:00".to_string(), event: "Non-Farm Payrolls".to_string(), country: "US".to_string(), impact: Impact::High, actual: Some("254K".to_string()), forecast: Some("180K".to_string()), previous: Some("187K".to_string()) },
        EconEvent { date: "02 Oct".to_string(), time: "09:00".to_string(), event: "Unemployment Rate".to_string(), country: "US".to_string(), impact: Impact::High, actual: Some("4.1%".to_string()), forecast: Some("4.2%".to_string()), previous: Some("4.2%".to_string()) },
        EconEvent { date: "02 Oct".to_string(), time: "10:30".to_string(), event: "WPI Inflation".to_string(), country: "IN".to_string(), impact: Impact::Low, actual: Some("2.1%".to_string()), forecast: Some("2.0%".to_string()), previous: Some("1.8%".to_string()) },
        EconEvent { date: "03 Oct".to_string(), time: "08:00".to_string(), event: "Consumer Confidence".to_string(), country: "US".to_string(), impact: Impact::Medium, actual: Some("103.0".to_string()), forecast: Some("100.0".to_string()), previous: Some("98.5".to_string()) },
        EconEvent { date: "03 Oct".to_string(), time: "11:30".to_string(), event: "Fiscal Deficit".to_string(), country: "IN".to_string(), impact: Impact::Low, actual: Some("4.8%".to_string()), forecast: Some("5.0%".to_string()), previous: Some("4.9%".to_string()) },
        EconEvent { date: "04 Oct".to_string(), time: "09:30".to_string(), event: "Services PMI".to_string(), country: "IN".to_string(), impact: Impact::Medium, actual: Some("60.2".to_string()), forecast: Some("59.5".to_string()), previous: Some("59.0".to_string()) },
        EconEvent { date: "04 Oct".to_string(), time: "13:00".to_string(), event: "Fed Chair Speech".to_string(), country: "US".to_string(), impact: Impact::High, actual: None, forecast: None, previous: None },
        EconEvent { date: "05 Oct".to_string(), time: "10:00".to_string(), event: "Industrial Production".to_string(), country: "IN".to_string(), impact: Impact::Low, actual: Some("4.2%".to_string()), forecast: Some("4.0%".to_string()), previous: Some("3.8%".to_string()) },
    ]
}

#[derive(Debug, Clone)]
pub struct EconCalendarConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for EconCalendarConfig {
    fn default() -> Self {
        Self {
            title: "Economic Calendar".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl EconCalendarConfig {
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

fn impact_color(theme: Theme, impact: Impact) -> RGBAColor {
    match impact {
        Impact::High => theme.loss().mix(0.7),
        Impact::Medium => theme.accent().mix(0.7),
        Impact::Low => theme.info().mix(0.5),
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    data: &[EconEvent],
    cfg: &EconCalendarConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if data.is_empty() {
        return Err(BtError::EmptySeries("econ data".into()));
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
        let color = impact_color(cfg.theme, event.impact);

        // Impact indicator bar
        root.draw(&Rectangle::new(
            [(0, y as i32), (6, (y + row_h) as i32)],
            color.filled(),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        // Date/time
        root.draw(&Text::new(
            format!("{} {}", event.date, event.time),
            (15, (y + row_h / 2.0) as i32),
            (LABEL_FONT, 11)
                .into_font()
                .color(&cfg.theme.text().mix(0.6)),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        // Event name
        root.draw(&Text::new(
            event.event.as_str(),
            (130, (y + row_h / 2.0) as i32),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        // Country
        root.draw(&Text::new(
            event.country.as_str(),
            (350, (y + row_h / 2.0) as i32),
            (LABEL_FONT, 10)
                .into_font()
                .color(&cfg.theme.text().mix(0.5)),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        // Impact label
        let impact_str = match event.impact {
            Impact::High => "HIGH",
            Impact::Medium => "MED",
            Impact::Low => "LOW",
        };
        root.draw(&Text::new(
            impact_str.to_string(),
            (400, (y + row_h / 2.0) as i32),
            (LABEL_FONT, 10).into_font().color(&color),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        // Data columns
        let actual = event.actual.clone().unwrap_or_else(|| "-".to_string());
        let forecast = event.forecast.clone().unwrap_or_else(|| "-".to_string());
        let previous = event.previous.clone().unwrap_or_else(|| "-".to_string());

        root.draw(&Text::new(
            format!("A: {}", actual),
            (460, (y + row_h / 2.0) as i32),
            (LABEL_FONT, 11).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        root.draw(&Text::new(
            format!("F: {}", forecast),
            (560, (y + row_h / 2.0) as i32),
            (LABEL_FONT, 11)
                .into_font()
                .color(&cfg.theme.text().mix(0.6)),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        root.draw(&Text::new(
            format!("P: {}", previous),
            (660, (y + row_h / 2.0) as i32),
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

pub fn render_png(data: &[EconEvent], cfg: &EconCalendarConfig, path: &str) -> Result<()> {
    render(png_root(path)?, data, cfg)
}

pub fn render_svg(data: &[EconEvent], cfg: &EconCalendarConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, data, cfg)
}

pub fn render_sample_png(cfg: &EconCalendarConfig, path: &str) -> Result<()> {
    let data = sample_econ();
    render_png(&data, cfg, path)
}

pub fn render_sample_svg(cfg: &EconCalendarConfig, path: &str) -> Result<()> {
    let data = sample_econ();
    render_svg(&data, cfg, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let data = sample_econ();
        let cfg = EconCalendarConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_econ_calendar.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&data, &cfg, &path).unwrap();
    }
}
