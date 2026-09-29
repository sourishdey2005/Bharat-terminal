//! Shared helper for embedding the Bharat Terminal icon and version metadata
//! into the Windows executables (Explorer icon, taskbar, title bar, Details
//! tab). Used by the `build.rs` of `bt-app` and `bt-cli`.
//!
//! Uses `rc.exe` from the Windows SDK when it is not already on `PATH`
//! (GitHub's runners have it; a bare local toolchain may only have the SDK).

pub fn run(file_description: &str, internal_name: &str, original_filename: &str) {
    let manifest = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let icon = manifest.join("../../assets/icons/app.ico");
    assert!(
        icon.exists(),
        "missing application icon: {}",
        icon.display()
    );

    // Make sure a resource compiler can be found.
    if std::process::Command::new("rc.exe")
        .arg("/?")
        .output()
        .is_err()
    {
        if let Some(dir) = find_sdk_rc_dir() {
            let path = std::env::var("PATH").unwrap_or_default();
            std::env::set_var("PATH", format!("{};{}", dir.display(), path));
        }
    }

    // Generate the .rc in OUT_DIR so the icon path can be absolute: the
    // resource compiler resolves relative paths against its own working
    // directory, which is not guaranteed to be the package root.
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".into());
    let (major, minor, patch) = parse_version(&version);
    let icon_str = icon.display().to_string().replace('\\', "\\\\");
    let rc = format!(
        "1 ICON \"{icon_str}\"\n\
         1 VERSIONINFO\n\
         FILEVERSION {major},{minor},{patch},0\n\
         PRODUCTVERSION {major},{minor},{patch},0\n\
         FILEFLAGSMASK 0x3fL\n\
         FILEFLAGS 0x0L\n\
         FILEOS 0x40004L\n\
         FILETYPE 0x1L\n\
         FILESUBTYPE 0x0L\n\
         BEGIN\n\
             BLOCK \"StringFileInfo\"\n\
             BEGIN\n\
                 BLOCK \"040904b0\"\n\
                 BEGIN\n\
                     VALUE \"CompanyName\", \"Sourish Dey\"\n\
                     VALUE \"FileDescription\", \"{file_description}\"\n\
                     VALUE \"FileVersion\", \"{version}\"\n\
                     VALUE \"InternalName\", \"{internal_name}\"\n\
                     VALUE \"LegalCopyright\", \"Copyright (c) 2026 Sourish Dey (MIT)\"\n\
                     VALUE \"OriginalFilename\", \"{original_filename}\"\n\
                     VALUE \"ProductName\", \"Bharat Terminal\"\n\
                     VALUE \"ProductVersion\", \"{version}\"\n\
                 END\n\
             END\n\
             BLOCK \"VarFileInfo\"\n\
             BEGIN\n\
                 VALUE \"Translation\", 0x409, 1200\n\
             END\n\
         END\n"
    );
    let rc_path = out.join("app.rc");
    std::fs::write(&rc_path, rc).expect("write app.rc");

    embed_resource::compile(&rc_path, embed_resource::NONE);

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed={}", icon.display());
}

/// Newest `rc.exe` shipped with any installed Windows 10/11 SDK, if any.
fn find_sdk_rc_dir() -> Option<std::path::PathBuf> {
    let kits = std::path::PathBuf::from(r"C:\Program Files (x86)\Windows Kits\10\bin");
    let entries = std::fs::read_dir(kits).ok()?;
    let mut best: Option<std::path::PathBuf> = None;
    for entry in entries.flatten() {
        let dir = entry.path().join("x64");
        if dir.join("rc.exe").exists() && best.as_ref().map_or(true, |b| dir > *b) {
            best = Some(dir);
        }
    }
    best
}

fn parse_version(version: &str) -> (u16, u16, u16) {
    let mut parts = version.split('.').map(|p| p.parse::<u16>().unwrap_or(0));
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}
