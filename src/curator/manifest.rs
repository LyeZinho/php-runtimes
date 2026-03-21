use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::info;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Manifest {
    pub updated_at: String,
    pub source: String,
    pub versions: HashMap<String, VersionInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionInfo {
    pub released: String,
    pub status: String,
    #[serde(rename = "linux-x86_64")]
    pub linux_x86_64: Option<RuntimeAsset>,
    #[serde(rename = "linux-aarch64")]
    pub linux_aarch64: Option<RuntimeAsset>,
    #[serde(rename = "macos-arm64")]
    pub macos_arm64: Option<RuntimeAsset>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RuntimeAsset {
    pub url: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub extensions: Vec<String>,
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            updated_at: Utc::now().to_rfc3339(),
            source: "https://github.com/crazywhalecc/static-php-cli-bin".to_string(),
            versions: HashMap::new(),
        }
    }
}

pub fn load_manifest() -> Result<Manifest> {
    let manifest_path = Path::new("manifests/manifest.json");

    if manifest_path.exists() {
        let content = fs::read_to_string(manifest_path)?;
        let manifest: Manifest =
            serde_json::from_str(&content).context("Failed to parse manifest.json")?;
        info!(
            "Loaded existing manifest with {} versions",
            manifest.versions.len()
        );
        Ok(manifest)
    } else {
        info!("No existing manifest found, creating new one");
        Ok(Manifest::default())
    }
}

pub fn save_manifest(manifest: &Manifest) -> Result<()> {
    let manifest_path = Path::new("manifests/manifest.json");

    fs::create_dir_all(manifest_path.parent().unwrap_or(Path::new(".")))?;

    let content = serde_json::to_string_pretty(manifest).context("Failed to serialize manifest")?;

    fs::write(manifest_path, content)?;
    info!("Saved manifest to {:?}", manifest_path);
    Ok(())
}

pub fn add_version(version: &str, sha256: &str) -> Result<()> {
    let mut manifest = load_manifest()?;

    let version_info = VersionInfo {
        released: Utc::now().to_rfc3339(),
        status: "stable".to_string(),
        linux_x86_64: Some(RuntimeAsset {
            url: format!(
                "https://github.com/crazywhalecc/static-php-cli-bin/releases/download/php-{}/php-{}-cli-linux-x86_64.tar.gz",
                version, version
            ),
            sha256: sha256.to_string(),
            size_bytes: 0,
            extensions: vec![
                "pdo_mysql".to_string(),
                "mbstring".to_string(),
                "openssl".to_string(),
                "zip".to_string(),
                "gd".to_string(),
            ],
        }),
        linux_aarch64: None,
        macos_arm64: None,
    };

    manifest.versions.insert(version.to_string(), version_info);
    manifest.updated_at = Utc::now().to_rfc3339();

    save_manifest(&manifest)?;
    info!("Added version {} to manifest", version);
    Ok(())
}
