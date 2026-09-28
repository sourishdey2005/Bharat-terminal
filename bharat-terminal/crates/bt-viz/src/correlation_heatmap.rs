//! Tier 3 #17 â€” Rolling correlation matrix heatmap. Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CorrelationHeatmapConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for CorrelationHeatmapConfig {
    fn default() -> Self {
        Self {
            title: "Correlation Matrix".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl CorrelationHeatmapConfig {
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
}

fn pearson(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len() as f64;
    let mean_a = a.iter().sum::<f64>() / n;
    let mean_b = b.iter().sum::<f64>() / n;
    let mut cov = 0.0;
    let mut var_a = 0.0;
    let mut var_b = 0.0;
    for i in 0..a.len() {
        let da = a[i] - mean_a;
        let db = b[i] - mean_b;
        cov += da * db;
        var_a += da * da;
        var_b += db * db;
    }
    if var_a <= 0.0 || var_b <= 0.0 {
        0.0
    } else {
        cov / (var_a.sqrt() * var_b.sqrt())
    }
}

/// Computes an NÃ—N Pearson correlation matrix from a set of return series.
pub fn correlation_matrix(series: &[(String, Vec<f64>)]) -> Vec<Vec<f64>> {
    let n = series.len();
    let mut m = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            m[i][j] = if i == j {
                1.0
            } else {
                pearson(&series[i].1, &series[j].1)
            };
        }
    }
    m
}

fn lerp_channel(bg: u8, fg: u8, t: f64) -> u8 {
    (bg as f64 + (fg as f64 - bg as f64) * t.clamp(0.0, 1.0)).round() as u8
}

fn corr_color(theme: Theme, v: f64) -> RGBColor {
    let v = v.clamp(-1.0, 1.0);
    let bg = theme.background();
    let target = if v >= 0.0 { theme.profit() } else { theme.loss() };
    let t = v.abs();
    RGBColor(
        lerp_channel(bg.0, target.0, t),
        lerp_channel(bg.1, target.1, t),
        lerp_channel(bg.2, target.2, t),
    )
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &[(String, Vec<f64>)],
    cfg: &CorrelationHeatmapConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if series.is_empty() {
        return Err(BtError::EmptySeries("correlation input".into()));
    }
    fill_background(&root, cfg.theme)?;
    let matrix = correlation_matrix(series);
    let n = series.len();

    let mut chart = ChartBuilder::on(&root)
        .caption(&cfg.title, (TITLE_FONT, 22).into_font().color(&cfg.theme.text()))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(80)
        .build_cartesian_2d(0..n, 0..n)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .disable_mesh()
        .x_labels(n)
        .y_labels(n)
        .x_label_formatter(&|idx| series.get(*idx).map(|s| s.0.clone()).unwrap_or_default())
        .y_label_formatter(&|idx| {
            (n.saturating_sub(1).checked_sub(*idx))
                .and_then(|i| series.get(i))
                .map(|s| s.0.clone())
                .unwrap_or_default()
        })
        .label_style((LABEL_FONT, 13).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let mut cells: Vec<(usize, usize, f64)> = Vec::with_capacity(n * n);
    for i in 0..n {
        for j in 0..n {
            cells.push((i, j, matrix[i][j]));
        }
    }

    chart
        .draw_series(cells.iter().map(|&(i, j, v)| {
            let y = n - 1 - j;
            Rectangle::new([(i, y), (i + 1, y + 1)], corr_color(cfg.theme, v).filled())
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(cells.iter().map(|&(i, j, v)| {
            let y = n - 1 - j;
            Text::new(
                format!("{:.2}", v),
                (i, y),
                (LABEL_FONT, 12).into_font().color(&cfg.theme.text()),
            )
        }))
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &[(String, Vec<f64>)], cfg: &CorrelationHeatmapConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &[(String, Vec<f64>)], cfg: &CorrelationHeatmapConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_constructs() {
        let cfg = CorrelationHeatmapConfig::new();
        let _ = cfg;
    }
}
