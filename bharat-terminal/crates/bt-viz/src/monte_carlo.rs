// crates/bt-viz/src/monte_carlo.rs
// Author: Sourish Dey

//! Monte Carlo path cloud. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct MonteCarloConfig {
    pub title: String,
    pub theme: Theme,
    pub n_paths: usize,
    pub horizon: usize,
    pub seed: u64,
}

impl Default for MonteCarloConfig {
    fn default() -> Self {
        Self {
            title: "Monte Carlo Path Cloud".to_string(),
            theme: Theme::Dark,
            n_paths: 50,
            horizon: 60,
            seed: 42,
        }
    }
}

impl MonteCarloConfig {
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
    pub fn n_paths(mut self, n: usize) -> Self {
        self.n_paths = n;
        self
    }
    pub fn horizon(mut self, h: usize) -> Self {
        self.horizon = h;
        self
    }
    pub fn seed(mut self, s: u64) -> Self {
        self.seed = s;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &MonteCarloConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let returns = series.returns();
    if returns.is_empty() {
        return Err(BtError::EmptySeries("returns".into()));
    }

    let n = returns.len() as f64;
    let mean = returns.iter().sum::<f64>() / n;
    let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n;
    let vol = variance.sqrt();
    let last_price = series.candles.last().map(|c| c.close).unwrap_or(100.0);

    let mut rng = StdRng::seed_from_u64(cfg.seed);
    let mut all_paths: Vec<Vec<f64>> = Vec::with_capacity(cfg.n_paths);

    for _ in 0..cfg.n_paths {
        let mut path = Vec::with_capacity(cfg.horizon);
        let mut price = last_price;
        for _ in 0..cfg.horizon {
            let shock: f64 = rng.gen_range(-1.0..1.0);
            let ret = mean + vol * shock;
            price = (price * (1.0 + ret)).max(0.01);
            path.push(price);
        }
        all_paths.push(path);
    }

    let y_min = all_paths
        .iter()
        .flat_map(|p| p.iter().cloned())
        .fold(f64::MAX, f64::min);
    let y_max = all_paths
        .iter()
        .flat_map(|p| p.iter().cloned())
        .fold(f64::MIN, f64::max);
    let pad = (y_max - y_min).max(1.0) * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            &cfg.title,
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(15)
        .x_label_area_size(35)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..cfg.horizon as f64, (y_min - pad)..(y_max + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .x_desc("Step")
        .y_desc("Price")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let palette = cfg.theme.categorical(0);
    for (idx, path) in all_paths.iter().enumerate() {
        let color = palette[idx % palette.len()];
        let points: Vec<(f64, f64)> = path
            .iter()
            .enumerate()
            .map(|(i, v)| (i as f64, *v))
            .collect();
        chart
            .draw_series(LineSeries::new(points, color.mix(0.4).stroke_width(1)))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    let median_path: Vec<f64> = (0..cfg.horizon)
        .map(|step| {
            let mut vals: Vec<f64> = all_paths.iter().map(|p| p[step]).collect();
            vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
            vals[vals.len() / 2]
        })
        .collect();
    let median_points: Vec<(f64, f64)> = median_path
        .iter()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect();
    chart
        .draw_series(LineSeries::new(
            median_points,
            cfg.theme.accent().stroke_width(3),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Median")
        .legend(move |(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                cfg.theme.accent().stroke_width(3),
            )
        });

    chart
        .configure_series_labels()
        .background_style(cfg.theme.background().mix(0.8))
        .border_style(cfg.theme.border())
        .label_font((LABEL_FONT, 13).into_font().color(&cfg.theme.text()))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &MonteCarloConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &MonteCarloConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = MonteCarloConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_monte_carlo.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
