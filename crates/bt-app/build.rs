// crates/bt-app/build.rs
// Author: Sourish Dey

fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icons/icon.ico");
        res.set("ProductName", "Bharat Terminal");
        res.set("FileDescription", "Bloomberg power. Zero cost. Made in India.");
        res.set("CompanyName", "Sourish Dey");
        res.set("LegalCopyright", "Copyright (c) 2026 Sourish Dey");
        res.set("ProductVersion", "3.0.0");
        res.compile().unwrap();
    }
}
