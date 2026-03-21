use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Platform {
    LinuxX64,
    LinuxArm64,
    MacosX64,
    MacosArm64,
    WindowsX64,
}

impl Platform {
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::LinuxX64 => "linux-x64",
            Platform::LinuxArm64 => "linux-arm64",
            Platform::MacosX64 => "macos-x64",
            Platform::MacosArm64 => "macos-arm64",
            Platform::WindowsX64 => "windows-x64",
        }
    }

    pub fn from_str(s: &str) -> Option<Platform> {
        match s {
            "linux-x64" => Some(Platform::LinuxX64),
            "linux-arm64" => Some(Platform::LinuxArm64),
            "linux-x86_64" => Some(Platform::LinuxX64),
            "linux-aarch64" => Some(Platform::LinuxArm64),
            "macos-x64" | "macos-x86_64" => Some(Platform::MacosX64),
            "macos-arm64" | "macos-aarch64" => Some(Platform::MacosArm64),
            "windows-x64" => Some(Platform::WindowsX64),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn all() -> Vec<Platform> {
        vec![
            Platform::LinuxX64,
            Platform::LinuxArm64,
            Platform::MacosX64,
            Platform::MacosArm64,
            Platform::WindowsX64,
        ]
    }

    #[allow(dead_code)]
    pub fn current() -> Option<Platform> {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86_64") => Some(Platform::LinuxX64),
            ("linux", "aarch64") => Some(Platform::LinuxArm64),
            ("macos", "x86_64") => Some(Platform::MacosX64),
            ("macos", "aarch64") => Some(Platform::MacosArm64),
            ("windows", "x86_64") => Some(Platform::WindowsX64),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn is_linux(&self) -> bool {
        matches!(self, Platform::LinuxX64 | Platform::LinuxArm64)
    }

    #[allow(dead_code)]
    pub fn is_macos(&self) -> bool {
        matches!(self, Platform::MacosX64 | Platform::MacosArm64)
    }

    #[allow(dead_code)]
    pub fn is_windows(&self) -> bool {
        matches!(self, Platform::WindowsX64)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub version: String,
    pub platform: String,
    pub status: BuildStatus,
    pub output_path: Option<String>,
    pub sha256: Option<String>,
    pub size_bytes: u64,
    pub built_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BuildStatus {
    Pending,
    Building,
    Success,
    Failed,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildManifest {
    pub builds: Vec<BuildResult>,
}

impl BuildManifest {
    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let manifest: BuildManifest = serde_json::from_str(&content)?;
            Ok(manifest)
        } else {
            Ok(BuildManifest::default())
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn add_build(&mut self, result: BuildResult) {
        self.builds.push(result);
    }
}

pub struct BuildConfig {
    pub base_path: PathBuf,
}

impl BuildConfig {
    pub fn new(base_path: &Path) -> Self {
        Self {
            base_path: base_path.to_path_buf(),
        }
    }

    pub fn sources_dir(&self) -> PathBuf {
        self.base_path.join("sources")
    }

    pub fn temp_dir(&self, platform: Platform) -> PathBuf {
        self.base_path.join("temp").join(platform.as_str())
    }

    pub fn output_dir(&self, platform: Platform) -> PathBuf {
        self.base_path.join("builds").join(platform.as_str())
    }

    pub fn source_archive(&self, version: &str) -> PathBuf {
        self.sources_dir().join(format!("php-{}.tar.xz", version))
    }

    #[allow(dead_code)]
    pub fn php_source_dir(&self, version: &str) -> PathBuf {
        self.sources_dir().join(format!("php-{}", version))
    }
}

pub fn get_configure_flags(platform: Platform) -> Vec<&'static str> {
    let mut flags = vec![
        "--disable-all",
        "--enable-cli",
        "--enable-mbstring",
        "--enable-intl",
        "--enable-bcmath",
        "--enable-calendar",
        "--enable-exif",
        "--enable-ftp",
        "--enable-pcntl",
        "--enable-sockets",
        "--enable-fileinfo",
        "--enable-pdo",
        "--with-openssl",
        "--with-curl",
        "--with-zlib",
        "--with-pdo-sqlite",
        "--with-pdo-mysql",
        "--with-gettext",
        "--disable-phpdbg",
        "--disable-cgi",
        "--disable-fpm",
    ];

    match platform {
        Platform::LinuxX64 | Platform::LinuxArm64 => {
            flags.push("--enable-sockets");
        }
        Platform::MacosX64 | Platform::MacosArm64 => {
            flags.push("--with-iconv=/usr");
        }
        Platform::WindowsX64 => {
            flags.push("--disable-zts");
        }
    }

    flags
}

pub fn check_dependencies(platform: Platform) -> Result<()> {
    let missing_deps: Vec<&str>;

    match platform {
        Platform::LinuxX64 | Platform::LinuxArm64 => {
            missing_deps = check_linux_deps();
        }
        Platform::MacosX64 | Platform::MacosArm64 => {
            missing_deps = check_macos_deps();
        }
        Platform::WindowsX64 => {
            missing_deps = check_windows_deps();
        }
    }

    if !missing_deps.is_empty() {
        warn!("Missing dependencies: {:?}", missing_deps);
        info!("Install with:");
        match platform {
            Platform::LinuxX64 | Platform::LinuxArm64 => {
                if Command::new("which")
                    .arg("pacman")
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
                {
                    println!("  sudo pacman -S {}", missing_deps.join(" "));
                } else {
                    println!(
                        "  sudo apt install {}",
                        missing_deps
                            .iter()
                            .map(|s| format!("{}-dev", s))
                            .collect::<Vec<_>>()
                            .join(" ")
                    );
                }
            }
            Platform::MacosX64 | Platform::MacosArm64 => {
                println!("  brew install {}", missing_deps.join(" "));
            }
            Platform::WindowsX64 => {
                println!("  Install Visual Studio 2022 with C++ workload");
            }
        }
    }

    Ok(())
}

fn check_linux_deps() -> Vec<&'static str> {
    let mut missing = Vec::new();
    let required_binaries = vec![
        "gcc",
        "g++",
        "make",
        "autoconf",
        "automake",
        "bison",
        "re2c",
        "libxml2",
        "openssl",
        "curl",
        "zip",
        "gd",
        "oniguruma",
        "sqlite3",
    ];

    let pkg_names: std::collections::HashMap<&str, &str> = [
        ("gcc", "gcc"),
        ("g++", "gcc"),
        ("make", "make"),
        ("autoconf", "autoconf"),
        ("automake", "automake"),
        ("bison", "bison"),
        ("re2c", "re2c"),
        ("libxml2", "libxml2"),
        ("openssl", "openssl"),
        ("curl", "curl"),
        ("zip", "libzip"),
        ("gd", "gd"),
        ("oniguruma", "oniguruma"),
        ("sqlite3", "sqlite"),
    ]
    .into_iter()
    .collect();

    for binary in required_binaries {
        if Command::new("which")
            .arg(binary)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
            == false
        {
            if let Some(pkg) = pkg_names.get(binary) {
                missing.push(*pkg);
            } else {
                missing.push(binary);
            }
        }
    }

    missing
}

fn check_macos_deps() -> Vec<&'static str> {
    let mut missing = Vec::new();
    let required = vec!["autoconf", "automake", "bison", "re2c"];

    for dep in required {
        if Command::new("which")
            .arg(dep)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
            == false
        {
            missing.push(dep);
        }
    }

    missing
}

fn check_windows_deps() -> Vec<&'static str> {
    vec!["Visual Studio 2022"]
}

pub fn extract_source(source_archive: &Path, temp_dir: &Path) -> Result<PathBuf> {
    info!("Extracting source from {:?}", source_archive);

    let extract_base = temp_dir.parent().unwrap_or(std::path::Path::new("."));
    let extract_base = std::fs::canonicalize(extract_base)?;
    let source_archive = std::fs::canonicalize(source_archive)?;

    std::fs::create_dir_all(&extract_base)?;

    let output = Command::new("tar")
        .args(["-xf", &source_archive.to_string_lossy()])
        .current_dir(&extract_base)
        .output()
        .context("Failed to extract source archive")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("tar stderr: {}", stderr);
        anyhow::bail!("tar extraction failed");
    }

    let extracted_dir = extract_base.join(
        source_archive
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .trim_end_matches(".tar.xz")
            .trim_end_matches(".tar.gz")
            .trim_end_matches(".tar.bz2")
            .trim_end_matches(".tar"),
    );

    info!("Extracted to {:?}", extracted_dir);
    Ok(extracted_dir)
}

