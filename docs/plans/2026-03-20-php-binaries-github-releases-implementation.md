# PHP Binaries GitHub Releases Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable automatic distribution of PHP binaries via GitHub Releases with versioning and public download URLs.

**Architecture:** GitHub Actions workflow triggered by Git tags, packaging binaries per platform into .tar.gz archives and attaching them to GitHub Releases.

**Tech Stack:** GitHub Actions, Bash scripting, Git tags, SHA256 checksums.

---

## Pre-Implementation

### Step 0: Verify project structure and existing workflow

**Files:**
- Read: `builds/manifest.json`
- Read: `.github/workflows/curate.yml`
- List: `builds/` directories to understand platform structure

**Command:**
```bash
ls -la builds/
```

**Expected:** See platform directories (linux-x64, macos, windows)

---

## Task 1: Create packaging script

**Files:**
- Create: `scripts/package-release.sh`
- Modify: `.gitignore` (add `dist/`)

### Step 1: Create scripts directory

**Command:**
```bash
mkdir -p scripts
```

### Step 2: Write packaging script

**File:** `scripts/package-release.sh`

```bash
#!/usr/bin/env bash
set -euo pipefail

# Extract version from git tag (v8.5.4 -> 8.5.4)
VERSION="${GITHUB_REF_NAME#v}"
if [[ -z "$VERSION" ]]; then
  echo "Error: Could not determine version from GITHUB_REF_NAME"
  exit 1
fi

echo "Packaging PHP binaries for version: $VERSION"

# Create output directory
DIST_DIR="dist"
mkdir -p "$DIST_DIR"

# Package each platform
for platform_dir in builds/*/; do
  platform=$(basename "$platform_dir")
  [[ "$platform" == "manifest.json" ]] && continue
  
  echo "Packaging $platform..."
  
  # Create tar.gz archive
  tar -czf "$DIST_DIR/php-$VERSION-$platform.tar.gz" -C "builds" "$platform"
  
  # Generate checksum
  sha256sum "$DIST_DIR/php-$VERSION-$platform.tar.gz" >> "$DIST_DIR/checksums-$VERSION.txt"
done

# Generate manifest with download URLs
cat > "$DIST_DIR/manifest-$VERSION.json" << EOF
{
  "version": "$VERSION",
  "assets": [
EOF

for asset in "$DIST_DIR"/php-$VERSION-*.tar.gz; do
  filename=$(basename "$asset")
  platform=${filename#php-$VERSION-}
  platform=${platform%.tar.gz}
  size=$(stat -f%z "$asset" 2>/dev/null || stat -c%s "$asset")
  sha256=$(sha256sum "$asset" | cut -d' ' -f1)
  
  cat >> "$DIST_DIR/manifest-$VERSION.json" << EOF
    {
      "platform": "$platform",
      "filename": "$filename",
      "size_bytes": $size,
      "sha256": "$sha256"
    },
EOF
done

# Remove trailing comma and close JSON
sed -i '$ s/,$//' "$DIST_DIR/manifest-$VERSION.json"
echo "  ]" >> "$DIST_DIR/manifest-$VERSION.json"
echo "}" >> "$DIST_DIR/manifest-$VERSION.json"

echo "Packaging complete. Assets in $DIST_DIR/"
ls -lh "$DIST_DIR/"
```

### Step 3: Make script executable

**Command:**
```bash
chmod +x scripts/package-release.sh
```

### Step 4: Test script locally (dry-run)

**Command:**
```bash
GITHUB_REF_NAME="v8.5.4" ./scripts/package-release.sh
```

**Expected:** Creates `dist/` directory with .tar.gz files and checksums

### Step 5: Add dist/ to .gitignore

**File:** `.gitignore` (append)

```
dist/
```

### Step 6: Commit packaging script

**Command:**
```bash
git add scripts/package-release.sh .gitignore
git commit -m "feat: add packaging script for GitHub Releases"
```

---

## Task 2: Create GitHub Actions workflow

**Files:**
- Create: `.github/workflows/release.yml`

### Step 1: Write release workflow

**File:** `.github/workflows/release.yml`

