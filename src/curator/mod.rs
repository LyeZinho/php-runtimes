pub mod discover;
pub mod download;
pub mod purify;
pub mod validate;
pub mod package;
pub mod manifest;
pub mod source;
pub mod build;

use anyhow::{Context, Result};
use tracing::{info, warn};

pub async fn run_curation() -> Result<()> {
    info!("Starting PHP runtime curation...");

    let latest_version = discover::check_latest_version().await?;
    info!("Latest PHP version available: {}", latest_version);

    let manifest = manifest::load_manifest()?;
    if manifest.versions.contains_key(&latest_version) {
        info!("Version {} already curated. Skipping.", latest_version);
        return Ok(());
    }

    info!("New version detected! Curating {}...", latest_version);
    
    let temp_dir = tempfile::tempdir()?;
    let download_path = temp_dir.path().join("php.tar.gz");
    
    download::download_binary(&latest_version, &download_path).await?;
    
    let extract_path = temp_dir.path().join("extracted");
    package::extract_tar_gz(&download_path, extract_path.as_path())?;
    
    let php_binary = extract_path.join("php");
    purify::strip_binary(php_binary.as_path())?;
    
    validate::verify_extensions(php_binary.as_path())?;
    
    let checksum = download::compute_sha256(php_binary.as_path())?;
    info!("SHA256: {}", checksum);

    manifest::add_version(&latest_version, &checksum)?;
    
    info!("Curation complete for PHP {}", latest_version);
    Ok(())
}

pub async fn sync_all() -> Result<()> {
    info!("Syncing all PHP versions...");

    let versions = discover::get_all_versions().await?;
    let manifest = manifest::load_manifest()?;
    let existing: Vec<_> = manifest.versions.keys().cloned().collect();

    let mut new_count = 0;
    let mut skipped_count = 0;

    for version in versions {
        if existing.contains(&version) {
            info!("Skipping {} (already curated)", version);
            skipped_count += 1;
            continue;
        }

        info!("Curating {}...", version);
        
        let temp_dir = tempfile::tempdir()?;
        let download_path = temp_dir.path().join("php.tar.gz");
        
        match download::download_binary(&version, &download_path).await {
            Ok(_) => {}
            Err(e) => {
                warn!("Failed to download {}: {}", version, e);
                continue;
            }
        }
        
        let extract_path = temp_dir.path().join("extracted");
        if let Err(e) = package::extract_tar_gz(&download_path, extract_path.as_path()) {
            warn!("Failed to extract {}: {}", version, e);
            continue;
        }
        
        let php_binary = extract_path.join("php");
        if !php_binary.exists() {
            warn!("PHP binary not found for {}", version);
            continue;
        }
        
        purify::strip_binary(php_binary.as_path())?;
        
        if let Err(e) = validate::verify_extensions(php_binary.as_path()) {
            warn!("Validation failed for {}: {}", version, e);
            continue;
        }
        
        let checksum = match download::compute_sha256(php_binary.as_path()) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to compute checksum for {}: {}", version, e);
                continue;
            }
        };

        if let Err(e) = manifest::add_version(&version, &checksum) {
            warn!("Failed to update manifest for {}: {}", version, e);
            continue;
        }
        
        info!("Successfully curated {}", version);
        new_count += 1;
    }

    info!("Sync complete! {} new versions, {} skipped", new_count, skipped_count);
    Ok(())
}

pub async fn download_sources(base_path: &std::path::Path) -> Result<()> {
    info!("Downloading PHP sources...");

    let versions = source::fetch_available_versions().await?;
    let manifest_path = base_path.join("sources").join("manifest.json");
    let sources_dir = base_path.join("sources");
    
    std::fs::create_dir_all(&sources_dir)?;

    let mut manifest = source::SourceManifest::load(&manifest_path)?;
    let existing: Vec<_> = manifest.sources.iter().map(|s| s.version.clone()).collect();

    let mut new_count = 0;

    for version in versions {
        if existing.contains(&version) {
            info!("Source {} already downloaded, skipping", version);
            continue;
        }

        match source::download_source(&version, &sources_dir, &mut manifest).await {
            Ok(path) => {
                info!("Downloaded source: {:?}", path);
                new_count += 1;
            }
            Err(e) => {
                warn!("Failed to download source {}: {}", version, e);
            }
        }

        if let Err(e) = manifest.save(&manifest_path) {
            warn!("Failed to save manifest: {}", e);
        }
    }

    info!("Download complete! {} new sources", new_count);
    Ok(())
}

pub fn list_sources(base_path: &std::path::Path) -> Result<Vec<source::SourceInfo>> {
    let manifest_path = base_path.join("sources").join("manifest.json");
    let manifest = source::SourceManifest::load(&manifest_path)?;
    Ok(manifest.sources)
}

pub fn build_single(
    base_path: &std::path::Path,
    version: &str,
    platform: build::Platform,
) -> Result<build::BuildResult> {
    use build::*;

    info!("Building PHP {} for platform {:?}", version, platform);

    let config = BuildConfig::new(base_path);
    let archive_path = config.source_archive(version);

    if !archive_path.exists() {
        anyhow::bail!("Source archive not found: {:?}", archive_path);
    }

    let temp_dir = config.temp_dir(platform);
    std::fs::create_dir_all(&temp_dir)?;

    let source_dir = extract_source(&archive_path, &temp_dir)
        .context("Failed to extract source")?;

    clean_bloat(&source_dir).ok();

    run_configure(&source_dir, platform).context("Configure failed")?;

    let php_binary = run_make(&source_dir, platform).context("Make failed")?;

    let _ = strip_binary(&php_binary).ok();

    verify_binary(&php_binary).context("Binary verification failed")?;

    let output_dir = config.output_dir(platform);
    let final_path = copy_to_output(&php_binary, &output_dir, version)?;

    let sha256 = compute_sha256(&final_path)?;
    let size_bytes = std::fs::metadata(&final_path)?.len();

    let result = BuildResult {
        version: version.to_string(),
        platform: platform.as_str().to_string(),
        status: BuildStatus::Success,
        output_path: Some(final_path.to_string_lossy().to_string()),
        sha256: Some(sha256),
        size_bytes,
        built_at: chrono::Utc::now().to_rfc3339(),
    };

    cleanup_build_dir(&temp_dir).ok();

    let manifest_path = base_path.join("builds").join("manifest.json");
    let mut manifest = BuildManifest::load(&manifest_path).unwrap_or_default();
    manifest.add_build(result.clone());
    manifest.save(&manifest_path).ok();

    info!("Build complete! {} bytes, SHA256: {}", result.size_bytes, result.sha256.as_ref().unwrap_or(&String::new()));

    Ok(result)
}

pub fn build_all(
    base_path: &std::path::Path,
    platforms: Vec<build::Platform>,
) -> Result<Vec<build::BuildResult>> {
    let sources = list_sources(base_path)?;
    let mut results = Vec::new();

    let stable_versions: Vec<_> = sources
        .iter()
        .filter(|s| {
            !s.version.contains("RC")
                && !s.version.contains("alpha")
                && !s.version.contains("beta")
        })
        .collect();

    for platform in platforms {
        for source in &stable_versions {
            match build_single(base_path, &source.version, platform) {
                Ok(result) => results.push(result),
                Err(e) => warn!("Failed to build {} for {:?}: {}", source.version, platform, e),
            }
        }
    }

    Ok(results)
}

pub fn check_deps(platform: build::Platform) -> Result<()> {
    build::check_dependencies(platform)
}
