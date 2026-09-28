// crates/bt-viz/src/spa_structured.rs
// Author: Sourish Dey

//! SPA: Structured product cash flows.
//! Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct CashFlow {
    pub time: f64,
    pub amount: f64,
    pub flow_type: String,
}

impl CashFlow {
    pub fn new(time: f64, amount: f64, flow_type: impl Into<String>) -> Self {
        Self { time, amount, flow_type: flow_type.into() }
    }
}

#[derive(Debug, Clone)]
pub struct SpaStructuredConfig {
    pub title: String,
    pub theme: Theme,
    pub notional: f64,
    pub coupon_rate: f64,
    pub maturity: f64,
}

impl Default for SpaStructuredConfig {
    fn default() -> Self {
        Self {
            title: "Structured Product Cash Flows".to_string(),
            theme: Theme::Dark,
            notional: 1_000_000.0,
            coupon_rate: 0.08,
            maturity: 5.0,
        }
    }
}

impl SpaStructuredConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn notional(mut self, n: f64) -> Self { self.notional = n.max(0.0); self }
    pub fn coupon_rate(mut self, r: f64) -> Self { self.coupon_rate = r.max(0.0); self }
    pub fn maturity(mut self, m: f64) -> Self { self.maturity = m.max(0.1); self }
}

fn generate_cash_flows(cfg: &SpaStructuredConfig) -> Vec<CashFlow> {
    let mut flows = Vec::new();
    let coupon = cfg.notional * cfg.coupon_rate;
    let n_coupons = cfg.maturity as usize;
    for i in 1..=n_coupons {
        flows.push(CashFlow::new(i as f64, coupon, "Coupon"));
    }
    flows.push(CashFlow::new(cfg.maturity, cfg.notional, "Principal"));
    flows
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    cfg: &SpaStructuredConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    fill_background(&root, cfg.theme)?;

    let flows = generate_cash_flows(cfg);
    let max_amount = flows.iter().map(|f| f.amount).fold(0.0_f64, f64::max);
    let t_max = cfg.maturity * 1.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — Notional: {:.0}", cfg.title, cfg.notional),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(80)
        .build_cartesian_2d(0.0..t_max, 0.0..max_amount * 1.1)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .x_desc("Time (years)")
        .y_desc("Amount")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    for flow in &flows {
        let color = if flow.flow_type == "Principal" {
            cfg.theme.accent()
        } else {
            cfg.theme.info()
        };
        chart
            .draw_series(std::iter::once(Rectangle::new(
                [(flow.time - 0.15, 0.0), (flow.time + 0.15, flow.amount)],
                color.filled(),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;

        chart
            .draw_series(std::iter::once(Text::new(
                format!("{:.0}", flow.amount),
                (flow.time - 0.3, flow.amount + max_amount * 0.02),
                (LABEL_FONT, 10).into_font().color(&cfg.theme.text()),
            )))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(cfg: &SpaStructuredConfig, path: &str) -> Result<()> {
    render(png_root(path)?, cfg)
}

pub fn render_svg(cfg: &SpaStructuredConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let cfg = SpaStructuredConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_spa_structured.png").to_str().unwrap().to_string();
        render_png(&cfg, &path).unwrap();
    }
}
