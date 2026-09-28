// crates/bt-viz/src/tri_stick_compare.rs
// Author: Sourish Dey

//! Side-by-side triangle-stick comparison of two symbols. Made by Sourish Dey.

use bt_analytics::correlation;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct TriStickCompareConfig {
    pub title: String,
    pub theme: Theme,
    pub normalize: bool,
    pub base_value: f64,
}

impl Default for TriStickCompareConfig {
    fn default() -> Self {
        Self {
            title: "Tri-Stick Compare".to_string(),
            theme: Theme::Dark,
            normalize: true,
            base_value: 100.0,
        }
    }
}

impl TriStickCompareConfig {
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

    pub fn normalize(mut self, n: bool) -> Self {
        self.normalize = n;
        self
    }

    pub fn base_value(mut self, v: f64) -> Self {
        self.base_value = v.max(1.0);
        self
    }
}

fn draw_panel<DB: DrawingBackend>(
    area: &plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    scale: f64,
    theme: Theme,
    caption: &str,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;
    let low = series
        .candles
        .iter()
        .map(|c| c.low * scale)
        .fold(f64::MAX, f64::min);
    let high = series
        .candles
        .iter()
        .map(|c| c.high * scale)
        .fold(f64::MIN, f64::max);
    let pad = (high - low).max(1.0) * 0.08;

    let mut chart = ChartBuilder::on(area)
        .caption(
            caption.to_string(),
            (TITLE_FONT, 16).into_font().color(&theme.text()),
        )
        .margin(6)
        .x_label_area_size(20)
        .y_label_area_size(45)
        .build_cartesian_2d(t_min..t_max, (low - pad)..(high + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 9).into_font().color(&theme.text()))
        .axis_style(&theme.border())
        .light_line_style(theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let bar_width = ((t_max - t_min) / series.candles.len() as f64).max(0.3) * 0.35;

    for c in &series.candles {
        let color = if c.is_bullish() {
            theme.profit()
        } else {
            theme.loss()
        };

        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![(c.t, c.high * scale), (c.t, c.low * scale)],
                color.stroke_width(1),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;

        chart
            .draw_series(std::iter::once(Polygon::new(
                vec![
                    (c.t, c.close * scale),
                    (c.t - bar_width, c.open * scale),
                    (c.t + bar_width, c.open * scale),
                ],
                color.filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    Ok(())
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    a: &OhlcvSeries,
    b: &OhlcvSeries,
    cfg: &TriStickCompareConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    a.validate()?;
    b.validate()?;
    fill_background(&root, cfg.theme)?;

    let ret_a = a.returns();
    let ret_b = b.returns();
    let n = ret_a.len().min(ret_b.len());
    let corr = if n > 1 {
        correlation(&ret_a[..n], &ret_b[..n])
    } else {
        0.0
    };

    let scale_a = if cfg.normalize {
        cfg.base_value / a.candles.first().unwrap().close
    } else {
        1.0
    };
    let scale_b = if cfg.normalize {
        cfg.base_value / b.candles.first().unwrap().close
    } else {
        1.0
    };

    let (left, right) = root.split_horizontally((50).percent());

    draw_panel(
        &left,
        a,
        scale_a,
        cfg.theme,
        &format!("{} — {}", cfg.title, a.symbol),
    )?;
    draw_panel(
        &right,
        b,
        scale_b,
        cfg.theme,
        &format!("{} — {}", cfg.title, b.symbol),
    )?;

    root.draw(&Text::new(
        format!("Returns correlation: {:.2}", corr),
        (15, 40),
        (LABEL_FONT, 12).into_font().color(&cfg.theme.accent()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(
    a: &OhlcvSeries,
    b: &OhlcvSeries,
    cfg: &TriStickCompareConfig,
    path: &str,
) -> Result<()> {
    render(png_root(path)?, a, b, cfg)
}

pub fn render_svg(
    a: &OhlcvSeries,
    b: &OhlcvSeries,
    cfg: &TriStickCompareConfig,
    path: &str,
) -> Result<()> {
    render(svg_root(path)?, a, b, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let a = synthetic_ohlcv("AAA", 100, 1, 100.0);
        let b = synthetic_ohlcv("BBB", 100, 2, 50.0);
        let cfg = TriStickCompareConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_tri_stick_compare.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&a, &b, &cfg, &path).unwrap();
    }
}
