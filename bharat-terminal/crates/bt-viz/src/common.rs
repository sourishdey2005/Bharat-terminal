//! Shared rendering helpers used by every chart. Made by Sourish Dey.

use std::sync::Once;

use bt_core::{BtError, Result, APP_NAME, AUTHOR};
use plotters::coord::Shift;
use plotters::prelude::*;
use plotters::style::register_font;
use plotters_bitmap::BitMapBackend;

use crate::palette::Theme;

static FONTS_REGISTERED: Once = Once::new();

const DEJAVU_SANS: &[u8] = include_bytes!("../../../assets/fonts/DejaVuSans.ttf");
const DEJAVU_SANS_BOLD: &[u8] = include_bytes!("../../../assets/fonts/DejaVuSans-Bold.ttf");
const DEJAVU_SANS_MONO: &[u8] = include_bytes!("../../../assets/fonts/DejaVuSansMono.ttf");

/// Registers the bundled fonts with plotters' embedded (`ab_glyph`) text
/// renderer exactly once. Because the fonts ship inside the binary, chart
/// export never depends on what fonts happen to be installed on the host.
pub fn ensure_fonts_registered() {
    FONTS_REGISTERED.call_once(|| {
        let _ = register_font(TITLE_FONT, FontStyle::Normal, DEJAVU_SANS);
        let _ = register_font(TITLE_FONT, FontStyle::Bold, DEJAVU_SANS_BOLD);
        let _ = register_font(LABEL_FONT, FontStyle::Normal, DEJAVU_SANS_MONO);
    });
}

/// Output format for a chart render.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Png,
    Svg,
}

impl Format {
    pub fn extension(&self) -> &'static str {
        match self {
            Format::Png => "png",
            Format::Svg => "svg",
        }
    }
}

/// Standard canvas size for every Bharat Terminal chart.
pub const WIDTH: u32 = 1200;
pub const HEIGHT: u32 = 700;

/// Font family used for chart titles/labels. The desktop/web app ships
/// with Inter, JetBrains Mono and Fira Code as design-system fonts (see
/// assets/fonts and bt-ui/theme.rs); static PNG/SVG export instead uses
/// widely-available system fonts so rendering never depends on which
/// fonts happen to be installed on the machine doing the export.
pub const TITLE_FONT: &str = "DejaVu Sans";
pub const LABEL_FONT: &str = "DejaVu Sans Mono";

/// Draws the "Made by Sourish Dey" footer + BHARAT TERMINAL wordmark that
/// must appear on every exported chart, per the branding spec.
pub fn draw_footer<'a, DB: DrawingBackend + 'a>(
    root: &DrawingArea<DB, Shift>,
    theme: Theme,
) -> Result<()> {
    let (w, h) = root.dim_in_pixel();
    let footer_h = 26i32;
    let y0 = h as i32 - footer_h;

    root.draw(&Rectangle::new(
        [(0, y0), (w as i32, h as i32)],
        theme.border().filled(),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    root.draw(&Text::new(
        format!("{APP_NAME} — Made by {AUTHOR}"),
        (10, y0 + 5),
        (LABEL_FONT, 14).into_font().color(&theme.accent()),
    ))
    .map_err(|e| BtError::Render(e.to_string()))?;

    Ok(())
}

/// Fills the whole canvas with the theme background before any chart
/// content is drawn (plotters draws on white by default otherwise).
pub fn fill_background<'a, DB: DrawingBackend + 'a>(
    root: &DrawingArea<DB, Shift>,
    theme: Theme,
) -> Result<()> {
    root.fill(&theme.background())
        .map_err(|e| BtError::Render(e.to_string()))
}

/// Opens a bitmap-backed drawing root at the standard chart size.
pub fn png_root(path: &str) -> Result<DrawingArea<BitMapBackend<'_>, Shift>> {
    ensure_fonts_registered();
    let backend = BitMapBackend::new(path, (WIDTH, HEIGHT));
    Ok(backend.into_drawing_area())
}

/// Opens an SVG-backed drawing root at the standard chart size.
pub fn svg_root(path: &str) -> Result<DrawingArea<SVGBackend<'_>, Shift>> {
    ensure_fonts_registered();
    let backend = SVGBackend::new(path, (WIDTH, HEIGHT));
    Ok(backend.into_drawing_area())
}
