// crates/bt-viz/src/vall_dealer.rs
// Author: Sourish Dey

//! VALL: Dealer bid/ask comparison.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct DealerQuote {
    pub dealer: String,
    pub bid: f64,
    pub ask: f64,
}

impl DealerQuote {
    pub fn new(dealer: impl Into<String>, bid: f64, ask: f64) -> Self {
        Self {
            dealer: dealer.into(),
            bid,
            ask,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VallDealerConfig {
    pub title: String,
    pub theme: Theme,
    pub show_spread: bool,
}

impl Default for VallDealerConfig {
    fn default() -> Self {
        Self {
            title: "Dealer Bid/Ask Comparison".to_string(),
            theme: Theme::Dark,
            show_spread: true,
        }
    }
}

impl VallDealerConfig {
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
    pub fn show_spread(mut self, s: bool) -> Self {
        self.show_spread = s;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    quotes: &[DealerQuote],
    cfg: &VallDealerConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if quotes.is_empty() {
        return Err(BtError::EmptySeries("dealer quotes".into()));
    }
    fill_background(&root, cfg.theme)?;

    let mut p_min = f64::INFINITY;
    let mut p_max = f64::NEG_INFINITY;
    for q in quotes {
        p_min = p_min.min(q.bid);
        p_max = p_max.max(q.ask);
    }
    let pad = (p_max - p_min).max(0.01) * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            cfg.title.clone(),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0..quotes.len(), (p_min - pad)..(p_max + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .x_labels(quotes.len())
        .x_label_formatter(&|x| quotes.get(*x).map(|q| q.dealer.clone()).unwrap_or_default())
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    for (i, q) in quotes.iter().enumerate() {
        chart
            .draw_series(std::iter::once(Circle::new(
                (i, q.bid),
                5,
                cfg.theme.profit().filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
        chart
            .draw_series(std::iter::once(Circle::new(
                (i, q.ask),
                5,
                cfg.theme.loss().filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![(i, q.bid), (i, q.ask)],
                cfg.theme.border().stroke_width(1),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;

        if cfg.show_spread {
            let spread = q.ask - q.bid;
            root.draw(&Text::new(
                format!("{:.3}", spread),
                (i as i32 * 80 + 10, 80),
                (LABEL_FONT, 10).into_font().color(&cfg.theme.accent()),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(quotes: &[DealerQuote], cfg: &VallDealerConfig, path: &str) -> Result<()> {
    render(png_root(path)?, quotes, cfg)
}

pub fn render_svg(quotes: &[DealerQuote], cfg: &VallDealerConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, quotes, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let quotes: Vec<DealerQuote> = (0..6)
            .map(|i| {
                DealerQuote::new(
                    format!("D{}", i),
                    100.0 + i as f64 * 0.1,
                    100.0 + i as f64 * 0.1 + 0.05,
                )
            })
            .collect();
        let cfg = VallDealerConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_vall_dealer.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&quotes, &cfg, &path).unwrap();
    }
}
