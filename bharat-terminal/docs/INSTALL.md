# Bharat Terminal v3 — Installation Guide

**Author:** Sourish Dey  
**Version:** 3.0.0

---

## System Requirements

- Windows 10/11 (64-bit)
- 4 GB RAM (8 GB recommended)
- 500 MB disk space
- Internet connection for real-time data

---

## Installation Options

### Option 1: Portable (Recommended)

1. Download `BharatTerminal-v3.0.0-portable.zip` from [GitHub Releases](https://github.com/sourishdey/bharat-terminal/releases)
2. Extract to any folder (e.g., `C:\BharatTerminal`)
3. Double-click `bt-app.exe`

### Option 2: MSI Installer

1. Download `Bharat Terminal.msi` from [GitHub Releases](https://github.com/sourishdey/bharat-terminal/releases)
2. Run the installer
3. Follow the wizard (Next → Install → Finish)
4. Launch from Start Menu or Desktop shortcut

### Option 3: Inno Setup

1. Download `BharatTerminal-v3.0.0-Setup.exe` from [GitHub Releases](https://github.com/sourishdey/bharat-terminal/releases)
2. Run the installer
3. Follow the wizard

---

## Building from Source

### Prerequisites

1. Install [Rust](https://rustup.rs/) (rustc 1.98.1+)
2. Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (MSVC)

### Build Commands

```powershell
# Clone the repository
git clone https://github.com/sourishdey/bharat-terminal.git
cd bharat-terminal

# Build entire workspace
cargo build --release --workspace

# Run tests
cargo test --workspace

# Launch the app
cargo run --release -p bt-app

# Or use the built executable directly
target\release\bt-app.exe
```

### Creating the Installer

```powershell
# Run the build script
powershell -ExecutionPolicy Bypass -File scripts\build-installer.ps1

# Or manually:
cargo install cargo-bundle
cargo bundle --release -p bt-app
# Output: target\release\bundle\windows\Bharat Terminal.msi
```

---

## First Run

1. On first run, the app creates:
   - `./data/` — Cache directory
   - `./data/cache.db` — SQLite cache
   - `./data/prefs.json` — User preferences

2. The app automatically fetches data for **RELIANCE.NS** (1 year daily)

3. Use the **Company** dropdown to switch to any of 78 companies

4. Use **1D/1W/1M/3M/6M/1Y/5Y** buttons to change time range

5. Toggle **Live \*** for 30-second auto-refresh

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| `cargo not found` | Run `set PATH=%PATH%;%USERPROFILE%\.cargo\bin` |
| `link.exe not found` | Restart terminal; run `vcvarsall.bat` |
| Blank charts | Click a tab; data may still be loading |
| Network errors | Check internet; app falls back to synthetic data |
| `cache.db` locked | Close other instances; delete `data\cache.db` |
| App won't start | Delete `data\prefs.json` and retry |

---

## Uninstallation

### Portable
Delete the extracted folder.

### Installed
1. Open Settings → Apps → Bharat Terminal → Uninstall
2. Or run `Uninstall.exe` from the install directory
3. Or use `apps.cpl` → Bharat Terminal → Uninstall

To fully remove all data:
```
rmdir /s %APPDATA%\BharatTerminal
rmdir /s %USERPROFILE%\.bharat-terminal
```

---

**Bharat Terminal v3 — Made by Sourish Dey**
*Bloomberg power. Zero cost. Made in India.*
