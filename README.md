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
1. Package binaries for all platforms
2. Create a GitHub Release
3. Attach .tar.gz archives and checksums
