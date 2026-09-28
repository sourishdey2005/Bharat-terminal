// crates/bt-viz/src/copula_3d.rs
// Author: Sourish Dey

//! Copula dependency 3D scatter chart. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

/// Computes pseudo-observations (uniform marginals via rank) for two series.
pub fn pseudo_observations(a: &[f64], b: &[f64]) -> Vec<(f64, f64)> {
    fn rank_uniform(data: &[f64]) -> Vec<f64> {
        let n = data.len();
        let mut indexed: Vec<(usize, f64)> = data.iter().enumerate().map(|(i, &v)| (i, v)).collect();
        indexed.sort_by(|x, y| x.1.partial_cmp(&y.1).unwrap_or(std::cmp::Ordering::Equal));
        let mut ranks = vec![0.0; n];
        for (rank, (orig_idx, _)) in indexed.iter().enumerate() {
            ranks[*orig_idx] = (rank as f64 + 0.5) / n as f64;
        }
        ranks
    }

    let ua = rank_uniform(a);
    let ub = rank_uniform(b);
    ua.into_iter().zip(ub.into_iter()).collect()
}

#[derive(Debug, Clone)]
pub struct Copula3DConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for Copula3DConfig {
    fn default() -> Self {
        Self {
            title: "Copula Dependency".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl Copula3DConfig {
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
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &Copula3DConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    let returns = series.returns();
    if returns.len() < 10 {
        return Err(BtError::InvalidInput("series too short for copula".into()));
    }
    fill_background(&root, cfg.theme)?;

    // Split returns into two halves and compute pseudo-observations
    let mid = returns.len() / 2;
    let a = &returns[..mid];
    let b = &returns[mid..];
    let obs = pseudo_observations(a, b);

    root.draw(&Text::new(
        cfg.title.as_str(),
        (root.dim_in_pixel().0 as i32 / 2, 20),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    let (w, h) = root.dim_in_pixel();
    let top_pad = 50;
    let bottom_pad = 50;
    let side_pad = 60;
    let plot_w = w as f64 - 2.0 * side_pad as f64;
    let plot_h = h as f64 - top_pad as f64 - bottom_pad as f64;

    // Axes
    root.draw(&PathElement::new(
        vec![
            (side_pad as i32, (top_pad as f64 + plot_h) as i32),
            ((side_pad as f64 + plot_w) as i32, (top_pad as f64 + plot_h) as i32),
        ],
        cfg.theme.border().stroke_width(1),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    root.draw(&PathElement::new(
        vec![
            (side_pad as i32, top_pad as i32),
            (side_pad as i32, (top_pad as f64 + plot_h) as i32),
        ],
        cfg.theme.border().stroke_width(1),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    // Axis labels
    root.draw(&Text::new(
        "Series A (uniform)",
        (side_pad as i32 + 40, (top_pad as f64 + plot_h + 20.0) as i32),
        (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    root.draw(&Text::new(
        "Series B",
        (side_pad as i32 - 40, (top_pad + 10) as i32),
        (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    // Diagonal (independence reference)
    root.draw(&PathElement::new(
        vec![
            (side_pad as i32, (top_pad as f64 + plot_h) as i32),
            ((side_pad as f64 + plot_w) as i32, top_pad as i32),
        ],
        cfg.theme.info().mix(0.5).stroke_width(1),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    // Scatter points
    for (ua, ub) in &obs {
        let x = side_pad as f64 + ua * plot_w;
        let y = top_pad as f64 + (1.0 - ub) * plot_h;

        // Color by distance from diagonal
        let dist = (ua - ub).abs();
        let color = if dist < 0.1 {
            cfg.theme.accent().mix(0.8)
        } else if *ua > 0.8 && *ub > 0.8 {
            cfg.theme.profit().mix(0.8)
        } else if *ua < 0.2 && *ub < 0.2 {
            cfg.theme.loss().mix(0.8)
        } else {
            cfg.theme.text().mix(0.4)
        };

        root.draw(&Circle::new(
            (x as i32, y as i32),
            3,
            color.filled(),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    // Legend
    root.draw(&Text::new(
        "Dependency: tail clustering visible in corners",
        (side_pad as i32, top_pad as i32 - 10),
        (LABEL_FONT, 11)
            .into_font()
            .color(&cfg.theme.text().mix(0.7)),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &Copula3DConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &Copula3DConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 200, 1, 100.0);
        let cfg = Copula3DConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_copula.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
