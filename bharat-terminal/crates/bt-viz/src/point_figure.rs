// crates/bt-viz/src/point_figure.rs
// Author: Sourish Dey

//! Point & Figure X/O chart. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct PointFigureConfig {
    pub title: String,
    pub theme: Theme,
    pub box_size: f64,
    pub reversal: usize,
}

impl Default for PointFigureConfig {
    fn default() -> Self {
        Self {
            title: "Point & Figure".to_string(),
            theme: Theme::Dark,
            box_size: 2.0,
            reversal: 3,
        }
    }
}

impl PointFigureConfig {
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

    pub fn box_size(mut self, size: f64) -> Self {
        self.box_size = size.max(0.01);
        self
    }

    pub fn reversal(mut self, r: usize) -> Self {
        self.reversal = r.max(1);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PfSymbol {
    X,
    O,
}

#[derive(Debug, Clone)]
struct PfColumn {
    symbols: Vec<PfSymbol>,
    start_price: f64,
    is_x: bool,
}

fn compute_point_figure(series: &OhlcvSeries, box_size: f64, reversal: usize) -> Vec<PfColumn> {
    if series.candles.is_empty() || box_size <= 0.0 {
        return Vec::new();
    }

    let mut columns: Vec<PfColumn> = Vec::new();
    let mut current_col: Option<PfColumn> = None;
    let mut last_price = series.candles[0].close;

    for candle in &series.candles {
        let close = candle.close;

        loop {
            match &mut current_col {
                None => {
                    let is_x = close > last_price;
                    let start = if is_x {
                        (close / box_size).floor() * box_size
                    } else {
                        (close / box_size).ceil() * box_size
                    };
                    current_col = Some(PfColumn {
                        symbols: vec![if is_x { PfSymbol::X } else { PfSymbol::O }],
                        start_price: start,
                        is_x,
                    });
                    last_price = close;
                    break;
                }
                Some(ref mut col) => {
                    if col.is_x {
                        let target = ((close - col.start_price) / box_size).floor() as i64;
                        let existing = col.symbols.len() as i64;
                        if target >= existing {
                            for _ in existing..target {
                                col.symbols.push(PfSymbol::X);
                            }
                            last_price = close;
                            break;
                        } else {
                            let change = existing - target;
                            if change >= reversal as i64 {
                                let new_start = col.start_price + existing as f64 * box_size;
                                let new_is_x = false;
                                columns.push(current_col.take().unwrap());
                                current_col = Some(PfColumn {
                                    symbols: vec![PfSymbol::O],
                                    start_price: new_start - box_size,
                                    is_x: new_is_x,
                                });
                                last_price = close;
                            } else {
                                break;
                            }
                        }
                    } else {
                        let target = ((col.start_price - close) / box_size).floor() as i64;
                        let existing = col.symbols.len() as i64;
                        if target >= existing {
                            for _ in existing..target {
                                col.symbols.push(PfSymbol::O);
                            }
                            last_price = close;
                            break;
                        } else {
                            let change = existing - target;
                            if change >= reversal as i64 {
                                let new_start = col.start_price - existing as f64 * box_size;
                                let new_is_x = true;
                                columns.push(current_col.take().unwrap());
                                current_col = Some(PfColumn {
                                    symbols: vec![PfSymbol::X],
                                    start_price: new_start + box_size,
                                    is_x: new_is_x,
                                });
                                last_price = close;
                            } else {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(col) = current_col.take() {
        columns.push(col);
    }

    columns
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &PointFigureConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let t_min = series.candles.first().unwrap().t;
    let t_max = series.candles.last().unwrap().t;
    let columns = compute_point_figure(series, cfg.box_size, cfg.reversal);

    if columns.is_empty() {
        let mut chart = ChartBuilder::on(&root)
            .caption(
                format!("{} — {} (No columns formed)", cfg.title, series.symbol),
                (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
            )
            .margin(10)
            .build_cartesian_2d(t_min..t_max, 0.0..1.0)
            .map_err(|e| BtError::Render(e.to_string()))?;

        chart
            .configure_mesh()
            .disable_mesh()
            .draw()
            .map_err(|e| BtError::Render(e.to_string()))?;

        draw_footer(&root, cfg.theme)?;
        root.present().map_err(|e| BtError::Render(e.to_string()))?;
        return Ok(());
    }

    let mut all_prices: Vec<f64> = Vec::new();
    for col in &columns {
        for (i, _) in col.symbols.iter().enumerate() {
            let price = if col.is_x {
                col.start_price + i as f64 * cfg.box_size
            } else {
                col.start_price - i as f64 * cfg.box_size
            };
            all_prices.push(price);
        }
    }

    let low = all_prices.iter().fold(f64::MAX, |a, &b| a.min(b));
    let high = all_prices.iter().fold(f64::MIN, |a, &b| a.max(b));
    let pad = (high - low).max(1.0) * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!(
                "{} — {} (Box: {:.2}, Rev: {})",
                cfg.title, series.symbol, cfg.box_size, cfg.reversal
            ),
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

    let col_width = (t_max - t_min) / columns.len().max(1) as f64;

    for (col_idx, col) in columns.iter().enumerate() {
        let x_center = t_min + (col_idx as f64 + 0.5) * col_width;
        let color = if col.is_x {
            cfg.theme.profit()
        } else {
            cfg.theme.loss()
        };

        for (row_idx, symbol) in col.symbols.iter().enumerate() {
            let price = if col.is_x {
                col.start_price + row_idx as f64 * cfg.box_size
            } else {
                col.start_price - row_idx as f64 * cfg.box_size
            };

            chart
                .draw_series(std::iter::once(Text::new(
                    match symbol {
                        PfSymbol::X => "X",
                        PfSymbol::O => "O",
                    },
                    (x_center, price),
                    (LABEL_FONT, 10).into_font().color(&color),
                )))
                .map_err(|e| BtError::Render(e.to_string()))?;
        }
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &PointFigureConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &PointFigureConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = PointFigureConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_point_figure.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
