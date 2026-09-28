//! Color palettes for Bharat Terminal charts. Made by Sourish Dey.
//!
//! Every visualization picks its base colors from here so themes stay
//! consistent across the 10 chart types while still giving each chart
//! its own accent identity.

use plotters::style::RGBColor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}

/// The terminal-wide amber/black identity (used in headers & footers).
pub const AMBER: RGBColor = RGBColor(0xFF, 0xB0, 0x00);
pub const BLACK: RGBColor = RGBColor(0x00, 0x00, 0x00);
pub const PANEL_DARK: RGBColor = RGBColor(0x0A, 0x0A, 0x0A);
pub const BORDER_DARK: RGBColor = RGBColor(0x1A, 0x1A, 0x1A);
pub const PROFIT: RGBColor = RGBColor(0x00, 0xFF, 0x88);
pub const LOSS: RGBColor = RGBColor(0xFF, 0x3B, 0x3B);
pub const INFO: RGBColor = RGBColor(0x00, 0xBF, 0xFF);

pub const WHITE: RGBColor = RGBColor(0xFF, 0xFF, 0xFF);
pub const PANEL_LIGHT: RGBColor = RGBColor(0xF5, 0xF5, 0xF5);
pub const BORDER_LIGHT: RGBColor = RGBColor(0xE0, 0xE0, 0xE0);
pub const GOLD: RGBColor = RGBColor(0xB8, 0x86, 0x0B);
pub const PROFIT_LIGHT: RGBColor = RGBColor(0x00, 0x80, 0x00);
pub const LOSS_LIGHT: RGBColor = RGBColor(0xCC, 0x00, 0x00);
pub const INFO_LIGHT: RGBColor = RGBColor(0x00, 0x66, 0xCC);

impl Theme {
    pub fn background(&self) -> RGBColor {
        match self {
            Theme::Dark => PANEL_DARK,
            Theme::Light => WHITE,
        }
    }

    pub fn border(&self) -> RGBColor {
        match self {
            Theme::Dark => BORDER_DARK,
            Theme::Light => BORDER_LIGHT,
        }
    }

    pub fn accent(&self) -> RGBColor {
        match self {
            Theme::Dark => AMBER,
            Theme::Light => GOLD,
        }
    }

    pub fn profit(&self) -> RGBColor {
        match self {
            Theme::Dark => PROFIT,
            Theme::Light => PROFIT_LIGHT,
        }
    }

    pub fn loss(&self) -> RGBColor {
        match self {
            Theme::Dark => LOSS,
            Theme::Light => LOSS_LIGHT,
        }
    }

    pub fn info(&self) -> RGBColor {
        match self {
            Theme::Dark => INFO,
            Theme::Light => INFO_LIGHT,
        }
    }

    pub fn text(&self) -> RGBColor {
        match self {
            Theme::Dark => WHITE,
            Theme::Light => BLACK,
        }
    }

    /// A 5-color sequential palette unique to each chart, derived from a
    /// per-chart hue offset so every visualization looks distinct even
    /// while sharing the same design system.
    pub fn categorical(&self, hue_offset: u8) -> Vec<RGBColor> {
        let base: [(u8, u8, u8); 6] = [
            (0xFF, 0xB0, 0x00),
            (0x00, 0xBF, 0xFF),
            (0x00, 0xFF, 0x88),
            (0xFF, 0x3B, 0x3B),
            (0xB0, 0x66, 0xFF),
            (0xFF, 0x8A, 0x00),
        ];
        base.iter()
            .map(|(r, g, b)| {
                let shift = hue_offset as u32;
                RGBColor(
                    r.wrapping_add(shift as u8 / 3),
                    g.wrapping_add(shift as u8 / 5),
                    b.wrapping_add(shift as u8 / 7),
                )
            })
            .collect()
    }
}
