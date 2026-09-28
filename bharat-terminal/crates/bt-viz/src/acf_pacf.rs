//! Tier 4 #24 â€” Autocorrelation (ACF/PACF) lollipop chart with confidence
//! bands. Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

/// Computes the sample autocorrelation function up to `max_lag`.
pub fn acf(series: &[f64], max_lag: usize) -> Vec<f64> {
    let n = series.len();
    let mean = series.iter().sum::<f64>() / n as f64;
    let var: f64 = series.iter().map(|v| (v - mean).powi(2)).sum();
    if var <= 0.0 {
        return vec![0.0; max_lag + 1];
    }
    (0..=max_lag)
        .map(|lag| {
            if lag == 0 {
                return 1.0;
            }
            if lag >= n {
                return 0.0;
            }
            let mut cov = 0.0;
            for t in 0..(n - lag) {
                cov += (series[t] - mean) * (series[t + lag] - mean);
            }
            cov / var
        })
        .collect()
}

/// Approximates the partial autocorrelation via the Durbinâ€“Levinson
/// recursion, which is exact for Gaussian AR processes and a standard
/// textbook estimator elsewhere.
pub fn pacf(series: &[f64], max_lag: usize) -> Vec<f64> {
    let r = acf(series, max_lag);
    let mut phi = vec![vec![0.0; max_lag + 1]; max_lag + 1];
    let mut out = vec![0.0; max_lag + 1];
    if max_lag == 0 {
        return out;
    }
    phi[1][1] = r[1];
    out[1] = r[1];
    for k in 2..=max_lag {
        let mut num = r[k];
        for j in 1..k {
            num -= phi[k - 1][j] * r[k - j];
        }
        let mut den = 1.0;
        for j in 1..k {
            den -= phi[k - 1][j] * r[j];
        }
        let phi_kk = if den.abs() > 1e-9 { num / den } else { 0.0 };
        phi[k][k] = phi_kk;
        for j in 1..k {
            phi[k][j] = phi[k - 1][j] - phi_kk * phi[k - 1][k - j];
        }
        out[k] = phi_kk;
    }
    out
}

#[derive(Debug, Clone)]
pub struct AcfPacfConfig {
    pub title: String,
    pub theme: Theme,
    pub max_lag: usize,
}

impl Default for AcfPacfConfig {
    fn default() -> Self {
        Self {
            title: "ACF / PACF".to_string(),
            theme: Theme::Dark,
            max_lag: 20,
        }
    }
}

impl AcfPacfConfig {
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
    pub fn max_lag(mut self, lag: usize) -> Self {
        self.max_lag = lag;
        self
    }
}

fn draw_lollipop_panel<DB: DrawingBackend>(
    area: &plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    caption: &str,
    values: &[f64],
    conf: f64,
    theme: Theme,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    let n = values.len() as f64;
    let mut chart = ChartBuilder::on(area)
        .caption(caption, (TITLE_FONT, 18).into_font().color(&theme.text()))
        .margin(10)
        .x_label_area_size(25)
        .y_label_area_size(50)
        .build_cartesian_2d(-0.5f64..n, -1.05f64..1.05f64)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 11).into_font().color(&theme.text()))
        .axis_style(&theme.border())
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(std::iter::once(Rectangle::new(
            [(-0.5, -conf), (n, conf)],
            theme.info().mix(0.15).filled(),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    for (lag, &v) in values.iter().enumerate() {
        let color = if v.abs() > conf {
            theme.accent()
        } else {
            theme.text()
        };
        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![(lag as f64, 0.0), (lag as f64, v)],
                color.stroke_width(2),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
        chart
            .draw_series(std::iter::once(Circle::new(
                (lag as f64, v),
                4,
                color.filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    Ok(())
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &[f64],
    cfg: &AcfPacfConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if series.len() < cfg.max_lag + 2 {
        return Err(BtError::InvalidInput(format!(
            "series too short ({}) for max_lag {}",
            series.len(),
            cfg.max_lag
        )));
    }
    fill_background(&root, cfg.theme)?;

    let acf_vals = acf(series, cfg.max_lag);
    let pacf_vals = pacf(series, cfg.max_lag);
    let conf = 1.96 / (series.len() as f64).sqrt();

    let (top, bottom) = root.split_vertically((50).percent());
    draw_lollipop_panel(
        &top,
        &format!("{} â€” ACF", cfg.title),
        &acf_vals,
        conf,
        cfg.theme,
    )?;
    draw_lollipop_panel(
        &bottom,
        &format!("{} â€” PACF", cfg.title),
        &pacf_vals,
        conf,
        cfg.theme,
    )?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &[f64], cfg: &AcfPacfConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &[f64], cfg: &AcfPacfConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_constructs() {
        let cfg = AcfPacfConfig::new();
        let _ = cfg;
    }
}
