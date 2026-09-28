// crates/bt-viz/src/market_breadth.rs
// Author: Sourish Dey

//! Market breadth indicators — advance/decline line, McClellan
//! oscillator, and breadth thrust detection. Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct MarketBreadthConfig {
    pub title: String,
    pub theme: Theme,
    pub show_mcclellan: bool,
    pub show_thrust: bool,
}

impl Default for MarketBreadthConfig {
    fn default() -> Self {
        Self {
            title: "Market Breadth".to_string(),
            theme: Theme::Dark,
            show_mcclellan: true,
            show_thrust: true,
        }
    }
}

impl MarketBreadthConfig {
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

    pub fn show_mcclellan(mut self, show: bool) -> Self {
        self.show_mcclellan = show;
        self
    }

    pub fn show_thrust(mut self, show: bool) -> Self {
        self.show_thrust = show;
        self
    }
}

#[derive(Debug, Clone)]
pub struct BreadthData {
    pub advances: usize,
    pub declines: usize,
    pub unchanged: usize,
    pub ad_ratio: f64,
    pub mcclellan: f64,
    pub thrust: bool,
}

fn sample_breadth() -> Vec<BreadthData> {
    vec![
        BreadthData {
            advances: 1200,
            declines: 800,
            unchanged: 100,
            ad_ratio: 1.5,
            mcclellan: 12.5,
            thrust: false,
        },
        BreadthData {
            advances: 1400,
            declines: 600,
            unchanged: 80,
            ad_ratio: 2.3,
            mcclellan: 25.1,
            thrust: true,
        },
        BreadthData {
            advances: 1100,
            declines: 900,
            unchanged: 120,
            ad_ratio: 1.2,
            mcclellan: -5.3,
            thrust: false,
        },
        BreadthData {
            advances: 1600,
            declines: 400,
            unchanged: 60,
            ad_ratio: 4.0,
            mcclellan: 45.8,
            thrust: true,
        },
        BreadthData {
            advances: 900,
            declines: 1100,
            unchanged: 150,
            ad_ratio: 0.8,
            mcclellan: -18.2,
            thrust: false,
        },
        BreadthData {
            advances: 1300,
            declines: 700,
            unchanged: 90,
            ad_ratio: 1.9,
            mcclellan: 8.7,
            thrust: false,
        },
        BreadthData {
            advances: 1500,
            declines: 500,
            unchanged: 70,
            ad_ratio: 3.0,
            mcclellan: 32.4,
            thrust: true,
        },
        BreadthData {
            advances: 1000,
            declines: 1000,
            unchanged: 130,
            ad_ratio: 1.0,
            mcclellan: -2.1,
            thrust: false,
        },
        BreadthData {
            advances: 1700,
            declines: 300,
            unchanged: 50,
            ad_ratio: 5.7,
            mcclellan: 58.3,
            thrust: true,
        },
        BreadthData {
            advances: 800,
            declines: 1200,
            unchanged: 140,
            ad_ratio: 0.7,
            mcclellan: -28.5,
            thrust: false,
        },
    ]
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    breadth: &[BreadthData],
    cfg: &MarketBreadthConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    fill_background(&root, cfg.theme)?;

    let n = breadth.len();
    let max_ad = breadth.iter().map(|b| b.advances).fold(0_usize, usize::max);
    let max_dec = breadth.iter().map(|b| b.declines).fold(0_usize, usize::max);
    let max_val = max_ad.max(max_dec) as f64;

    let (ad_area, lower_area) = root.split_vertically((50).percent());
    let (mcclellan_area, thrust_area) = if cfg.show_mcclellan && cfg.show_thrust {
        let split = lower_area.split_vertically((50).percent());
        (Some(split.0), Some(split.1))
    } else if cfg.show_mcclellan {
        (Some(lower_area), None)
    } else if cfg.show_thrust {
        (None, Some(lower_area))
    } else {
        (None, None)
    };

    let mut ad_chart = ChartBuilder::on(&ad_area)
        .caption(
            format!("{} — Advance/Decline", cfg.title),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..n as f64, 0.0..(max_val * 1.1))
        .map_err(|e| BtError::Render(e.to_string()))?;

    ad_chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let bar_width = 0.35;
    for (i, b) in breadth.iter().enumerate() {
        ad_chart
            .draw_series(std::iter::once(Rectangle::new(
                [(i as f64 - bar_width, 0.0), (i as f64, b.advances as f64)],
                cfg.theme.profit().mix(0.7).filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;

        ad_chart
            .draw_series(std::iter::once(Rectangle::new(
                [(i as f64, 0.0), (i as f64 + bar_width, b.declines as f64)],
                cfg.theme.loss().mix(0.7).filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    ad_chart
        .draw_series(LineSeries::new(
            vec![(0.0, 0.0), ((n - 1) as f64, 0.0)],
            cfg.theme.border().mix(0.3).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Advances")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                cfg.theme.profit().stroke_width(2),
            )
        });

    ad_chart
        .configure_series_labels()
        .background_style(cfg.theme.background().mix(0.9))
        .border_style(cfg.theme.border())
        .label_font((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    if let Some(mcc_area) = mcclellan_area {
        let max_mcc = breadth
            .iter()
            .map(|b| b.mcclellan)
            .fold(f64::NEG_INFINITY, f64::max);
        let min_mcc = breadth
            .iter()
            .map(|b| b.mcclellan)
            .fold(f64::INFINITY, f64::min);
        let mcc_range = (max_mcc - min_mcc).max(1.0);

        let mut mcc_chart = ChartBuilder::on(&mcc_area)
            .caption(
                "McClellan Oscillator",
                (TITLE_FONT, 16).into_font().color(&cfg.theme.text()),
            )
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(60)
            .build_cartesian_2d(
                0.0..n as f64,
                (min_mcc - mcc_range * 0.1)..(max_mcc + mcc_range * 0.1),
            )
            .map_err(|e| BtError::Render(e.to_string()))?;

        mcc_chart
            .configure_mesh()
            .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
            .axis_style(&cfg.theme.border())
            .light_line_style(cfg.theme.border().mix(0.3))
            .draw()
            .map_err(|e| BtError::Render(e.to_string()))?;

        mcc_chart
            .draw_series(LineSeries::new(
                vec![(0.0, 0.0), ((n - 1) as f64, 0.0)],
                cfg.theme.border().mix(0.5).stroke_width(1),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;

        mcc_chart
            .draw_series(LineSeries::new(
                breadth
                    .iter()
                    .enumerate()
                    .map(|(i, b)| (i as f64, b.mcclellan)),
                cfg.theme.accent().stroke_width(2),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?
            .label("McClellan")
            .legend(|(x, y)| {
                PathElement::new(
                    vec![(x, y), (x + 20, y)],
                    cfg.theme.accent().stroke_width(2),
                )
            });

        for (i, b) in breadth.iter().enumerate() {
            let color = if b.mcclellan > 0.0 {
                cfg.theme.profit().mix(0.3)
            } else {
                cfg.theme.loss().mix(0.3)
            };
            mcc_chart
                .draw_series(std::iter::once(Rectangle::new(
                    [
                        (i as f64 - 0.3, 0.0_f64.min(b.mcclellan)),
                        (i as f64 + 0.3, b.mcclellan.max(0.0)),
                    ],
                    color.filled(),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }

        mcc_chart
            .configure_series_labels()
            .background_style(cfg.theme.background().mix(0.9))
            .border_style(cfg.theme.border())
            .label_font((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
            .draw()
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    if let Some(thrust_area) = thrust_area {
        let mut thrust_chart = ChartBuilder::on(&thrust_area)
            .caption(
                "Breadth Thrust",
                (TITLE_FONT, 16).into_font().color(&cfg.theme.text()),
            )
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(60)
            .build_cartesian_2d(0.0..n as f64, 0.0..6.0)
            .map_err(|e| BtError::Render(e.to_string()))?;

        thrust_chart
            .configure_mesh()
            .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
            .axis_style(&cfg.theme.border())
            .light_line_style(cfg.theme.border().mix(0.3))
            .draw()
            .map_err(|e| BtError::Render(e.to_string()))?;

        thrust_chart
            .draw_series(LineSeries::new(
                breadth
                    .iter()
                    .enumerate()
                    .map(|(i, b)| (i as f64, b.ad_ratio)),
                cfg.theme.info().stroke_width(2),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?
            .label("A/D Ratio")
            .legend(|(x, y)| {
                PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.info().stroke_width(2))
            });

        for (i, b) in breadth.iter().enumerate() {
            if b.thrust {
                thrust_chart
                    .draw_series(std::iter::once(Circle::new(
                        (i as f64, b.ad_ratio),
                        5,
                        cfg.theme.accent().filled(),
                    )))
                    .map_err(|e| BtError::Render(e.to_string()))?;
            }
        }

        thrust_chart
            .configure_series_labels()
            .background_style(cfg.theme.background().mix(0.9))
            .border_style(cfg.theme.border())
            .label_font((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
            .draw()
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(breadth: &[BreadthData], cfg: &MarketBreadthConfig, path: &str) -> Result<()> {
    render(png_root(path)?, breadth, cfg)
}

pub fn render_svg(breadth: &[BreadthData], cfg: &MarketBreadthConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, breadth, cfg)
}

pub fn render_sample_png(cfg: &MarketBreadthConfig, path: &str) -> Result<()> {
    let data = sample_breadth();
    render_png(&data, cfg, path)
}

pub fn render_sample_svg(cfg: &MarketBreadthConfig, path: &str) -> Result<()> {
    let data = sample_breadth();
    render_svg(&data, cfg, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let data = sample_breadth();
        let cfg = MarketBreadthConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_market_breadth.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&data, &cfg, &path).unwrap();
    }
}
