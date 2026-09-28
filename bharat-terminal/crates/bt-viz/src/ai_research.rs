// crates/bt-viz/src/ai_research.rs
// Author: Sourish Dey

//! AI research summarization (key insights, sentiment, confidence). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct AiResearchConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for AiResearchConfig {
    fn default() -> Self {
        Self {
            title: "AI Research Summary".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl AiResearchConfig {
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

fn sample_insights() -> Vec<(String, String, f64, String)> {
    vec![
        (
            "Rate cycle peaking".to_string(),
            "Macro".to_string(),
            0.85,
            "High".to_string(),
        ),
        (
            "Earnings recovery".to_string(),
            "Fundamental".to_string(),
            0.78,
            "High".to_string(),
        ),
        (
            "FII inflows returning".to_string(),
            "Flow".to_string(),
            0.72,
            "Medium".to_string(),
        ),
        (
            "INR stability".to_string(),
            "FX".to_string(),
            0.65,
            "Medium".to_string(),
        ),
        (
            "Capex cycle intact".to_string(),
            "Sector".to_string(),
            0.80,
            "High".to_string(),
        ),
        (
            "Valuation rich".to_string(),
            "Technical".to_string(),
            0.55,
            "Medium".to_string(),
        ),
        (
            "Monsoon normal".to_string(),
            "Agri".to_string(),
            0.90,
            "High".to_string(),
        ),
        (
            "Global risk-on".to_string(),
            "Global".to_string(),
            0.60,
            "Medium".to_string(),
        ),
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &AiResearchConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let data = sample_insights();
    let (w, h) = root.dim_in_pixel();

    root.draw(&Text::new(
        format!("{} — {}", cfg.title, series.symbol),
        (10, 10),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let row_h = (h as i32 - 100) / (data.len() as i32 + 1);
    let col_px = [10, 300, 500, 700, 900];

    for (x, label) in col_px
        .iter()
        .zip(["Insight", "Category", "Confidence", "Signal", "Score"].iter())
    {
        root.draw(&Text::new(
            label.to_string(),
            (*x, 45),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.accent()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    for (i, (insight, category, conf, signal)) in data.iter().enumerate() {
        let y = 45 + (i as i32 + 1) * row_h;
        root.draw(&Text::new(
            insight.clone(),
            (col_px[0], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            category.clone(),
            (col_px[1], y + row_h / 2 - 6),
            (LABEL_FONT, 12)
                .into_font()
                .color(&cfg.theme.text().mix(0.8)),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        let conf_color = if *conf >= 0.75 {
            cfg.theme.profit()
        } else if *conf >= 0.6 {
            cfg.theme.accent()
        } else {
            cfg.theme.loss()
        };
        root.draw(&Text::new(
            format!("{:.0}%", conf * 100.0),
            (col_px[2], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&conf_color),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        let sig_color = if signal == "High" {
            cfg.theme.profit()
        } else {
            cfg.theme.accent()
        };
        root.draw(&Text::new(
            signal.clone(),
            (col_px[3], y + row_h / 2 - 6),
            (LABEL_FONT, 12).into_font().color(&sig_color),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        let bar_w = (conf * 200.0) as i32;
        root.draw(&Rectangle::new(
            [
                (col_px[4], y + row_h / 2 - 8),
                (col_px[4] + bar_w, y + row_h / 2 + 4),
            ],
            conf_color.filled(),
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

pub fn render_png(series: &OhlcvSeries, cfg: &AiResearchConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &AiResearchConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("AI", 100, 1, 100.0);
        let cfg = AiResearchConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_ai_research.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
