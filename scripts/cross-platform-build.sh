#!/usr/bin/env bash
set -euo pipefail

# Cross-platform PHP build script
# Supports: Linux x64, Linux ARM64, Windows x64 (via Docker), macOS (native only)
#
# Usage:
#   ./scripts/cross-platform-build.sh --version 8.5.4 --platform linux-x64
#   ./scripts/cross-platform-build.sh --version 8.5.4 --platform windows-x64 --docker-image maxrd2/arch-mingw
#   ./scripts/cross-platform-build.sh --version 8.5.4 --platform all

# Default configuration
DEFAULT_DOCKER_IMAGE="maxrd2/arch-mingw"
PHP_CONFIGURE_OPTS="--disable-all --enable-cli --enable-json --enable-mbstring --enable-phar --enable-tokenizer --without-pear --disable-cgi"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
VERSION=""
PLATFORM=""
DOCKER_IMAGE="$DEFAULT_DOCKER_IMAGE"
SOURCE_DIR=""
OUTPUT_DIR="builds"
DRY_RUN=false
JOBS=$(nproc 2>/dev/null || echo 4)

while [[ $# -gt 0 ]]; do
  case $1 in
    --version)
      VERSION="$2"
      shift 2
      ;;
    --platform)
      PLATFORM="$2"
      shift 2
      ;;
    --docker-image)
      DOCKER_IMAGE="$2"
      shift 2
      ;;
    --source-dir)
      SOURCE_DIR="$2"
      shift 2
      ;;
    --output-dir)
      OUTPUT_DIR="$2"
      shift 2
      ;;
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    --jobs|-j)
      JOBS="$2"
      shift 2
      ;;
    --help|-h)
      echo "Cross-platform PHP build script"
      echo ""
      echo "Usage: $0 [OPTIONS]"
      echo ""
      echo "Options:"
      echo "  --version VERSION        PHP version to build (e.g., 8.5.4)"
      echo "  --platform PLATFORM      Target platform: linux-x64, linux-arm64, windows-x64, macos-x64, macos-arm64, all"
      echo "  --docker-image IMAGE     Docker image for Windows cross-compilation (default: $DEFAULT_DOCKER_IMAGE)"
      echo "  --source-dir DIR         PHP source directory (default: builds/php-VERSION)"
      echo "  --output-dir DIR         Output directory (default: builds)"
      echo "  --dry-run                Show commands without executing"
      echo "  --jobs, -j N             Number of parallel jobs (default: $JOBS)"
      echo "  --help, -h               Show this help"
      echo ""
      echo "Examples:"
      echo "  $0 --version 8.5.4 --platform linux-x64"
      echo "  $0 --version 8.5.4 --platform windows-x64 --docker-image maxrd2/arch-mingw"
      echo "  $0 --version 8.5.4 --platform all"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Validate arguments
if [[ -z "$VERSION" ]]; then
  echo -e "${RED}Error: --version is required${NC}"
  exit 1
fi

if [[ -z "$PLATFORM" ]]; then
  echo -e "${RED}Error: --platform is required${NC}"
  exit 1
fi

# Set source directory if not provided
if [[ -z "$SOURCE_DIR" ]]; then
  SOURCE_DIR="builds/php-$VERSION"
fi

# Check if source directory exists
if [[ ! -d "$SOURCE_DIR" ]]; then
  echo -e "${YELLOW}Source directory $SOURCE_DIR not found, downloading PHP $VERSION...${NC}"
  
  # Download PHP source
  PHP_URL="https://www.php.net/distributions/php-$VERSION.tar.gz"
  mkdir -p "$SOURCE_DIR"
  
  echo "Downloading from $PHP_URL..."
  curl -L "$PHP_URL" -o "/tmp/php-$VERSION.tar.gz"
  
  echo "Extracting..."
  tar -xzf "/tmp/php-$VERSION.tar.gz" -C "$SOURCE_DIR" --strip-components=1
  
  echo -e "${GREEN}Downloaded PHP $VERSION${NC}"
fi

# Platform-specific build functions
build_linux_x64() {
  echo -e "${BLUE}Building PHP $VERSION for Linux x64...${NC}"
  
  local output_dir="$OUTPUT_DIR/linux-x64"
  mkdir -p "$output_dir"
  
  cd "$SOURCE_DIR"
  
  if [[ "$DRY_RUN" == true ]]; then
    echo "Would run: ./configure $PHP_CONFIGURE_OPTS && make -j$JOBS && cp sapi/cli/php $output_dir/php-$VERSION"
    return 0
  fi
  
  ./configure $PHP_CONFIGURE_OPTS || return 1
  make -j"$JOBS" || return 1
  cp sapi/cli/php "$output_dir/php-$VERSION"
  
  echo -e "${GREEN}Built: $output_dir/php-$VERSION${NC}"
}

build_linux_arm64() {
  echo -e "${BLUE}Building PHP $VERSION for Linux ARM64...${NC}"
  
  local output_dir="$OUTPUT_DIR/linux-arm64"
  mkdir -p "$output_dir"
  
  cd "$SOURCE_DIR"
  
  if [[ "$DRY_RUN" == true ]]; then
    echo "Would run: ./configure --host=aarch64-linux-gnu $PHP_CONFIGURE_OPTS && make -j$JOBS && cp sapi/cli/php $output_dir/php-$VERSION"
    return 0
  fi
  
  ./configure --host=aarch64-linux-gnu $PHP_CONFIGURE_OPTS || return 1
  make -j"$JOBS" || return 1
  cp sapi/cli/php "$output_dir/php-$VERSION"
  
  echo -e "${GREEN}Built: $output_dir/php-$VERSION${NC}"
}

