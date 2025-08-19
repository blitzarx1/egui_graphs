use std::process::Command;

fn main() {
    // Re-run if the Git HEAD changes
    println!("cargo:rerun-if-changed=.git/HEAD");

    // Try to get a descriptive tag or short hash
    let describe = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(o.stdout)
            } else {
                None
            }
        })
        .and_then(|b| String::from_utf8(b).ok())
        .map(|s| s.trim().to_string());

    if let Some(d) = describe {
        println!("cargo:rustc-env=DEMO_GIT_DESCRIBE={}", d);
    }

    let short = Command::new("git")
        .args(["rev-parse", "--short=9", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(o.stdout)
            } else {
                None
            }
        })
        .and_then(|b| String::from_utf8(b).ok())
        .map(|s| s.trim().to_string());

    if let Some(s) = short {
        println!("cargo:rustc-env=DEMO_GIT_HASH={}", s);
    }

    // Generate assets manifest for Import/Load tab (compile-time include for web)
    use std::{env, fs, io::Write, path::PathBuf};
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let assets_dir = PathBuf::from(&manifest_dir).join("assets");
    println!("cargo:rerun-if-changed={}", assets_dir.display());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_file = out_dir.join("assets_manifest.rs");

    let mut entries: Vec<String> = Vec::new();
    if let Ok(read_dir) = fs::read_dir(&assets_dir) {
        for e in read_dir.flatten() {
            let path = e.path();
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ext.eq_ignore_ascii_case("json") {
                    if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                        // Use include_str! with absolute path to asset
                        let abs = path.to_string_lossy().replace('"', "\\\"");
                        entries.push(format!(
                            "(\"{name}\", include_str!(\"{abs}\"))",
                            name = file_name,
                            abs = abs,
                        ));
                        println!("cargo:rerun-if-changed={}", path.display());
                    }
                }
            }
        }
    }
    entries.sort();
    let content = format!(
        "pub static ASSETS: &[(&str, &str)] = &[\n    {}\n];\n",
        entries.join(",\n    ")
    );
    if let Ok(mut f) = fs::File::create(&out_file) {
        let _ = f.write_all(content.as_bytes());
    }
}
