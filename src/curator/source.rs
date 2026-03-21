use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::io::Write;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub version: String,
    pub filename: String,
    pub url: String,
    pub size_bytes: u64,
    pub downloaded: bool,
    pub extracted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourceManifest {
    pub sources: Vec<SourceInfo>,
}

impl SourceManifest {
    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let manifest: SourceManifest = serde_json::from_str(&content)?;
            Ok(manifest)
        } else {
            Ok(SourceManifest::default())
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

pub fn get_source_url(version: &str) -> String {
    format!("https://www.php.net/distributions/php-{}.tar.xz", version)
}

pub fn get_source_filename(version: &str) -> String {
    format!("php-{}.tar.xz", version)
}

pub async fn fetch_available_versions() -> Result<Vec<String>> {
    info!("Fetching PHP versions from GitHub...");
    
    let client = reqwest::Client::new();
    let url = "https://api.github.com/repos/php/php-src/tags?per_page=100";
    
    let response = client
        .get(url)
        .header("User-Agent", "php-curator")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .context("Failed to fetch PHP releases from GitHub")?;

    #[derive(Deserialize)]
    struct Tag {
        name: String,
    }
    
    let tags: Vec<Tag> = response.json().await?;

    let mut versions: Vec<String> = tags
        .iter()
        .filter_map(|tag| {
            let name = &tag.name;
            if name.starts_with("php-") {
                Some(name.strip_prefix("php-").unwrap().to_string())
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

    info!("Found {} PHP versions", versions.len());
    Ok(versions)
}

pub async fn download_source(
    version: &str,
    output_dir: &Path,
    manifest: &mut SourceManifest,
) -> Result<PathBuf> {
    let filename = get_source_filename(version);
    let url = get_source_url(version);
    let output_path = output_dir.join(&filename);

    if output_path.exists() {
        info!("Source {} already exists, skipping download", version);
        return Ok(output_path);
    }

    info!("Downloading PHP {} source from {}", version, url);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "php-curator")
        .send()
        .await
        .with_context(|| format!("Failed to download from {}", url))?;

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let file = std::fs::File::create(&output_path)?;
    let mut writer = std::io::BufWriter::new(file);

    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        std::io::Write::write_all(&mut writer, &chunk)?;
        downloaded += chunk.len() as u64;
        if downloaded % (5 * 1024 * 1024) == 0 {
            info!("Downloaded {} / {} bytes", downloaded, total_size);
        }
    }

    writer.flush()?;

    let source_info = SourceInfo {
        version: version.to_string(),
        filename,
        url,
        size_bytes: downloaded,
        downloaded: true,
        extracted: false,
    };

    if let Some(existing) = manifest.sources.iter_mut().find(|s| s.version == version) {
        *existing = source_info;
    } else {
        manifest.sources.push(source_info);
    }

    info!("Downloaded {} ({} bytes)", version, downloaded);
    Ok(output_path)
}
