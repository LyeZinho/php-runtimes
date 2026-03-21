mod curator;

use clap::Parser;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber;

#[derive(Parser, Debug)]
#[command(name = "php-curator")]
#[command(about = "Automated PHP runtime curation and build system for CleanServe")]
enum Cli {
    Run,
    CheckVersion,
    SyncAll,
    DownloadSources {
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    ListSources {
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    Build {
        #[arg(long)]
        version: String,
        #[arg(long, default_value = "linux-x64")]
        platform: String,
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    BuildAll {
        #[arg(long, value_delimiter = ',', default_value = "linux-x64")]
        platforms: Vec<String>,
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    CheckDeps {
        #[arg(long, default_value = "linux-x64")]
        platform: String,
    },
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    match cli {
        Cli::Run => {
            info!("Starting PHP Curator...");
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(curator::run_curation())?;
        }
        Cli::CheckVersion => {
            let rt = tokio::runtime::Runtime::new()?;
            let version = rt.block_on(curator::discover::check_latest_version())?;
            println!("Latest PHP version: {}", version);
        }
        Cli::SyncAll => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(curator::sync_all())?;
        }
        Cli::DownloadSources { path } => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(curator::download_sources(&path))?;
        }
        Cli::ListSources { path } => {
            let sources = curator::list_sources(&path)?;
            println!("Available PHP sources:\n");
            for src in sources {
                println!(
                    "  {} - {} ({} bytes) {}",
                    src.version,
                    src.filename,
                    src.size_bytes,
                    if src.downloaded { "✓" } else { "✗" }
                );
            }
        }
        Cli::Build { version, platform, path } => {
            let p = match curator::build::Platform::from_str(&platform) {
                Some(p) => p,
                None => {
                    eprintln!("Unknown platform: {}", platform);
                    eprintln!("Available: linux-x64, linux-arm64, macos-x64, macos-arm64, windows-x64");
                    std::process::exit(1);
                }
            };
            match curator::build_single(&path, &version, p) {
                Ok(result) => {
                    println!("\n✅ Build successful!");
                    println!("  Version: {}", result.version);
                    println!("  Platform: {}", result.platform);
                    println!("  Size: {} bytes", result.size_bytes);
                    println!("  SHA256: {}", result.sha256.unwrap_or_default());
                    println!("  Output: {}", result.output_path.unwrap_or_default());
                }
                Err(e) => {
                    eprintln!("\n❌ Build failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Cli::BuildAll { platforms, path } => {
            let platform_list: Vec<curator::build::Platform> = platforms
                .iter()
                .filter_map(|p| curator::build::Platform::from_str(p))
                .collect();

            if platform_list.is_empty() {
                eprintln!("No valid platforms specified");
                std::process::exit(1);
            }

            println!("Building for platforms: {:?}", platform_list);
            let results = curator::build_all(&path, platform_list)?;
            println!("\n✅ Built {} versions", results.len());
        }
        Cli::CheckDeps { platform } => {
            let p = match curator::build::Platform::from_str(&platform) {
                Some(p) => p,
                None => {
                    eprintln!("Unknown platform: {}", platform);
                    std::process::exit(1);
                }
            };
            curator::check_deps(p)?;
        }
    }

    Ok(())
}