pub fn clean_bloat(source_dir: &Path) -> Result<u64> {
    info!("Cleaning bloat from {:?}", source_dir);
    let mut freed: u64 = 0;

    let patterns_to_remove = vec!["**/*.phpt", "**/man/**/*.1", "**/man/**/*.3"];

    for pattern in patterns_to_remove {
        let full_pattern = source_dir.join(pattern);
        if let Ok(entries) = glob::glob(&full_pattern.to_string_lossy()) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    freed += metadata.len();
                    let _ = std::fs::remove_file(&entry);
                }
            }
        }
    }

    let dirs_to_remove = vec![source_dir.join("php/man")];

    for dir in dirs_to_remove {
        if dir.exists() {
            if let Ok(metadata) = dir.metadata() {
                freed += metadata.len();
            }
            let _ = std::fs::remove_dir_all(&dir);
        }
    }

    info!("Freed {} bytes from bloat", freed);
    Ok(freed)
}

pub fn run_configure(source_dir: &Path, platform: Platform) -> Result<()> {
    info!("Running configure for {:?}", platform);

    let flags = get_configure_flags(platform);
    let mut cmd = Command::new("./configure");
    cmd.current_dir(source_dir);

    for flag in &flags {
        cmd.arg(flag);
    }

    cmd.arg(&format!(
        "--prefix={}",
        source_dir.join("install").display()
    ));

    match platform {
        Platform::LinuxX64 => {}
        Platform::LinuxArm64 => {
            cmd.arg("--host=aarch64-linux-gnu");
        }
        Platform::MacosX64 | Platform::MacosArm64 => {
            cmd.env("CFLAGS", "-mmacosx-version-min=10.15");
            cmd.env("CXXFLAGS", "-mmacosx-version-min=10.15");
        }
        Platform::WindowsX64 => {
            cmd.arg("--host=x86_64-w64-mingw32");
        }
    }

    info!("Configure command: {:?}", cmd);

    let output = cmd.output().context("Failed to run configure")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("Configure failed: {}", stderr);
        anyhow::bail!("Configure failed");
    }

    info!("Configure completed successfully");
    Ok(())
}

