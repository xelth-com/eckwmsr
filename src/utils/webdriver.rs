use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

/// Get the current executable directory.
pub fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Start a process with no visible console window (Windows).
#[cfg(windows)]
pub fn start_hidden(exe: &Path, args: &[&str]) -> Result<std::process::Child> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    Command::new(exe)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .context("Failed to spawn hidden process")
}

#[cfg(not(windows))]
pub fn start_hidden(exe: &Path, args: &[&str]) -> Result<std::process::Child> {
    Command::new(exe)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("Failed to spawn process")
}

/// Ensure chromedriver is running on port 9515.
/// Checks if already bound, otherwise tries exe_dir then system PATH.
pub async fn ensure_chromedriver() -> Result<()> {
    if std::net::TcpStream::connect("127.0.0.1:9515").is_ok() {
        info!("WebDriver: ChromeDriver already running on port 9515");
        return Ok(());
    }

    let driver_name = if cfg!(windows) {
        "chromedriver.exe"
    } else {
        "chromedriver"
    };
    let driver_path = exe_dir().join(driver_name);

    if driver_path.exists() {
        match start_hidden(&driver_path, &["--port=9515", "--silent", "--log-level=OFF"]) {
            Ok(_) => {
                info!("WebDriver: Started chromedriver from {}", driver_path.display());
                sleep(Duration::from_secs(2)).await;
                return Ok(());
            }
            Err(e) => {
                error!("WebDriver: Failed to start chromedriver: {}", e);
                return Err(e);
            }
        }
    }

    warn!(
        "WebDriver: {} not found in {}, attempting system PATH...",
        driver_name,
        exe_dir().display()
    );
    match start_hidden(Path::new(driver_name), &["--port=9515", "--silent"]) {
        Ok(_) => {
            info!("WebDriver: Started chromedriver from system PATH");
            sleep(Duration::from_secs(2)).await;
            Ok(())
        }
        Err(e) => {
            let msg = format!("chromedriver not found. Please install it. Error: {}", e);
            error!("{}", msg);
            Err(anyhow::anyhow!(msg))
        }
    }
}
