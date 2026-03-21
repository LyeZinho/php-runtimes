use anyhow::{Context, Result};
use std::process::Command;
use tracing::{info, warn};

const ESSENTIAL_EXTENSIONS: &[&str] = &["Core", "date", "libxml", "openssl", "pcre"];

const RECOMMENDED_EXTENSIONS: &[&str] = &[
    "mbstring",
    "pdo",
    "pdo_mysql",
    "curl",
    "zip",
    "gd",
    "json",
    "session",
];

pub fn verify_extensions(php_binary: &std::path::Path) -> Result<Vec<String>> {
    info!("Validating PHP extensions...");

    let output = Command::new(php_binary)
        .arg("-m")
        .output()
        .context("Failed to run php -m")?;

    if !output.status.success() {
        warn!("PHP binary exited with non-zero status");
    }

    let modules = String::from_utf8_lossy(&output.stdout);
    let modules_lower = modules.to_lowercase();

    let mut found = Vec::new();
    let mut missing = Vec::new();

    for ext in ESSENTIAL_EXTENSIONS {
        if modules_lower.contains(&ext.to_lowercase()) {
            found.push(ext.to_string());
        } else {
            missing.push(ext.to_string());
        }
    }

    if !missing.is_empty() {
        warn!("Missing essential extensions: {:?}", missing);
    }

    let mut recommended_found = 0;
    for ext in RECOMMENDED_EXTENSIONS {
        if modules_lower.contains(&ext.to_lowercase()) {
            recommended_found += 1;
        }
    }

    info!(
        "Extension check: {}/{} essential found, {}/{} recommended found",
        found.len(),
        ESSENTIAL_EXTENSIONS.len(),
        recommended_found,
        RECOMMENDED_EXTENSIONS.len()
    );

    Ok(found)
}

#[allow(dead_code)]
pub fn check_extension(php_binary: &std::path::Path, extension: &str) -> Result<bool> {
    let output = Command::new(php_binary)
        .args([
            "-r",
            &format!("echo extension_loaded('{}') ? 'yes' : 'no';", extension),
        ])
        .output()?;

    let result = String::from_utf8_lossy(&output.stdout);
    Ok(result.trim() == "yes")
}
