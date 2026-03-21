#!/usr/bin/env bash
set -euo pipefail

# Add checksums to manifest using local files
#
# Usage:
#   ./scripts/add-checksums-to-manifest.sh [--dry-run]

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Parse arguments
DRY_RUN=false

while [[ $# -gt 0 ]]; do
  case $1 in
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    --help|-h)
      echo "Add checksums to manifest using local dist/ files"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

DIST_DIR="dist"
MANIFEST_FILE="manifests/versions.json"

if [[ ! -f "$MANIFEST_FILE" ]]; then
  echo -e "${RED}Error: Manifest file not found: $MANIFEST_FILE${NC}"
  exit 1
fi

if [[ ! -d "$DIST_DIR" ]]; then
  echo -e "${RED}Error: Dist directory not found: $DIST_DIR${NC}"
  exit 1
fi

echo -e "${BLUE}Adding checksums to manifest...${NC}"

# Read manifest
MANIFEST=$(cat "$MANIFEST_FILE")

# For each checksums file in dist/
for CHECKSUM_FILE in "$DIST_DIR"/checksums-*.txt; do
  if [[ ! -f "$CHECKSUM_FILE" ]]; then
    continue
  fi
  
  # Extract version from filename
  BASENAME=$(basename "$CHECKSUM_FILE")
  VERSION=$(echo "$BASENAME" | sed 's/checksums-//;s/\.txt//')
  TAG="v$VERSION"
  
  echo -e "  Processing $VERSION..."
  
  # Read checksum
  CHECKSUM=$(head -1 "$CHECKSUM_FILE" | awk '{print $1}')
  ARCHIVE_NAME=$(basename "$(head -1 "$CHECKSUM_FILE" | awk '{print $2}')")
  
  # Update manifest
  MANIFEST=$(echo "$MANIFEST" | jq --arg tag "$TAG" --arg filename "$ARCHIVE_NAME" --arg sha "$CHECKSUM" '
    .versions |= map(
      if .tag == $tag then
        .platforms |= map(
          if .filename == $filename then
            .sha256 = $sha
          else
            .
          end
        )
      else
        .
      end
    )
  ')
done

# Write updated manifest
if [[ "$DRY_RUN" == true ]]; then
  echo -e "${YELLOW}[DRY RUN] Would update manifest${NC}"
  echo "$MANIFEST" | jq '.versions[0].platforms[0]' | head -10
else
  echo "$MANIFEST" | jq . > "$MANIFEST_FILE"
  echo -e "${GREEN}Manifest updated with checksums${NC}"
  
  # Count updated entries
  NULL_COUNT=$(echo "$MANIFEST" | jq '[.versions[].platforms[].sha256] | map(select(. == null)) | length')
  echo -e "  Entries with checksums: $(echo "$MANIFEST" | jq '[.versions[].platforms[].sha256] | map(select(. != null)) | length')"
  echo -e "  Entries without checksums: $NULL_COUNT"
fi
