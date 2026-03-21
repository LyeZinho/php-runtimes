use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use tracing::{info, warn};

pub fn strip_binary(path: &Path) -> Result<()> {
    info!("Stripping debug symbols from {:?}", path);

    if !path.exists() {
        warn!("Binary not found at {:?}, skipping strip", path);
        return Ok(());
    }

    let status = Command::new("strip")
        .arg("--strip-all")
        .arg(path)
        .status()
        .context("Failed to execute strip command")?;

    if !status.success() {
        warn!("strip command returned non-zero exit code");
    }

    let metadata = fs::metadata(path)?;
    let original_size = metadata.len();

    info!("Binary stripped. Final size: {} bytes", original_size);
    Ok(())
}

#[allow(dead_code)]
pub fn remove_unnecessary_files(dir: &Path) -> Result<u64> {
    let mut freed: u64 = 0;

    let patterns = ["*.h", "*.hpp", "*.c", "*.o", "*.phpt", "*.md", "test*"];

    for pattern in patterns {
        let glob_pattern = dir.join(pattern);
        if let Ok(entries) = glob::glob(&glob_pattern.to_string_lossy()) {
            for entry in entries.flatten() {
                if let Ok(metadata) = fs::metadata(&entry) {
                    freed += metadata.len();
                    let _ = fs::remove_file(&entry);
                    info!("Removed: {:?}", entry);
                }
            }
        }
    }

    info!("Freed {} bytes by removing unnecessary files", freed);
    Ok(freed)
}
