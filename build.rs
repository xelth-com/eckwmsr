fn main() {
    // Pass build timestamp as compile-time env var
    let output = std::process::Command::new("date")
        .args(["+%Y-%m-%dT%H:%M:%SZ"])
        .output();
    let now = match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Err(_) => "unknown".to_string(),
    };
    println!("cargo:rustc-env=BUILT_AT={}", now);
}
