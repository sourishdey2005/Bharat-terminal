// crates/bt-viz/src/gpo_ohlc.rs
// Author: Sourish Dey

//! GPO: OHLC bar chart for volatility discovery.
//! Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct GpoOhlcConfig {
    pub title: String,
    pub theme: Theme,
    pub bar_width: f64,
    pub show_volatility: bool,
}

impl Default for GpoOhlcConfig {
    fn default() -> Self {
        Self {
            title: "OHLC Bar Chart".to_string(),
            theme: Theme::Dark,
            bar_width: 0.6,
            show_volatility: true,
        }
    }
}

impl GpoOhlcConfig {
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
    pub fn bar_width(mut self, w: f64) -> Self {
        self.bar_width = w.max(0.1).min(1.0);
        self
    }
    pub fn show_volatility(mut self, s: bool) -> Self {
        self.show_volatility = s;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &GpoOhlcConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let t_min = series.candles.first().map(|c| c.t).unwrap_or(0.0);
    let t_max = series.candles.last().map(|c| c.t).unwrap_or(1.0);
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
    let pad = (high - low).max(1.0) * 0.05;

    let (price_area, vol_area) = if cfg.show_volatility {
        let split = root.split_vertically((70).percent());
        (split.0, Some(split.1))
    } else {
        (root.clone(), None)
    };

    let mut price_chart = ChartBuilder::on(&price_area)
        .caption(
            format!("{} — {}", cfg.title, series.symbol),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, (low - pad)..(high + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    price_chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let bw = cfg.bar_width;
    price_chart
        .draw_series(series.candles.iter().map(|c| {
            let color = if c.is_bullish() {
                cfg.theme.profit()
            } else {
                cfg.theme.loss()
            };
            Rectangle::new(
                [(c.t - bw / 2.0, c.open), (c.t + bw / 2.0, c.close)],
                color.filled(),
            )
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    price_chart
        .draw_series(series.candles.iter().map(|c| {
            let color = if c.is_bullish() {
                cfg.theme.profit()
            } else {
                cfg.theme.loss()
            };
            PathElement::new(vec![(c.t, c.high), (c.t, c.low)], color.stroke_width(1))
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    if let Some(vol_area) = vol_area {
        let mut vol_chart = ChartBuilder::on(&vol_area)
            .caption(
                "Volatility",
                (TITLE_FONT, 16).into_font().color(&cfg.theme.text()),
            )
            .margin(10)
            .x_label_area_size(20)
            .y_label_area_size(60)
            .build_cartesian_2d(t_min..t_max, 0.0..1.0)
            .map_err(|e| BtError::Render(e.to_string()))?;

        vol_chart
            .configure_mesh()
            .label_style((LABEL_FONT, 10).into_font().color(&cfg.theme.text()))
            .axis_style(&cfg.theme.border())
            .disable_x_mesh()
            .draw()
            .map_err(|e| BtError::Render(e.to_string()))?;

        let max_vol = series
            .candles
            .iter()
            .map(|c| c.volume)
            .fold(0.0_f64, f64::max);
        if max_vol > 0.0 {
            vol_chart
                .draw_series(series.candles.iter().map(|c| {
                    Rectangle::new(
                        [(c.t - bw / 2.0, 0.0), (c.t + bw / 2.0, c.volume / max_vol)],
                        cfg.theme.info().mix(0.4).filled(),
                    )
                }))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &GpoOhlcConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &GpoOhlcConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = GpoOhlcConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_gpo_ohlc.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
