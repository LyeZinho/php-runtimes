use anyhow::{Context, Result};
use serde::Deserialize;
use tracing::info;

const BASE_URL: &str = "https://dl.static-php.dev/static-php-cli/bulk";

#[derive(Debug, Deserialize)]
struct DirectoryEntry {
    name: String,
}

pub async fn check_latest_version() -> Result<String> {
    info!("Fetching PHP versions from dl.static-php.dev...");

    let client = reqwest::Client::new();
    let url = format!("{}/?format=json", BASE_URL);

    let response = client
        .get(&url)
        .header("User-Agent", "php-curator")
        .send()
        .await
        .with_context(|| format!("Failed to fetch from {}", url))?;

    let entries: Vec<DirectoryEntry> = response.json().await?;

    let mut versions: Vec<String> = entries
        .iter()
        .filter_map(|e| {
            let name = &e.name;
            if name.starts_with("php-") && name.contains("-cli-linux-x86_64.tar.gz") {
                name.strip_prefix("php-")
                    .and_then(|s| s.strip_suffix("-cli-linux-x86_64.tar.gz"))
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();

    versions.sort_by(|a, b| {
        let parts_a: Vec<u32> = a.split('.').filter_map(|s| s.parse().ok()).collect();
        let parts_b: Vec<u32> = b.split('.').filter_map(|s| s.parse().ok()).collect();
        
        for (pa, pb) in parts_a.iter().zip(parts_b.iter()) {
            if pa != pb {
                return pb.cmp(pa);
            }
        }
        parts_b.len().cmp(&parts_a.len())
    });

    let latest = versions.first()
        .ok_or_else(|| anyhow::anyhow!("No PHP versions found"))?
        .clone();

    info!("Latest PHP version: {}", latest);
    Ok(latest)
}

pub async fn get_all_versions() -> Result<Vec<String>> {
    let client = reqwest::Client::new();
    let url = format!("{}/?format=json", BASE_URL);

    let response = client
        .get(&url)
        .header("User-Agent", "php-curator")
        .send()
        .await
        .with_context(|| format!("Failed to fetch from {}", url))?;

    let entries: Vec<DirectoryEntry> = response.json().await?;

    let mut versions: Vec<String> = entries
        .iter()
        .filter_map(|e| {
            let name = &e.name;
            if name.starts_with("php-") && name.contains("-cli-linux-x86_64.tar.gz") {
                name.strip_prefix("php-")
                    .and_then(|s| s.strip_suffix("-cli-linux-x86_64.tar.gz"))
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();

    versions.sort_by(|a, b| {
        let parts_a: Vec<u32> = a.split('.').filter_map(|s| s.parse().ok()).collect();
        let parts_b: Vec<u32> = b.split('.').filter_map(|s| s.parse().ok()).collect();
        
        for (pa, pb) in parts_a.iter().zip(parts_b.iter()) {
            if pa != pb {
                return pb.cmp(pa);
            }
        }
        parts_b.len().cmp(&parts_a.len())
    });

    Ok(versions)
}

#[allow(dead_code)]
pub fn get_download_url(version: &str, os: &str, arch: &str) -> String {
    let suffix = match (os, arch) {
        ("linux", "x86_64") => "cli-linux-x86_64.tar.gz",
        ("linux", "aarch64") => "cli-linux-aarch64.tar.gz",
        ("macos", "x86_64") => "cli-macos-x86_64.tar.gz",
        ("macos", "aarch64") => "cli-macos-aarch64.tar.gz",
        _ => "cli-linux-x86_64.tar.gz",
    };

    format!("{}/php-{}-{}", BASE_URL, version, suffix)
}

#[allow(dead_code)]
pub fn parse_version(version: &str) -> (String, String, String) {
    let parts: Vec<&str> = version.split('-').collect();
    let php_version = parts.get(1).unwrap_or(&version).to_string();

    let os = if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "unknown"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "unknown"
    };

    (php_version, os.to_string(), arch.to_string())
}
