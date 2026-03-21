use anyhow::{Context, Result};
use sha2::{Sha256, Digest};
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use tracing::info;
use futures_util::StreamExt;

const BASE_URL: &str = "https://dl.static-php.dev/static-php-cli/bulk";

pub async fn download_binary(version: &str, output_path: &Path) -> Result<()> {
    let client = reqwest::Client::new();
    
    let os = if cfg!(target_os = "linux") { "linux" } else { "linux" };
    let arch = if cfg!(target_arch = "x86_64") { "x86_64" } else { "x86_64" };
    
    let filename = format!("php-{}-cli-{}-{}.tar.gz", version, os, arch);
    let url = format!("{}/{}", BASE_URL, filename);
    
    info!("Downloading {}...", url);
    
    let response = client
        .get(&url)
        .header("User-Agent", "php-curator")
        .send()
        .await
        .with_context(|| format!("Failed to download from {}", url))?;

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);
    
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        writer.write_all(&chunk)?;
        downloaded += chunk.len() as u64;
        info!("Downloaded {}/{} bytes", downloaded, total_size);
    }
    
    writer.flush()?;
    info!("Download complete: {} bytes", downloaded);
    
    Ok(())
}

pub fn compute_sha256(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    Ok(hex::encode(hasher.finalize()))
}

#[allow(dead_code)]
pub fn verify_checksum(path: &Path, expected: &str) -> Result<bool> {
    let actual = compute_sha256(path)?;
    let expected_clean = expected.trim_start_matches("0x");
    Ok(actual.to_lowercase() == expected_clean.to_lowercase())
}
