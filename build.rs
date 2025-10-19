fn main() {
    // Only build on macOS
    if !std::env::var("TARGET").map_or(false, |t| t.contains("apple-darwin")) {
        return;
    }

    // Link against GTK4
    println!("cargo:rustc-link-lib=gtk-4.1");
    println!("cargo:rustc-link-search=/opt/homebrew/lib");
    println!("cargo:rustc-link-search=/opt/homebrew/Cellar/gtk4/4.20.2/lib");

    // Get Homebrew prefix dynamically
    if let Ok(output) = std::process::Command::new("brew")
        .arg("--prefix")
        .output()
    {
        if let Ok(homebrew_prefix) = String::from_utf8(output.stdout) {
            let lib_path = format!("{}/lib", homebrew_prefix.trim());
            println!("cargo:rustc-link-search={}", lib_path);
        }
    }

    // Link CoreGraphics framework
    println!("cargo:rustc-link-lib=framework=CoreGraphics");
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=AppKit");

    // Get the manifest directory (project root)
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let objc_file = format!("{}/macos_bridge.m", manifest_dir);
    
    // Compile the Objective-C bridge
    let mut build = cc::Build::new();
    build
        .file(&objc_file)
        .flag("-fobjc-arc")
        .flag("-fmodules")
        .flag("-Wno-ambiguous-macro") // Suppress the ambiguous macro warnings
        .include("/opt/homebrew/include/gtk-4.0")
        .include("/opt/homebrew/include/glib-2.0")
        .include("/opt/homebrew/lib/glib-2.0/include")
        .include("/opt/homebrew/include/pango-1.0")
        .include("/opt/homebrew/include/harfbuzz")
        .include("/opt/homebrew/include/cairo")
        .include("/opt/homebrew/include/gdk-pixbuf-2.0")
        .include("/opt/homebrew/include/graphene-1.0")
        .include("/opt/homebrew/lib/graphene-1.0/include");
    
    // Add additional include paths that might be needed
    if let Ok(output) = std::process::Command::new("pkg-config")
        .args(["--cflags", "gtk4"])
        .output() 
    {
        if let Ok(flags) = String::from_utf8(output.stdout) {
            for flag in flags.split_whitespace() {
                if flag.starts_with("-I") {
                    build.flag(flag);
                }
            }
        }
    }
    
    build.compile("macos_bridge");

    println!("cargo:rerun-if-changed={}", objc_file);
}