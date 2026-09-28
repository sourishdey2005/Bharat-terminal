// crates/bt-viz/src/rsi_heatmap_multi.rs
// Author: Sourish Dey

//! RSI across multiple timeframes. Made by Sourish Dey.

use bt_analytics::rsi;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;
use plotters::style::RGBColor;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct RsiHeatmapMultiConfig {
    pub title: String,
    pub theme: Theme,
    pub timeframes: Vec<usize>,
}

impl Default for RsiHeatmapMultiConfig {
    fn default() -> Self {
        Self {
            title: "RSI Multi-Timeframe".to_string(),
            theme: Theme::Dark,
            timeframes: vec![5, 10, 14, 21, 50],
        }
    }
}

impl RsiHeatmapMultiConfig {
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

    pub fn timeframes(mut self, tf: Vec<usize>) -> Self {
        self.timeframes = if tf.is_empty() { vec![14] } else { tf };
        self
    }
}

fn rsi_to_color(value: f64, theme: Theme) -> RGBAColor {
    if value.is_nan() {
        return theme.border().mix(0.3);
    }
    if value >= 80.0 {
        theme.loss().mix(0.9)
    } else if value >= 60.0 {
        theme.loss().mix(0.5)
    } else if value >= 40.0 {
        theme.accent().mix(0.4)
    } else if value >= 20.0 {
        theme.profit().mix(0.5)
    } else {
        theme.profit().mix(0.9)
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &RsiHeatmapMultiConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;
    let n = series.candles.len();
    let num_tf = cfg.timeframes.len();

    let mut rsi_values: Vec<Vec<f64>> = Vec::new();
    for &tf in &cfg.timeframes {
        rsi_values.push(rsi(series, tf));
    }

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — {}", cfg.title, series.symbol),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(80)
        .build_cartesian_2d(t_min..t_max, 0.0..(num_tf as f64))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .y_labels(num_tf)
        .y_label_formatter(&|y| {
            let idx = *y as usize;
            if idx < num_tf {
                format!("TF{}", cfg.timeframes[idx])
            } else {
                String::new()
            }
        })
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let cell_w = (t_max - t_min) / n as f64;
    let cell_h = 1.0;

    for (tf_idx, values) in rsi_values.iter().enumerate() {
        let y = tf_idx as f64;
        for i in 0..n {
            let color = rsi_to_color(values[i], cfg.theme);
            chart
                .draw_series(std::iter::once(Rectangle::new(
                    [
                        (series.candles[i].t, y),
                        (series.candles[i].t + cell_w, y + cell_h),
                    ],
                    color.filled(),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;

            if !values[i].is_nan() && n <= 40 {
                chart
                    .draw_series(std::iter::once(Text::new(
                        format!("{:.0}", values[i]),
                        (series.candles[i].t + cell_w * 0.5, y + cell_h * 0.5),
                        (LABEL_FONT, 8).into_font().color(&cfg.theme.text()),
                    )))
                    .map_err(|e| BtError::Render(e.to_string()))?;
            }
        }
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &RsiHeatmapMultiConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &RsiHeatmapMultiConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = RsiHeatmapMultiConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_rsi_heatmap_multi.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
