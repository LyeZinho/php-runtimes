# PHP Runtimes

Pre-compiled PHP CLI binaries for Linux, distributed via GitHub Releases.

![PHP Versions](https://img.shields.io/badge/php-8.3%20--%208.5-blue)
![Platform](https://img.shields.io/badge/platform-linux--x64-green)
![License](https://img.shields.io/badge/license-MIT-purple)

## Available Versions

| Version | Release Date | Download |
|---------|--------------|----------|
| **8.5.4** | Latest | [Download](https://github.com/LyeZinho/php-runtimes/releases/latest) |
| 8.5.x | 12 versions | [All 8.5.x](https://github.com/LyeZinho/php-runtimes/releases?q=v8.5) |
| 8.4.x | 20 versions | [All 8.4.x](https://github.com/LyeZinho/php-runtimes/releases?q=v8.4) |
| 8.3.x | 12 versions | [All 8.3.x](https://github.com/LyeZinho/php-runtimes/releases?q=v8.3) |

**Total:** 37 PHP versions available

## Quick Start

### Download Latest PHP

```bash
# Download PHP 8.5.4 for Linux x64
curl -L -O https://github.com/LyeZinho/php-runtimes/releases/download/v8.5.4/php-8.5.4-linux-x64.tar.gz

# Extract
tar -xzf php-8.5.4-linux-x64.tar.gz

# Verify it works
./php-8.5.4 --version
```

### Download Specific Version

```bash
VERSION="8.4.12"

curl -L -O "https://github.com/LyeZinho/php-runtimes/releases/download/v${VERSION}/php-${VERSION}-linux-x64.tar.gz"
tar -xzf "php-${VERSION}-linux-x64.tar.gz"
./php-${VERSION} --version
```

### Verify Checksum

```bash
# Download checksum file
curl -L -O https://github.com/LyeZinho/php-runtimes/releases/download/v8.5.4/checksums-8.5.4.txt

# Verify
sha256sum -c checksums-8.5.4.txt
```

## Version Manifest

The `manifests/versions.json` file provides a programmatic index of all releases for installer tools.

### Fetch Manifest

```bash
# Get manifest
curl -s https://raw.githubusercontent.com/LyeZinho/php-runtimes/main/manifests/versions.json

# Get latest version
LATEST=$(curl -s https://raw.githubusercontent.com/LyeZinho/php-runtimes/main/manifests/versions.json | jq -r '.versions[0].version')

# Get download URL for specific version
URL=$(curl -s https://raw.githubusercontent.com/LyeZinho/php-runtimes/main/manifests/versions.json | \
  jq -r --arg ver "8.5.4" --arg plat "linux-x64" \
  '.versions[] | select(.version==$ver) | .platforms[] | select(.platform==$plat) | .download_url')
```

### Manifest Structure

```json
{
  "schema_version": "1.0",
  "updated_at": "2026-03-21T02:00:04Z",
  "repository": "LyeZinho/php-runtimes",
  "versions": [
    {
      "version": "8.5.4",
      "tag": "v8.5.4",
      "published_at": "2026-03-21T01:53:44Z",
      "html_url": "https://github.com/LyeZinho/php-runtimes/releases/tag/v8.5.4",
      "platforms": [
        {
          "platform": "linux-x64",
          "filename": "php-8.5.4-linux-x64.tar.gz",
          "download_url": "https://github.com/LyeZinho/php-runtimes/releases/download/v8.5.4/php-8.5.4-linux-x64.tar.gz",
          "size_bytes": 5776985,
          "sha256": "2a99aaf8bd2e98ef2cf5028fa62ac165a06d144d2672f15665e72dd815d9ed6c"
        }
      ]
    }
  ]
}
```

## Scripts

### Create All Releases

Batch create GitHub releases for all PHP versions:

```bash
# Dry run (preview)
./scripts/create-all-releases.sh --dry-run

# Create releases
./scripts/create-all-releases.sh
```

### Update Manifest

Regenerate the version manifest from GitHub Releases:

```bash
# Generate manifest
./scripts/update-manifest.sh

# Add checksums from local dist/ files
./scripts/add-checksums-to-manifest.sh

# Review and commit
git diff manifests/versions.json
git add manifests/versions.json
git commit -m "docs: update version manifest"
git push
```

### Cross-Platform Build

Build PHP for different platforms:

```bash
# Linux x64 (native)
./scripts/cross-platform-build.sh --version 8.5.4 --platform linux-x64

# Windows x64 (via Docker - experimental)
./scripts/cross-platform-build.sh --version 8.5.4 --platform windows-x64

# All platforms
./scripts/cross-platform-build.sh --version 8.5.4 --platform all
```

## Creating a New Release

### Manual Release

```bash
# Create and push tag
git tag v8.5.5
git push origin v8.5.5
```

The CI workflow automatically:
1. Packages binaries for all platforms
2. Creates a GitHub Release
3. Attaches .tar.gz archives and checksums

### Batch Release

```bash
# Create releases for all versions at once
./scripts/create-all-releases.sh

# Update manifest
./scripts/update-manifest.sh
./scripts/add-checksums-to-manifest.sh

# Commit and push
git add manifests/versions.json
git commit -m "docs: update manifest with new releases"
git push
```

## Project Structure

```
php-runtimes/
├── builds/                    # Compiled PHP binaries
│   └── linux-x64/
│       ├── php-8.5.4
│       ├── php-8.4.12
│       └── ...
├── dist/                      # Packaged archives (temporary)
├── manifests/
│   └── versions.json          # Version manifest for installers
├── scripts/
│   ├── cross-platform-build.sh    # Cross-platform build script
│   ├── create-all-releases.sh     # Batch release creation
│   ├── add-checksums-to-manifest.sh  # Add checksums to manifest
│   ├── package-release.sh         # Package single release
│   └── update-manifest.sh         # Update manifest from GitHub
├── src/
│   └── curator/               # Rust-based PHP curator
└── .github/
    └── workflows/
        ├── curate.yml         # Auto-curate new PHP versions
        └── release.yml        # Create releases on tag push
```

## Platform Support

| Platform | Status | Method |
|----------|--------|--------|
| Linux x64 | ✅ Supported | Native build |
| Linux ARM64 | 🚧 Planned | Cross-compilation |
| Windows x64 | 🚧 Planned | Docker/MSVC |
| macOS x64 | 🚧 Planned | Native build |
| macOS ARM64 | 🚧 Planned | Native build |

## PHP Configuration

The compiled PHP binaries include:

- **CLI SAPI** only (no server modules)
- Extensions: `json`, `mbstring`, `phar`, `tokenizer`
- No PEAR
- No CGI/FPM
- Thread Safety: **Disabled** (NTS)
- OPcache: **Enabled**

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

MIT License - see [LICENSE](LICENSE) for details.
