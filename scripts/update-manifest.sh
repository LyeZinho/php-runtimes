#!/usr/bin/env bash
set -euo pipefail

# Script to generate hierarchical version manifest from GitHub Releases
# Usage: ./scripts/update-manifest.sh [--dry-run] [--output <path>]

# Parse arguments
DRY_RUN=false
OUTPUT_FILE="manifests/versions.json"
while [[ $# -gt 0 ]]; do
  case $1 in
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    --output)
      OUTPUT_FILE="$2"
      shift 2
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Detect repository from git remote
REPO_URL=$(git remote -v | grep -oP '(?<=:)[^/]+/[^/]+(?=\.git)' | head -1)
if [[ -z "$REPO_URL" ]]; then
  echo "Error: Could not detect GitHub repository from git remote"
  exit 1
fi

OWNER=$(echo "$REPO_URL" | cut -d'/' -f1)
REPO=$(echo "$REPO_URL" | cut -d'/' -f2)

echo "Fetching releases for $OWNER/$REPO..."

# Fetch all releases (paginated)
PAGE=1
ALL_RELEASES="[]"
while true; do
  RESPONSE=$(curl -s "https://api.github.com/repos/$OWNER/$REPO/releases?page=$PAGE&per_page=100")
  
  # Check if empty array
  COUNT=$(echo "$RESPONSE" | jq 'length')
  if [[ "$COUNT" -eq 0 ]]; then
    break
  fi
  
  # Merge releases
  ALL_RELEASES=$(echo "$ALL_RELEASES" "$RESPONSE" | jq -s '.[0] + .[1]')
  
  PAGE=$((PAGE + 1))
done

echo "Found $(echo "$ALL_RELEASES" | jq 'length') releases"

# Process releases into hierarchical structure
MANIFEST=$(echo "$ALL_RELEASES" | jq '
{
  "schema_version": "1.0",
  "updated_at": (now | strftime("%Y-%m-%dT%H:%M:%SZ")),
  "repository": "'"$OWNER/$REPO"'",
  "versions": [
    .[] | {
      "version": (.tag_name | ltrimstr("v")),
      "tag": .tag_name,
      "published_at": .published_at,
      "html_url": .html_url,
      "platforms": [
        .assets[] | select(.name | startswith("php-") and endswith(".tar.gz")) | {
          "platform": (.name | split("-") | .[2:length-1] | join("-")),
          "filename": .name,
          "download_url": .browser_download_url,
          "size_bytes": .size
        }
      ]
    }
  ]
}')

# Add SHA256 checksums by fetching checksums files
echo "Fetching checksums for SHA256 verification..."
FINAL_MANIFEST=$(echo "$MANIFEST" | jq '.versions |= map(.platforms |= map(. + {"sha256": null}))')

# For each version, fetch checksums file and match
VERSIONS=$(echo "$MANIFEST" | jq -r '.versions[].tag')
for TAG in $VERSIONS; do
  VERSION=${TAG#v}
  CHECKSUMS_URL="https://github.com/$OWNER/$REPO/releases/download/$TAG/checksums-$VERSION.txt"
  
  # Download checksums file (may not exist for older releases)
  CHECKSUMS_CONTENT=$(curl -s -f "$CHECKSUMS_URL" 2>/dev/null || echo "")
  
  if [[ -n "$CHECKSUMS_CONTENT" ]]; then
    # Parse checksums and update manifest
    while IFS= read -r LINE; do
      if [[ "$LINE" =~ ^([a-f0-9]{64})\ \ +(.+)$ ]]; then
        SHA="${BASH_REMATCH[1]}"
        FILENAME="${BASH_REMATCH[2]}"
        # Update matching platform in manifest
        FINAL_MANIFEST=$(echo "$FINAL_MANIFEST" | jq --arg tag "$TAG" --arg filename "$FILENAME" --arg sha "$SHA" '
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
      fi
    done <<< "$CHECKSUMS_CONTENT"
  fi
done

# Output
if [[ "$DRY_RUN" == true ]]; then
  echo "Dry run - would write to $OUTPUT_FILE:"
  echo "$FINAL_MANIFEST" | jq .
else
  mkdir -p "$(dirname "$OUTPUT_FILE")"
  echo "$FINAL_MANIFEST" | jq . > "$OUTPUT_FILE"
  echo "Manifest written to $OUTPUT_FILE"
  echo "Summary:"
  echo "  Versions: $(echo "$FINAL_MANIFEST" | jq '.versions | length')"
  echo "  Total platforms: $(echo "$FINAL_MANIFEST" | jq '[.versions[].platforms | length] | add')"
fi
