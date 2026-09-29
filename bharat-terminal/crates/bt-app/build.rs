//! Build script: embeds the Bharat Terminal icon and version metadata into
//! the desktop app executable. The GUI hides its console window in release
//! builds (see `windows_subsystem` in `main.rs`).

#[path = "../icon_build.rs"]
mod icon_build;

fn main() {
    icon_build::run(
        "Bharat Terminal - Bloomberg power. Zero cost. Made in India.",
        "bt-app",
        "bt-app.exe",
    );
}
