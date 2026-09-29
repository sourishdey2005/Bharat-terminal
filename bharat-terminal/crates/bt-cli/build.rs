//! Build script: embeds the Bharat Terminal icon and version metadata into
//! the CLI executable. The CLI keeps its console window.

#[path = "../icon_build.rs"]
mod icon_build;

fn main() {
    icon_build::run(
        "Bharat Terminal CLI - market-data visualizations in your terminal.",
        "bt-cli",
        "bt-cli.exe",
    );
}
