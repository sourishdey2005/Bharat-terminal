// crates/bt-viz/src/adx.rs
// Author: Sourish Dey

//! Tier 3 â€” ADX / DI+ / DI- (Trend strength + threshold).
//! Made by Sourish Dey.

use bt_analytics::adx;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct ADXConfig {
    pub title: String,
    pub theme: Theme,
    pub period: usize,
    pub threshold: f64,
}

impl Default for ADXConfig {
    fn default() -> Self {
        Self {
            title: "ADX / DI+ / DI-".to_string(),
            theme: Theme::Dark,
            period: 14,
            threshold: 25.0,
        }
    }
}

impl ADXConfig {
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

    pub fn period(mut self, period: usize) -> Self {
        self.period = period.max(2);
        self
    }

    pub fn threshold(mut self, level: f64) -> Self {
        self.threshold = level;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &ADXConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;

    let (adx_vals, plus_di, minus_di) = adx(series, cfg.period);

    // Layout: Price (top 55%), ADX panel (bottom 45%)
    let (price_area, adx_area) = root.split_vertically((55).percent());

    // Price chart
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

    let mut price_chart = ChartBuilder::on(&price_area)
        .caption(
            format!("{} â€” {}", cfg.title, series.symbol),
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

    let candle_width = ((t_max - t_min) / series.candles.len() as f64).max(0.3) * 0.4;
    price_chart
        .draw_series(series.candles.iter().map(|c| {
            let color = if c.is_bullish() {
                cfg.theme.profit()
            } else {
                cfg.theme.loss()
            };
            CandleStick::new(
                c.t,
                c.open,
                c.high,
                c.low,
                c.close,
                color.filled(),
                color.filled(),
                (candle_width * 10.0) as u32,
            )
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    // ADX panel
    let mut adx_chart = ChartBuilder::on(&adx_area)
        .caption(
            format!("ADX({}) / DI+ / DI-", cfg.period),
            (TITLE_FONT, 16).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, 0.0..100.0)
        .map_err(|e| BtError::Render(e.to_string()))?;

    adx_chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Threshold line
    adx_chart
        .draw_series(LineSeries::new(
            vec![(t_min, cfg.threshold), (t_max, cfg.threshold)],
            cfg.theme.loss().mix(0.7).stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    // ADX line
    adx_chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !adx_vals[i].is_nan() {
                    Some((c.t, adx_vals[i]))
                } else {
                    None
                }
            }),
            cfg.theme.info().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("ADX")
        .legend(|(x, y)| {
            PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.info().stroke_width(2))
        });

    // +DI line
    adx_chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !plus_di[i].is_nan() {
                    Some((c.t, plus_di[i]))
                } else {
                    None
                }
            }),
            cfg.theme.profit().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("+DI")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                cfg.theme.profit().stroke_width(2),
            )
        });

    // -DI line
    adx_chart
        .draw_series(LineSeries::new(
            series.candles.iter().enumerate().filter_map(|(i, c)| {
                if !minus_di[i].is_nan() {
                    Some((c.t, minus_di[i]))
                } else {
                    None
                }
            }),
            cfg.theme.loss().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("-DI")
        .legend(|(x, y)| {
            PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.loss().stroke_width(2))
        });

    adx_chart
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

pub fn render_png(series: &OhlcvSeries, cfg: &ADXConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &ADXConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_constructs() {
        let cfg = ADXConfig::new();
        let _ = cfg;
    }
}
