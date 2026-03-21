#!/usr/bin/env bash
set -euo pipefail

# Create GitHub releases for all PHP versions
#
# Usage:
#   ./scripts/create-all-releases.sh [--dry-run] [--skip-existing]
#
# This script:
# 1. Packages each version into tar.gz archives
# 2. Creates checksums
# 3. Creates GitHub releases with assets

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Parse arguments
DRY_RUN=false
SKIP_EXISTING=true
DELETE_EXISTING=false

while [[ $# -gt 0 ]]; do
  case $1 in
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    --force)
      SKIP_EXISTING=false
      shift
      ;;
    --replace)
      SKIP_EXISTING=false
      DELETE_EXISTING=true
      shift
      ;;
    --help|-h)
      echo "Create GitHub releases for all PHP versions"
      echo ""
      echo "Usage: $0 [OPTIONS]"
      echo ""
      echo "Options:"
      echo "  --dry-run    Show what would be done without making changes"
      echo "  --force      Create releases even if they already exist"
      echo "  --replace    Delete existing releases and recreate them"
      echo "  --help, -h   Show this help"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Check if gh CLI is available
if ! command -v gh &> /dev/null; then
  echo -e "${RED}Error: GitHub CLI (gh) is not installed${NC}"
  echo "Install with: sudo apt install gh"
  exit 1
fi

# Check if authenticated
if ! gh auth status &> /dev/null; then
  echo -e "${RED}Error: Not authenticated with GitHub${NC}"
  echo "Run: gh auth login"
  exit 1
fi

# Get repository info
REPO=$(gh repo view --json nameWithOwner -q '.nameWithOwner' 2>/dev/null || true)
if [[ -z "$REPO" ]]; then
  echo -e "${RED}Error: Not in a GitHub repository${NC}"
  exit 1
fi

echo -e "${BLUE}Repository: $REPO${NC}"
echo ""

# Find all PHP versions in builds/linux-x64/
BUILD_DIR="builds/linux-x64"
if [[ ! -d "$BUILD_DIR" ]]; then
  echo -e "${RED}Error: Build directory $BUILD_DIR not found${NC}"
  exit 1
fi

# Create dist directory for packaging
DIST_DIR="dist"
mkdir -p "$DIST_DIR"

# Get list of versions
VERSIONS=$(ls "$BUILD_DIR" | grep "^php-" | sed 's/php-//' | sort -V)
TOTAL=$(echo "$VERSIONS" | wc -l)

echo -e "${BLUE}Found $TOTAL PHP versions${NC}"
echo ""

# Process each version
COUNT=0
SKIPPED=0
CREATED=0
FAILED=0

for VERSION in $VERSIONS; do
  COUNT=$((COUNT + 1))
  TAG="v$VERSION"
  
  echo -e "${BLUE}[$COUNT/$TOTAL] Processing PHP $VERSION${NC}"
  
  # Check if release already exists
  if gh release view "$TAG" &> /dev/null; then
    if [[ "$DELETE_EXISTING" == true ]]; then
      echo -e "  ${YELLOW}Deleting existing release $TAG...${NC}"
      if [[ "$DRY_RUN" == false ]]; then
        gh release delete "$TAG" --yes 2>/dev/null || true
      fi
    elif [[ "$SKIP_EXISTING" == true ]]; then
      echo -e "  ${YELLOW}Release $TAG already exists, skipping${NC}"
      SKIPPED=$((SKIPPED + 1))
      continue
    fi
  fi
  
  # Package the binary (only this specific version)
  ARCHIVE="$DIST_DIR/php-$VERSION-linux-x64.tar.gz"
  
  echo "  Packaging..."
  
  # Create temporary directory with only this version's binary
  TMP_PKG=$(mktemp -d)
  mkdir -p "$TMP_PKG/linux-x64"
  cp "$BUILD_DIR/php-$VERSION" "$TMP_PKG/linux-x64/"
  
  # Create tar.gz archive
  tar -czf "$ARCHIVE" -C "$TMP_PKG" "linux-x64"
  
  # Cleanup
  rm -rf "$TMP_PKG"
  
  # Generate checksum
  CHECKSUM_FILE="$DIST_DIR/checksums-$VERSION.txt"
  (cd "$DIST_DIR" && sha256sum "php-$VERSION-linux-x64.tar.gz" > "checksums-$VERSION.txt")
  
  # Create release notes
  RELEASE_BODY="## PHP Runtime $VERSION

### Assets

- \`php-$VERSION-linux-x64.tar.gz\` - Linux x86_64 CLI binary

### Installation

\`\`\`bash
# Download
curl -L -O https://github.com/$REPO/releases/download/$TAG/php-$VERSION-linux-x64.tar.gz

# Verify checksum
sha256sum -c checksums-$VERSION.txt

# Extract
tar -xzf php-$VERSION-linux-x64.tar.gz
./php-$VERSION --version
\`\`\`

### SHA256 Checksums

\`\`\`
$(cat "$CHECKSUM_FILE")
\`\`\`"

  # Create release
  if [[ "$DRY_RUN" == true ]]; then
    echo -e "  ${YELLOW}[DRY RUN] Would create release $TAG${NC}"
    echo -e "  ${YELLOW}  Archive: $ARCHIVE${NC}"
    echo -e "  ${YELLOW}  Checksum: $(cat "$CHECKSUM_FILE")${NC}"
  else
    echo "  Creating release..."
    
    # Create release and capture output
    if RELEASE_URL=$(gh release create "$TAG" \
      --title "PHP $VERSION" \
      --notes "$RELEASE_BODY" \
      "$ARCHIVE" \
      "$CHECKSUM_FILE" 2>&1); then
      echo -e "  ${GREEN}Created: $RELEASE_URL${NC}"
      CREATED=$((CREATED + 1))
    else
      echo -e "  ${RED}Failed: $RELEASE_URL${NC}"
      FAILED=$((FAILED + 1))
    fi
  fi
  
  echo ""
done

# Summary
echo -e "${BLUE}=================== Summary ===================${NC}"
echo -e "Total versions: $TOTAL"
echo -e "Skipped: $SKIPPED"
if [[ "$DRY_RUN" == true ]]; then
  echo -e "${YELLOW}Dry run - no changes made${NC}"
else
  echo -e "Created: ${GREEN}$CREATED${NC}"
  echo -e "Failed: ${RED}$FAILED${NC}"
fi
echo ""

# Next steps
if [[ "$DRY_RUN" == false ]] && [[ $CREATED -gt 0 ]]; then
  echo -e "${GREEN}Next steps:${NC}"
  echo "1. Run './scripts/update-manifest.sh' to update the version manifest"
  echo "2. Commit and push the updated manifest"
fi