pub fn run_make(source_dir: &Path, platform: Platform) -> Result<PathBuf> {
    info!("Running make for {:?}", platform);

    let jobs = std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(4);

    let mut cmd = Command::new("make");
    cmd.current_dir(source_dir);

    match platform {
        Platform::WindowsX64 => {
            cmd = Command::new("nmake");
            cmd.current_dir(source_dir);
        }
        _ => {
            cmd.arg("-j").arg(jobs.to_string());
        }
    }

    let output = cmd.output().context("Failed to run make")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("Make failed: {}", stderr);
        anyhow::bail!("Make failed");
    }

    let php_binary = source_dir.join("sapi").join("cli").join("php");
    let php_binary_alt = source_dir.join("php");

    if php_binary.exists() {
        Ok(php_binary)
    } else if php_binary_alt.exists() {
        Ok(php_binary_alt)
    } else {
        anyhow::bail!("PHP binary not found after make")
    }
}

pub fn strip_binary(binary_path: &Path) -> Result<u64> {
    info!("Stripping binary {:?}", binary_path);

    let original_size = std::fs::metadata(binary_path)?.len();

    let output = Command::new("strip")
        .arg("--strip-all")
        .arg(binary_path)
        .output()
        .context("Failed to strip binary")?;

    if !output.status.success() {
        warn!("Strip command returned non-zero status");
    }

    let new_size = std::fs::metadata(binary_path)?.len();
    let saved = original_size - new_size;

    info!(
        "Binary stripped: {} -> {} bytes (saved {})",
        original_size, new_size, saved
    );
    Ok(saved)
}

pub fn verify_binary(binary_path: &Path) -> Result<()> {
    info!("Verifying binary {:?}", binary_path);

    let output = Command::new(binary_path)
        .args(["-v"])
        .output()
        .context("Failed to run php -v")?;

    if !output.status.success() {
        anyhow::bail!("PHP binary verification failed");
    }

    let version = String::from_utf8_lossy(&output.stdout);
    info!(
        "PHP version: {}",
        version.lines().next().unwrap_or("unknown")
    );

    Ok(())
}

pub fn compute_sha256(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    use std::io;

    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn copy_to_output(binary_path: &Path, output_dir: &Path, version: &str) -> Result<PathBuf> {
    std::fs::create_dir_all(output_dir)?;

    let output_name = if cfg!(target_os = "windows") {
        format!("php-{}.exe", version)
    } else {
        format!("php-{}", version)
    };

    let output_path = output_dir.join(&output_name);
    std::fs::copy(binary_path, &output_path)?;

    let size = std::fs::metadata(&output_path)?.len();
    info!("Copied to {:?} ({} bytes)", output_path, size);

    Ok(output_path)
}

pub fn cleanup_build_dir(temp_dir: &Path) -> Result<()> {
    info!("Cleaning up {:?}", temp_dir);

    if temp_dir.exists() {
        std::fs::remove_dir_all(temp_dir)?;
    }

    Ok(())
}
