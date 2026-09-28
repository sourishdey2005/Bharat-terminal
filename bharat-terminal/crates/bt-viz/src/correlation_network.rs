// crates/bt-viz/src/correlation_network.rs
// Author: Sourish Dey

//! Cross-asset correlation network graph. Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

/// Asset node in the correlation network.
#[derive(Debug, Clone)]
pub struct AssetNode {
    pub symbol: String,
    pub name: String,
    pub asset_class: String,
    pub returns: Vec<f64>,
}

/// Sample cross-asset data.
fn sample_assets() -> Vec<AssetNode> {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    let mut rng = StdRng::seed_from_u64(42);
    let n = 120;

    let factor: Vec<f64> = (0..n).map(|_| rng.gen_range(-1.0..1.0) * 0.01).collect();

    let specs = [
        ("NIFTY", "Nifty 50", "Equity", 0.8),
        ("SENSEX", "Sensex", "Equity", 0.75),
        ("BANKNIFTY", "Bank Nifty", "Equity", 0.7),
        ("GOLD", "Gold", "Commodity", 0.1),
        ("CRUDE", "Crude Oil", "Commodity", 0.15),
        ("SILVER", "Silver", "Commodity", 0.2),
        ("USDINR", "USD/INR", "FX", 0.05),
        ("EURINR", "EUR/INR", "FX", 0.08),
        ("GBPINR", "GBP/INR", "FX", 0.06),
        ("US10Y", "US 10Y Bond", "Bond", -0.3),
        ("IND10Y", "India 10Y", "Bond", -0.2),
        ("BTC", "Bitcoin", "Crypto", 0.3),
    ];

    specs
        .into_iter()
        .enumerate()
        .map(|(idx, (sym, name, class, beta))| {
            let returns: Vec<f64> = (0..n)
                .map(|i| beta * factor[i] + rng.gen_range(-1.0..1.0) * 0.008)
                .collect();
            AssetNode {
                symbol: sym.to_string(),
                name: name.to_string(),
                asset_class: class.to_string(),
                returns,
            }
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct CorrelationNetworkConfig {
    pub title: String,
    pub theme: Theme,
    pub threshold: f64,
}

impl Default for CorrelationNetworkConfig {
    fn default() -> Self {
        Self {
            title: "Correlation Network".to_string(),
            theme: Theme::Dark,
            threshold: 0.5,
        }
    }
}

impl CorrelationNetworkConfig {
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
    pub fn threshold(mut self, t: f64) -> Self {
        self.threshold = t.clamp(0.0, 1.0);
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

fn class_color(theme: Theme, class: &str) -> RGBColor {
    match class {
        "Equity" => theme.profit(),
        "Commodity" => theme.accent(),
        "FX" => theme.info(),
        "Bond" => RGBColor(0xB0, 0x66, 0xFF),
        "Crypto" => RGBColor(0xFF, 0x8A, 0x00),
        _ => theme.text(),
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    data: &[AssetNode],
    cfg: &CorrelationNetworkConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if data.len() < 2 {
        return Err(BtError::InvalidInput("need at least 2 assets".into()));
    }
    fill_background(&root, cfg.theme)?;

    let (w, h) = root.dim_in_pixel();
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0 + 10.0;
    let radius = (w.min(h) as f64) * 0.35;

    root.draw(&Text::new(
        cfg.title.as_str(),
        (w as i32 / 2, 20),
        (TITLE_FONT, 20).into_font().color(&cfg.theme.text()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    // Position nodes in a circle
    let n = data.len();
    let positions: Vec<(f64, f64)> = (0..n)
        .map(|i| {
            let angle = (i as f64 / n as f64) * std::f64::consts::TAU - std::f64::consts::FRAC_PI_2;
            (
                cx + radius * angle.cos(),
                cy + radius * angle.sin(),
            )
        })
        .collect();

    // Draw edges
    for i in 0..n {
        for j in (i + 1)..n {
            let corr = pearson(&data[i].returns, &data[j].returns);
            if corr.abs() >= cfg.threshold {
                let (x0, y0) = positions[i];
                let (x1, y1) = positions[j];
                let alpha = (corr.abs() - cfg.threshold) / (1.0 - cfg.threshold);
                let color = if corr > 0.0 {
                    cfg.theme.profit().mix(alpha * 0.6)
                } else {
                    cfg.theme.loss().mix(alpha * 0.6)
                };
                root.draw(&PathElement::new(
                    vec![(x0 as i32, y0 as i32), (x1 as i32, y1 as i32)],
                    color.stroke_width((alpha * 3.0) as u32 + 1),
                ))
                .map_err(|e| BtError::Render(e.to_string()))?;
            }
        }
    }

    // Draw nodes
    for (i, node) in data.iter().enumerate() {
        let (x, y) = positions[i];
        let color = class_color(cfg.theme, &node.asset_class);

        root.draw(&Circle::new(
            (x as i32, y as i32),
            18,
            color.filled(),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        root.draw(&Circle::new(
            (x as i32, y as i32),
            18,
            cfg.theme.border().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

        root.draw(&Text::new(
            node.symbol.as_str(),
            (x as i32 - 20, y as i32 - 5),
            (LABEL_FONT, 10).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    // Legend
    let classes = ["Equity", "Commodity", "FX", "Bond", "Crypto"];
    for (idx, class) in classes.iter().enumerate() {
        let lx = 20;
        let ly = 60 + idx as i32 * 20;
        root.draw(&Circle::new((lx + 5, ly + 5), 6, class_color(cfg.theme, class).filled()))
            .map_err(|e| BtError::Render(e.to_string()))?;
        root.draw(&Text::new(
            *class,
            (lx + 18, ly),
            (LABEL_FONT, 11).into_font().color(&cfg.theme.text()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    root.draw(&Text::new(
        format!("Threshold: |corr| >= {:.2}", cfg.threshold),
        (w as i32 - 200, 45),
        (LABEL_FONT, 11)
            .into_font()
            .color(&cfg.theme.text().mix(0.6)),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(data: &[AssetNode], cfg: &CorrelationNetworkConfig, path: &str) -> Result<()> {
    render(png_root(path)?, data, cfg)
}

pub fn render_svg(data: &[AssetNode], cfg: &CorrelationNetworkConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, data, cfg)
}

pub fn render_sample_png(cfg: &CorrelationNetworkConfig, path: &str) -> Result<()> {
    let data = sample_assets();
    render_png(&data, cfg, path)
}

pub fn render_sample_svg(cfg: &CorrelationNetworkConfig, path: &str) -> Result<()> {
    let data = sample_assets();
    render_svg(&data, cfg, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let data = sample_assets();
        let cfg = CorrelationNetworkConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_corr_network.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&data, &cfg, &path).unwrap();
    }
}