```yaml
name: Release PHP Binaries

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write

jobs:
  package-and-release:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Package binaries
        id: package
        run: |
          chmod +x scripts/package-release.sh
          ./scripts/package-release.sh
          
          # Output version for later use
          VERSION="${GITHUB_REF_NAME#v}"
          echo "version=$VERSION" >> $GITHUB_OUTPUT

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          tag_name: ${{ github.ref_name }}
          name: "PHP ${{ steps.package.outputs.version }}"
          body: |
            ## PHP Runtime ${{ steps.package.outputs.version }}
            
            ### Assets
            
            Download the appropriate archive for your platform:
            - `php-${{ steps.package.outputs.version }}-linux-x64.tar.gz` - Linux x86_64
            - `php-${{ steps.package.outputs.version }}-macos-arm64.tar.gz` - macOS ARM64 (Apple Silicon)
            - `php-${{ steps.package.outputs.version }}-macos-x64.tar.gz` - macOS x86_64
            - `php-${{ steps.package.outputs.version }}-windows-x64.zip` - Windows x86_64
            
            ### Verification
            
            Verify downloads using `checksums-${{ steps.package.outputs.version }}.txt`.
            
            ### API Access
            
            ```bash
            # Get latest release info
            curl -s https://api.github.com/repos/${{ github.repository }}/releases/latest
            
            # Download specific asset
            curl -L -O https://github.com/${{ github.repository }}/releases/download/v${{ steps.package.outputs.version }}/php-${{ steps.package.outputs.version }}-linux-x64.tar.gz
            ```
          files: |
            dist/*.tar.gz
            dist/*.txt
            dist/*.json
          draft: false
          prerelease: false
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### Step 2: Test workflow syntax

**Command:**
```bash
# Validate YAML syntax
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))"
```

**Expected:** No error

### Step 3: Commit workflow

**Command:**
```bash
git add .github/workflows/release.yml
git commit -m "ci: add release workflow for packaging PHP binaries"
```

---

## Task 3: Update documentation and create test tag

**Files:**
- Modify: `README.md`
- Create: Test tag (optional)

### Step 1: Update README with release instructions

**File:** `README.md` (append)

```markdown

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
```

### Step 2: Commit documentation

**Command:**
```bash
git add README.md
git commit -m "docs: add release download instructions"
```

### Step 3: Create test tag (optional)

**Command:**
```bash
# Create a test tag
git tag v8.5.4-test

# Push to trigger release workflow
git push origin v8.5.4-test
```

**Expected:** GitHub Actions runs and creates a release

### Step 4: Verify release was created

**Command:**
```bash
# Wait a minute for CI to complete, then check
curl -s https://api.github.com/repos/${{ github.repository }}/releases/latest | jq '{tag_name, name, assets: .assets[].name}'
```

**Expected:** Release with assets listed

---

## Task 4: Cleanup and finalization

### Step 1: Remove test release (if created)

**Command:**
```bash
# Delete test tag locally and remotely
git tag -d v8.5.4-test
git push origin --delete v8.5.4-test

# Delete test release via GitHub API
curl -X DELETE -H "Authorization: token $GITHUB_TOKEN" \
  "https://api.github.com/repos/${{ github.repository }}/releases/$(curl -s https://api.github.com/repos/${{ github.repository }}/releases | jq -r '.[0].id')"
```

### Step 2: Final commit with all changes

**Command:**
```bash
git add -A
git commit -m "feat: complete GitHub Releases integration for PHP binaries"
```

### Step 3: Push changes

**Command:**
```bash
git push origin main
```

---

## Verification Checklist

- [ ] Packaging script creates .tar.gz files
- [ ] Checksums file is generated
- [ ] Workflow YAML is valid
- [ ] Test release appears on GitHub
- [ ] Assets are downloadable via direct URL
- [ ] API returns correct release information
- [ ] Documentation is clear and complete

---

## Expected URLs After Release

For version 8.5.4:
- `https://github.com/owner/php-runtimes/releases/download/v8.5.4/php-8.5.4-linux-x64.tar.gz`
- `https://github.com/owner/php-runtimes/releases/download/v8.5.4/php-8.5.4-macos-arm64.tar.gz`
- `https://github.com/owner/php-runtimes/releases/download/v8.5.4/checksums-8.5.4.txt`
- `https://github.com/owner/php-runtimes/releases/download/v8.5.4/manifest-8.5.4.json`

## Notes

- GitHub Releases has a 2GB per-file limit and 50GB per-repository limit
- Public repositories have unlimited bandwidth for public downloads
- The `softprops/action-gh-release` action handles asset uploads automatically
- Checksums are essential for security verification in automated downloads