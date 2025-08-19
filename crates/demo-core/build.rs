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
}
