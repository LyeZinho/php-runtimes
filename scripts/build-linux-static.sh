#!/usr/bin/env bash
# Build a fully static Linux PHP CLI binary inside an Alpine (musl) container.
#
# Usage: scripts/build-linux-static.sh <php-version> [--tarball FILE] [--out FILE] [--flags FILE]
#   --tarball  PHP source archive (default: sources/php-<version>.tar.xz)
#   --out      where to write the binary (default: builds/linux-x64/php-<version>)
#   --flags    file with one ./configure flag per line
#              (default: output of `php-curator configure-flags`)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGE="php-runtimes-builder-linux-static"

VERSION="${1:-}"
[ -n "$VERSION" ] || { sed -n '2,9p' "$0" | sed 's/^# \{0,1\}//'; exit 1; }
shift

TARBALL="$ROOT/sources/php-$VERSION.tar.xz"
OUT="$ROOT/builds/linux-x64/php-$VERSION"
FLAGS_FILE=""
while [ $# -gt 0 ]; do
  case "$1" in
    --tarball) TARBALL="$2"; shift 2 ;;
    --out)     OUT="$2"; shift 2 ;;
    --flags)   FLAGS_FILE="$2"; shift 2 ;;
    *) echo "unknown option: $1" >&2; exit 1 ;;
  esac
done

[ -f "$TARBALL" ] || { echo "error: source archive not found: $TARBALL" >&2; exit 1; }
command -v docker >/dev/null 2>&1 || { echo "error: docker is required" >&2; exit 1; }

WORK="$(mktemp -d "${TMPDIR:-/tmp}/php-static.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

if [ -z "$FLAGS_FILE" ]; then
  FLAGS_FILE="$WORK/flags.txt"
  if [ -x "$ROOT/target/release/php-curator" ]; then
    "$ROOT/target/release/php-curator" configure-flags --platform linux-x64 > "$FLAGS_FILE"
  else
    (cd "$ROOT" && cargo run --release --quiet -- configure-flags --platform linux-x64) > "$FLAGS_FILE"
  fi
fi

echo "==> Building image $IMAGE"
docker build -q -t "$IMAGE" "$ROOT/docker/linux-static" >/dev/null

mkdir -p "$(dirname "$OUT")"
cp "$TARBALL" "$WORK/php.tar.xz"
cp "$FLAGS_FILE" "$WORK/flags.txt" 2>/dev/null || true

cat > "$WORK/inner.sh" <<'INNER'
set -euo pipefail
mkdir -p /build && cd /build
tar -xJf /work/php.tar.xz --strip-components=1

# Flags that only make sense for a dynamic build are dropped; the rest comes from the curator.
mapfile -t FLAGS < <(grep -v '^$' /work/flags.txt | sort -u)

export PKG_CONFIG="pkg-config --static"
export CFLAGS="-Os -fno-pie"
# /opt/icudata holds libicudata.a with the real locale data; it must win over /usr/lib's stub.
export LDFLAGS="-L/opt/icudata -static"
export LIBS="-lstdc++"
export CXXFLAGS="-Os"

./configure --prefix=/opt/php "${FLAGS[@]}" --disable-opcache-jit >/work/configure.log 2>&1 \
  || { cp config.log /work/config.log 2>/dev/null || true; tail -40 /work/configure.log; exit 1; }
make -j"$(nproc)" EXTRA_LDFLAGS_PROGRAM="-all-static" >/work/make.log 2>&1 \
  || { tail -60 /work/make.log; exit 1; }
strip sapi/cli/php
cp sapi/cli/php /work/php.out
INNER

echo "==> Compiling PHP $VERSION (static, musl). This takes a few minutes."
if ! docker run --rm -v "$WORK:/work" "$IMAGE" bash /work/inner.sh; then
  # Keep the logs: $WORK is deleted on exit and they are the only way to see why it failed.
  for f in config.log configure.log make.log; do
    [ -f "$WORK/$f" ] && cp "$WORK/$f" "$OUT.failed-$f"
  done
  echo "error: build failed; logs saved next to $OUT as *.failed-*.log" >&2
  exit 1
fi

cp "$WORK/php.out" "$OUT"
chmod +x "$OUT"
echo "==> $OUT ($(du -h "$OUT" | cut -f1))"
