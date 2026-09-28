// crates/bt-viz/src/oi_profile.rs
// Author: Sourish Dey

//! OI: Open interest profile.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct OiStrike {
    pub strike: f64,
    pub call_oi: f64,
    pub put_oi: f64,
}

impl OiStrike {
    pub fn new(strike: f64, call_oi: f64, put_oi: f64) -> Self {
        Self {
            strike,
            call_oi: call_oi.max(0.0),
            put_oi: put_oi.max(0.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OiProfileConfig {
    pub title: String,
    pub theme: Theme,
    pub show_ratio: bool,
}

impl Default for OiProfileConfig {
    fn default() -> Self {
        Self {
            title: "Open Interest Profile".to_string(),
            theme: Theme::Dark,
            show_ratio: true,
        }
    }
}

impl OiProfileConfig {
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
    pub fn show_ratio(mut self, s: bool) -> Self {
        self.show_ratio = s;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    strikes: &[OiStrike],
    cfg: &OiProfileConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if strikes.is_empty() {
        return Err(BtError::EmptySeries("OI strikes".into()));
    }
    fill_background(&root, cfg.theme)?;

    let max_oi = strikes
        .iter()
        .map(|s| s.call_oi.max(s.put_oi))
        .fold(0.0_f64, f64::max);

    let mut chart = ChartBuilder::on(&root)
        .caption(
            cfg.title.clone(),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0..strikes.len(), -max_oi * 1.1..max_oi * 1.1)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .x_labels(strikes.len())
        .x_label_formatter(&|x| {
            strikes
                .get(*x)
                .map(|s| format!("{:.0}", s.strike))
                .unwrap_or_default()
        })
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(strikes.iter().enumerate().map(|(i, s)| {
            Rectangle::new(
                [(i, 0.0), (i + 1, s.call_oi)],
                cfg.theme.profit().mix(0.6).filled(),
            )
        }))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Call OI")
        .legend(|(x, y)| {
            Rectangle::new(
                [(x, y - 5), (x + 20, y + 5)],
                cfg.theme.profit().mix(0.6).filled(),
            )
        });

    chart
        .draw_series(strikes.iter().enumerate().map(|(i, s)| {
            Rectangle::new(
                [(i, 0.0), (i + 1, -s.put_oi)],
                cfg.theme.loss().mix(0.6).filled(),
            )
        }))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Put OI")
        .legend(|(x, y)| {
            Rectangle::new(
                [(x, y - 5), (x + 20, y + 5)],
                cfg.theme.loss().mix(0.6).filled(),
            )
        });

    if cfg.show_ratio {
        let max_ratio = strikes
            .iter()
            .map(|s| {
                if s.call_oi > 0.0 {
                    s.put_oi / s.call_oi
                } else {
                    0.0
                }
            })
            .fold(0.0_f64, f64::max);

        if max_ratio > 0.0 {
            chart
                .draw_series(LineSeries::new(
                    strikes.iter().enumerate().map(|(i, s)| {
                        let ratio = if s.call_oi > 0.0 {
                            s.put_oi / s.call_oi
                        } else {
                            0.0
                        };
                        (i, ratio * max_oi * 0.5)
                    }),
                    cfg.theme.accent().stroke_width(2),
                ))
                .map_err(|e| BtError::Render(e.to_string()))?
                .label("P/C Ratio")
                .legend(|(x, y)| {
                    PathElement::new(
                        vec![(x, y), (x + 20, y)],
                        cfg.theme.accent().stroke_width(2),
                    )
                });
        }
    }

    chart
        .configure_series_labels()
        .background_style(cfg.theme.background().mix(0.9))
        .border_style(cfg.theme.border())
        .label_font((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(strikes: &[OiStrike], cfg: &OiProfileConfig, path: &str) -> Result<()> {
    render(png_root(path)?, strikes, cfg)
}

pub fn render_svg(strikes: &[OiStrike], cfg: &OiProfileConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, strikes, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let strikes: Vec<OiStrike> = (0..10)
            .map(|i| {
                OiStrike::new(
                    90.0 + i as f64 * 5.0,
                    (10 - i) as f64 * 1000.0,
                    i as f64 * 1000.0,
                )
            })
            .collect();
        let cfg = OiProfileConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_oi_profile.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&strikes, &cfg, &path).unwrap();
    }
}
