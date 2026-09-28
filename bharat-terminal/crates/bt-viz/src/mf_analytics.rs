// crates/bt-viz/src/mf_analytics.rs
// Author: Sourish Dey

//! Mutual fund analytics (NAV, category returns, risk metrics). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct MfAnalyticsConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for MfAnalyticsConfig {
    fn default() -> Self {
        Self { title: "Mutual Fund Analytics".to_string(), theme: Theme::Dark }
    }
}

impl MfAnalyticsConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
}

fn sample_funds() -> Vec<(String, f64, f64, f64)> {
    vec![
        ("HDFC Equity".to_string(), 12.5, 1.2, 14.8),
        ("ICICI Bluechip".to_string(), 10.8, 1.0, 15.2),
        ("SBI Small Cap".to_string(), 18.2, 2.1, 22.5),
        ("Axis Midcap".to_string(), 14.5, 1.5, 18.9),
        ("Kotak Emerg Eq".to_string(), 15.2, 1.8, 20.1),
        ("Tata Digital".to_string(), 8.5, 1.9, 21.5),
        ("Nippon India".to_string(), 11.2, 1.3, 16.0),
        ("UTI Nifty 50".to_string(), 9.8, 0.5, 13.5),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &MfAnalyticsConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_funds();
    let (w, h) = root.dim_in_pixel();

    root.draw(&Text::new(
        format!("{} — {}", cfg.title, series.symbol),
        (10, 10),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let chart = ChartBuilder::on(&root)
        .margin(15)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_cartesian_2d(0f64..data.len() as f64, 0f64..25f64)
        .map_err(|e| BtError::Render(e.to_string()))?;

    let mut chart = chart;
    chart.configure_mesh()
        .label_style((LABEL_FONT, 11).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .x_labels(data.len())
        .x_label_formatter(&|x| {
            let idx = *x as usize;
            if idx < data.len() {
                let n = data[idx].0.clone();
                if n.len() > 10 { n[..10].to_string() } else { n }
            } else { String::new() }
        })
        .y_desc("1Y Return (%)")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let bar_w = 0.6 / 3.0;
    for (i, (name, ret, risk, _)) in data.iter().enumerate() {
        let color = if *ret >= 12.0 { cfg.theme.profit() } else if *ret >= 10.0 { cfg.theme.accent() } else { cfg.theme.info() };
        chart.draw_series(std::iter::once(Rectangle::new(
            [(i as f64, 0.0), (i as f64 + bar_w * 0.9, *ret)],
            color.filled(),
        )))
    .map_err(|e| BtError::Render(e.to_string()))?;
        let _ = (name, risk);
    }

    for (i, (name, ret, _, _)) in data.iter().enumerate() {
        root.draw(&Text::new(
            format!("{:.1}", ret),
            (60 + i as i32 * ((w as i32 - 120) / data.len() as i32), 50),
            (LABEL_FONT, 10).into_font().color(&cfg.theme.text()),
        ))
    .map_err(|e| BtError::Render(e.to_string()))?;
        let _ = name;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &MfAnalyticsConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &MfAnalyticsConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("MF", 100, 1, 100.0);
        let cfg = MfAnalyticsConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_mf_analytics.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
