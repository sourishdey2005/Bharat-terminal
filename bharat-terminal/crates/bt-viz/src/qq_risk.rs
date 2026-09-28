// crates/bt-viz/src/qq_risk.rs
// Author: Sourish Dey

//! Q-Q plot for tail risk.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;
use rand::Rng;
use rand::SeedableRng;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct QqRiskConfig {
    pub title: String,
    pub theme: Theme,
    pub confidence: f64,
    pub returns: Vec<f64>,
}

impl Default for QqRiskConfig {
    fn default() -> Self {
        Self {
            title: "Q-Q Plot (Tail Risk)".to_string(),
            theme: Theme::Dark,
            confidence: 0.95,
            returns: Vec::new(),
        }
    }
}

impl QqRiskConfig {
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
    pub fn confidence(mut self, c: f64) -> Self {
        self.confidence = c.clamp(0.5, 0.999);
        self
    }
    pub fn returns(mut self, r: Vec<f64>) -> Self {
        self.returns = r;
        self
    }
}

fn normal_quantile(p: f64) -> f64 {
    if p <= 0.0 || p >= 1.0 {
        return 0.0;
    }
    let a = [
        -3.969683028665376e+01,
        2.209460984245205e+02,
        -2.759285104469687e+02,
        1.383577518672690e+02,
        -3.066479806614716e+01,
        2.506628277459239e+00,
    ];
    let b = [
        -5.447609879822406e+01,
        1.615858368580409e+02,
        -1.556989798598866e+02,
        6.680131188771972e+01,
        -1.328068155288572e+01,
    ];
    let c = [
        -7.784894002430293e-03,
        -3.223964580411365e-01,
        -2.400758277161838e+00,
        -2.549732539343734e+00,
        4.374664141464968e+00,
        2.938163982698783e+00,
    ];
    let d = [
        7.784695709041462e-03,
        3.224671290700398e-01,
        2.445134137142996e+00,
        3.754408661907416e+00,
    ];

    let p_low = 0.02425;
    let p_high = 1.0 - p_low;

    if p < p_low {
        let q = (-2.0 * p.ln()).sqrt();
        (((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    } else if p <= p_high {
        let q = p - 0.5;
        let r = q * q;
        (((((a[0] * r + a[1]) * r + a[2]) * r + a[3]) * r + a[4]) * r + a[5]) * q
            / (((((b[0] * r + b[1]) * r + b[2]) * r + b[3]) * r + b[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    cfg: &QqRiskConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    fill_background(&root, cfg.theme)?;

    let returns = if cfg.returns.is_empty() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        (0..100)
            .map(|_| {
                let shock: f64 = rng.gen_range(-1.0..1.0);
                -0.01 + shock * 0.012
            })
            .collect()
    } else {
        cfg.returns.clone()
    };

    if returns.len() < 10 {
        return Err(BtError::InvalidInput(
            "Need at least 10 returns for Q-Q plot".into(),
        ));
    }

    let mut sorted = returns.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let n = sorted.len();
    let mean = sorted.iter().sum::<f64>() / n as f64;
    let variance = sorted.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n as f64;
    let std = variance.sqrt();

    let points: Vec<(f64, f64)> = (0..n)
        .map(|i| {
            let p = (i as f64 + 0.5) / n as f64;
            let theoretical = normal_quantile(p);
            let actual = (sorted[i] - mean) / std;
            (theoretical, actual)
        })
        .collect();

    let t_min = points.iter().map(|(t, _)| *t).fold(f64::INFINITY, f64::min);
    let t_max = points
        .iter()
        .map(|(t, _)| *t)
        .fold(f64::NEG_INFINITY, f64::max);
    let a_min = points.iter().map(|(_, a)| *a).fold(f64::INFINITY, f64::min);
    let a_max = points
        .iter()
        .map(|(_, a)| *a)
        .fold(f64::NEG_INFINITY, f64::max);

    let x_min = t_min.min(a_min);
    let x_max = t_max.max(a_max);
    let pad = (x_max - x_min).max(0.1) * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — N: {}", cfg.title, n),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d((x_min - pad)..(x_max + pad), (x_min - pad)..(x_max + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .x_desc("Theoretical Quantiles")
        .y_desc("Sample Quantiles")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(LineSeries::new(
            vec![(x_min - pad, x_min - pad), (x_max + pad, x_max + pad)],
            cfg.theme.border().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Normal")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                cfg.theme.border().mix(0.5).stroke_width(1),
            )
        });

    chart
        .draw_series(
            points
                .iter()
                .map(|(t, a)| Circle::new((*t, *a), 3, cfg.theme.info().filled())),
        )
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
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

pub fn render_png(cfg: &QqRiskConfig, path: &str) -> Result<()> {
    render(png_root(path)?, cfg)
}

pub fn render_svg(cfg: &QqRiskConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let cfg = QqRiskConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_qq_risk.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&cfg, &path).unwrap();
    }
}