build_windows_x64() {
  echo -e "${BLUE}Building PHP $VERSION for Windows x64 (via Docker)...${NC}"
  
  local output_dir="$OUTPUT_DIR/windows-x64"
  mkdir -p "$output_dir"
  
  local abs_source=$(realpath "$SOURCE_DIR")
  local abs_output=$(realpath "$output_dir")
  
  echo "Using Docker image: $DOCKER_IMAGE"
  
  if [[ "$DRY_RUN" == true ]]; then
    echo "Would run: docker run --rm -v $abs_source:/work -v $abs_output:/output $DOCKER_IMAGE bash -c '...'"
    return 0
  fi
  
  # Check if Docker is available
  if ! command -v docker &> /dev/null; then
    echo -e "${RED}Error: Docker is not installed${NC}"
    return 1
  fi
  
  # Pull image if not exists
  docker pull "$DOCKER_IMAGE" || true
  
  # Build PHP with cross-compilation
  docker run --rm \
    -v "$abs_source:/work" \
    -v "$abs_output:/output" \
    "$DOCKER_IMAGE" \
    bash -c "
      set -e
      cd /work
      
      # Configure with Windows target
      ./configure \
        --host=x86_64-w64-mingw32 \
        --prefix=/tmp/install \
        $PHP_CONFIGURE_OPTS \
        CFLAGS=\"-D_WIN32_WINNT=0x0600\" \
        LIBS=\"-lws2_32\" \
        || exit 1
      
      # Build
      make -j$(nproc) || exit 1
      
      # Copy binary
      cp sapi/cli/php.exe /output/php-$VERSION.exe
      
      echo 'Windows build complete!'
    " || return 1
  
  echo -e "${GREEN}Built: $output_dir/php-$VERSION.exe${NC}"
}

build_macos_x64() {
  echo -e "${BLUE}Building PHP $VERSION for macOS x64...${NC}"
  
  local output_dir="$OUTPUT_DIR/macos-x64"
  mkdir -p "$output_dir"
  
  cd "$SOURCE_DIR"
  
  if [[ "$DRY_RUN" == true ]]; then
    echo "Would run: CFLAGS='-mmacosx-version-min=10.15' ./configure $PHP_CONFIGURE_OPTS && make -j$JOBS && cp sapi/cli/php $output_dir/php-$VERSION"
    return 0
  fi
  
  CFLAGS="-mmacosx-version-min=10.15" ./configure $PHP_CONFIGURE_OPTS || return 1
  make -j"$JOBS" || return 1
  cp sapi/cli/php "$output_dir/php-$VERSION"
  
  echo -e "${GREEN}Built: $output_dir/php-$VERSION${NC}"
}

build_macos_arm64() {
  echo -e "${BLUE}Building PHP $VERSION for macOS ARM64...${NC}"
  
  local output_dir="$OUTPUT_DIR/macos-arm64"
  mkdir -p "$output_dir"
  
  cd "$SOURCE_DIR"
  
  if [[ "$DRY_RUN" == true ]]; then
    echo "Would run: CFLAGS='-mmacosx-version-min=10.15 -arch arm64' ./configure $PHP_CONFIGURE_OPTS && make -j$JOBS && cp sapi/cli/php $output_dir/php-$VERSION"
    return 0
  fi
  
  CFLAGS="-mmacosx-version-min=10.15 -arch arm64" ./configure $PHP_CONFIGURE_OPTS || return 1
  make -j"$JOBS" || return 1
  cp sapi/cli/php "$output_dir/php-$VERSION"
  
  echo -e "${GREEN}Built: $output_dir/php-$VERSION${NC}"
}

# Execute build based on platform
case "$PLATFORM" in
  linux-x64)
    build_linux_x64
    ;;
  linux-arm64)
    build_linux_arm64
    ;;
  windows-x64)
    build_windows_x64
    ;;
  macos-x64)
    build_macos_x64
    ;;
  macos-arm64)
    build_macos_arm64
    ;;
  all)
    echo -e "${BLUE}Building PHP $VERSION for all platforms...${NC}"
    
    # Linux x64 (native)
    build_linux_x64 || echo -e "${RED}Failed: linux-x64${NC}"
    
    # Windows x64 (via Docker)
    build_windows_x64 || echo -e "${YELLOW}Skipped: windows-x64 (Docker not available or build failed)${NC}"
    
    # Note: Other platforms require cross-compilation toolchains
    echo -e "${YELLOW}Note: linux-arm64, macos-x64, macos-arm64 require cross-compilation toolchains${NC}"
    ;;
  *)
    echo -e "${RED}Error: Unknown platform '$PLATFORM'${NC}"
    echo "Valid platforms: linux-x64, linux-arm64, windows-x64, macos-x64, macos-arm64, all"
    exit 1
    ;;
esac

echo -e "${GREEN}Done!${NC}"
