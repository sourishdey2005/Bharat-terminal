// crates/bt-viz/src/candlestick_3d.rs
// Author: Sourish Dey

//! Tier 1 #10 â€” Candlestick 3D Perspective (pseudo-3D via depth-shading).
//! Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct Candlestick3DConfig {
    pub title: String,
    pub theme: Theme,
    pub depth_factor: f64, // 0.0 to 1.0, how much depth to show
    pub show_volume: bool,
}

impl Default for Candlestick3DConfig {
    fn default() -> Self {
        Self {
            title: "Candlestick 3D".to_string(),
            theme: Theme::Dark,
            depth_factor: 0.15,
            show_volume: true,
        }
    }
}

impl Candlestick3DConfig {
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

    pub fn depth_factor(mut self, factor: f64) -> Self {
        self.depth_factor = factor.clamp(0.0, 0.5);
        self
    }

    pub fn show_volume(mut self, show: bool) -> Self {
        self.show_volume = show;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &Candlestick3DConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let (chart_area, volume_area) = if cfg.show_volume {
        let split = root.split_vertically((70).percent());
        (split.0, Some(split.1))
    } else {
        (root.clone(), None)
    };

    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;
    let low = series
        .candles
        .iter()
        .map(|c| c.low)
        .fold(f64::MAX, f64::min);
    let high = series
        .candles
        .iter()
        .map(|c| c.high)
        .fold(f64::MIN, f64::max);
    let pad = (high - low) * 0.05;

    let mut chart = ChartBuilder::on(&chart_area)
        .caption(
            format!("{} â€” {}", cfg.title, series.symbol),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, (low - pad)..(high + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let n = series.candles.len();
    let candle_width = ((t_max - t_min) / n as f64).max(0.3) * 0.4;
    let depth_offset = (t_max - t_min) * cfg.depth_factor / n as f64;

    // Draw candlesticks with 3D effect (back to front for proper layering)
    for i in (0..n).rev() {
        let c = &series.candles[i];
        let is_bullish = c.is_bullish();
        let base_color = if is_bullish {
            cfg.theme.profit()
        } else {
            cfg.theme.loss()
        };

        // 3D effect: draw side face (darker) and top face (lighter)
        let x = c.t;
        let x_back = x + depth_offset;
        let w = candle_width;

        // Side face (darker)
        let side_color = RGBColor(
            (base_color.0 as f64 * 0.6) as u8,
            (base_color.1 as f64 * 0.6) as u8,
            (base_color.2 as f64 * 0.6) as u8,
        );

        // Draw side parallelogram for body
        let body_top = c.open.max(c.close);
        let body_bottom = c.open.min(c.close);

        // Side face
        let side_points = vec![
            (x + w, body_bottom),
            (x_back + w, body_bottom),
            (x_back + w, body_top),
            (x + w, body_top),
        ];
        chart
            .draw_series(std::iter::once(Polygon::new(
                side_points,
                side_color.filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;

        // Top face (for bullish, top of body; for bearish, bottom)
        let top_color = if is_bullish {
            RGBColor(
                (base_color.0 as f64 * 1.2).min(255.0) as u8,
                (base_color.1 as f64 * 1.2).min(255.0) as u8,
                (base_color.2 as f64 * 1.2).min(255.0) as u8,
            )
        } else {
            base_color
        };

        let top_points = vec![
            (x, body_top),
            (x_back, body_top),
            (x_back + w, body_top),
            (x + w, body_top),
        ];
        chart
            .draw_series(std::iter::once(Polygon::new(
                top_points,
                top_color.filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;

        // Front face (main body)
        chart
            .draw_series(std::iter::once(Rectangle::new(
                [(x - w, body_bottom), (x + w, body_top)],
                base_color.filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;

        // Wicks (3D lines)
        let wick_x = x;
        let wick_back = x_back;

        // Front wick
        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![(wick_x, c.low), (wick_x, c.high)],
                base_color.stroke_width(1),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;

        // Back wick (darker)
        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![(wick_back, c.low), (wick_back, c.high)],
                side_color.stroke_width(1),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;

        // Wick top connectors
        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![(wick_x, c.high), (wick_back, c.high)],
                base_color.stroke_width(1),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![(wick_x, c.low), (wick_back, c.low)],
                base_color.stroke_width(1),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    if let Some(vol_area) = volume_area {
        let max_vol = series
            .candles
            .iter()
            .map(|c| c.volume)
            .fold(0.0_f64, f64::max);
        let mut vol_chart = ChartBuilder::on(&vol_area)
            .margin(10)
            .x_label_area_size(20)
            .y_label_area_size(60)
            .build_cartesian_2d(t_min..t_max, 0.0..(max_vol * 1.1))
            .map_err(|e| BtError::Render(e.to_string()))?;

        vol_chart
            .configure_mesh()
            .label_style((LABEL_FONT, 10).into_font().color(&cfg.theme.text()))
            .axis_style(&cfg.theme.border())
            .disable_x_mesh()
            .draw()
            .map_err(|e| BtError::Render(e.to_string()))?;

        // 3D volume bars
        for i in (0..n).rev() {
            let c = &series.candles[i];
            let is_bullish = c.is_bullish();
            let base_color = if is_bullish {
                cfg.theme.profit()
            } else {
                cfg.theme.loss()
            };
            let side_color = RGBColor(
                (base_color.0 as f64 * 0.6) as u8,
                (base_color.1 as f64 * 0.6) as u8,
                (base_color.2 as f64 * 0.6) as u8,
            );

            let x = c.t;
            let x_back = x + depth_offset;
            let w = candle_width;

            // Side face
            let side_points = vec![
                (x + w, 0.0),
                (x_back + w, 0.0),
                (x_back + w, c.volume),
                (x + w, c.volume),
            ];
            vol_chart
                .draw_series(std::iter::once(Polygon::new(
                    side_points,
                    side_color.filled(),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;

            // Top face
            let top_points = vec![
                (x, c.volume),
                (x_back, c.volume),
                (x_back + w, c.volume),
                (x + w, c.volume),
            ];
            vol_chart
                .draw_series(std::iter::once(Polygon::new(
                    top_points,
                    base_color.filled(),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;

            // Front face
            vol_chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(x - w, 0.0), (x + w, c.volume)],
                    base_color.mix(0.6).filled(),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &Candlestick3DConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &Candlestick3DConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_constructs() {
        let cfg = Candlestick3DConfig::new();
        let _ = cfg;
    }
}
