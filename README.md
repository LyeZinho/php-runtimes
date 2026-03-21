# php-runtimes
Php runtimes for cleanserve.

## Downloading PHP Binaries

Binaries are distributed via [GitHub Releases](https://github.com/${{ github.repository }}/releases).

### Manual Download

1. Visit [Releases](https://github.com/${{ github.repository }}/releases)
2. Download the archive for your platform
3. Extract: `tar -xzf php-8.5.4-linux-x64.tar.gz`

### Automated Download

```bash
# Get latest version
LATEST=$(curl -s https://api.github.com/repos/${{ github.repository }}/releases/latest | jq -r .tag_name)
VERSION=${LATEST#v}

# Download for Linux x64
curl -L -O "https://github.com/${{ github.repository }}/releases/download/$LATEST/php-$VERSION-linux-x64.tar.gz"

# Verify integrity
sha256sum -c <(curl -s "https://github.com/${{ github.repository }}/releases/download/$LATEST/checksums-$VERSION.txt" | grep "linux-x64")
```

### Creating a New Release

```bash
# Create tag
git tag v8.5.4

# Push tag to trigger release
git push origin v8.5.4
```

The CI will automatically:
1. Package binaries for supported platforms (currently Linux x64)
2. Create a GitHub Release
3. Attach .tar.gz archives and checksums

**Note:** Windows support is planned but not yet implemented due to cross-compilation limitations.

## Version Manifest

The `manifests/versions.json` file provides a hierarchical index of all available PHP binary releases for installer consumption.

### Updating the Manifest

After creating a new GitHub Release:

```bash
# Generate/update manifest
./scripts/update-manifest.sh

# Review changes
git diff manifests/versions.json

# Commit and push
git add manifests/versions.json
git commit -m "docs: update version manifest for vX.Y.Z"
git push
```

### Manifest Structure

The manifest contains:
- `versions[]` - Array of releases in reverse chronological order
  - `version` - PHP version (e.g., "8.5.4")
  - `tag` - Git tag (e.g., "v8.5.4")
  - `platforms[]` - Available platforms
    - `platform` - Platform identifier (e.g., "linux-x64")
    - `filename` - Archive filename
    - `download_url` - Direct download URL
    - `size_bytes` - File size
    - `sha256` - SHA256 checksum (if available)

### Installer Usage

```bash
# Fetch manifest
curl -s https://raw.githubusercontent.com/LyeZinho/php-runtimes/main/manifests/versions.json

# Get latest version
LATEST=$(curl -s https://raw.githubusercontent.com/LyeZinho/php-runtimes/main/manifests/versions.json | jq -r '.versions[0].version')

# Get download URL for specific version and platform
URL=$(curl -s https://raw.githubusercontent.com/LyeZinho/php-runtimes/main/manifests/versions.json | \
  jq -r --arg ver "8.5.4" --arg plat "linux-x64" \
  '.versions[] | select(.version==$ver) | .platforms[] | select(.platform==$plat) | .download_url')
```
