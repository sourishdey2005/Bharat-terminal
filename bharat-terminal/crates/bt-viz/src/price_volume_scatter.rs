// crates/bt-viz/src/price_volume_scatter.rs
// Author: Sourish Dey

//! Tier 8 — Price-Volume Scatter with Regression.
//! Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct PriceVolumeScatterConfig {
    pub title: String,
    pub theme: Theme,
    pub show_regression: bool,
    pub show_correlation: bool,
}

impl Default for PriceVolumeScatterConfig {
    fn default() -> Self {
        Self {
            title: "Price-Volume Scatter".to_string(),
            theme: Theme::Dark,
            show_regression: true,
            show_correlation: true,
        }
    }
}

impl PriceVolumeScatterConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn show_regression(mut self, show: bool) -> Self {
        self.show_regression = show;
        self
    }

    pub fn show_correlation(mut self, show: bool) -> Self {
        self.show_correlation = show;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &PriceVolumeScatterConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    // Prepare data points: (volume, price change %)
    let mut points = Vec::new();
    for i in 1..series.candles.len() {
        let prev_close = series.candles[i - 1].close;
        let curr_close = series.candles[i].close;
        let volume = series.candles[i].volume;
        let change_pct = if prev_close != 0.0 {
            (curr_close - prev_close) / prev_close * 100.0
        } else {
            0.0
        };
        points.push((volume, change_pct));
    }

    if points.is_empty() {
        // Draw empty chart
        let (w, h) = root.dim_in_pixel();
        root.draw(&Text::new(
            "Insufficient data for scatter plot",
            (w as i32 / 2, h as i32 / 2),
            (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
        draw_footer(&root, cfg.theme)?;
        root.present().map_err(|e| BtError::Render(e.to_string()))?;
        return Ok(());
    }

    let vol_min = points.iter().map(|(v, _)| *v).fold(f64::INFINITY, f64::min);
    let vol_max = points
        .iter()
        .map(|(v, _)| *v)
        .fold(f64::NEG_INFINITY, f64::max);
    let change_min = points.iter().map(|(_, c)| *c).fold(f64::INFINITY, f64::min);
    let change_max = points
        .iter()
        .map(|(_, c)| *c)
        .fold(f64::NEG_INFINITY, f64::max);

    let vol_pad = (vol_max - vol_min) * 0.05;
    let change_pad = (change_max - change_min) * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — {}", cfg.title, series.symbol),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(
            (vol_min - vol_pad)..(vol_max + vol_pad),
            (change_min - change_pad)..(change_max + change_pad),
        )
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .x_desc("Volume")
        .y_desc("Price Change (%)")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Zero line
    chart
        .draw_series(LineSeries::new(
            vec![(vol_min - vol_pad, 0.0), (vol_max + vol_pad, 0.0)],
            cfg.theme.border().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Scatter points
    chart
        .draw_series(points.iter().map(|(vol, change)| {
            let color = if *change >= 0.0 {
                cfg.theme.profit()
            } else {
                cfg.theme.loss()
            };
            Circle::new((*vol, *change), 3, color.filled())
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Linear regression
    if cfg.show_regression && points.len() >= 2 {
        let n = points.len() as f64;
        let sum_x: f64 = points.iter().map(|(x, _)| *x).sum();
        let sum_y: f64 = points.iter().map(|(_, y)| *y).sum();
        let sum_xy: f64 = points.iter().map(|(x, y)| x * y).sum();
        let sum_x2: f64 = points.iter().map(|(x, _)| x * x).sum();

        let denom = n * sum_x2 - sum_x * sum_x;
        if denom.abs() > 1e-10 {
            let slope = (n * sum_xy - sum_x * sum_y) / denom;
            let intercept = (sum_y - slope * sum_x) / n;

            // Correlation coefficient
            let mean_x = sum_x / n;
            let mean_y = sum_y / n;
            let mut ss_xy = 0.0;
            let mut ss_xx = 0.0;
            let mut ss_yy = 0.0;
            for (x, y) in &points {
                let dx = x - mean_x;
                let dy = y - mean_y;
                ss_xy += dx * dy;
                ss_xx += dx * dx;
                ss_yy += dy * dy;
            }
            let correlation = if ss_xx > 0.0 && ss_yy > 0.0 {
                ss_xy / (ss_xx * ss_yy).sqrt()
            } else {
                0.0
            };

            // Draw regression line
            let x1 = vol_min - vol_pad;
            let x2 = vol_max + vol_pad;
            let y1 = slope * x1 + intercept;
            let y2 = slope * x2 + intercept;

            chart
                .draw_series(LineSeries::new(
                    vec![(x1, y1), (x2, y2)],
                    cfg.theme.accent().stroke_width(2),
                ))
                .map_err(|e| BtError::Render(e.to_string()))?
                .label(format!("Regression (r={:.3})", correlation))
                .legend(|(x, y)| {
                    PathElement::new(
                        vec![(x, y), (x + 20, y)],
                        cfg.theme.accent().stroke_width(2),
                    )
                });

            // Show correlation text
            if cfg.show_correlation {
                let (w, h) = root.dim_in_pixel();
                root.draw(&Text::new(
                    format!("Correlation: {:.3} | Slope: {:.6}", correlation, slope),
                    (w as i32 / 2, 45),
                    (LABEL_FONT, 14).into_font().color(&cfg.theme.accent()),
                ))
                .map_err(|e| BtError::Render(e.to_string()))?;
            }

            chart
                .configure_series_labels()
                .background_style(cfg.theme.background().mix(0.9))
                .border_style(cfg.theme.border())
                .label_font((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
                .draw()
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &PriceVolumeScatterConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &PriceVolumeScatterConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_constructs() {
        let cfg = PriceVolumeScatterConfig::new();
        let _ = cfg;
    }
}
