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
# GitHub Releases integration
