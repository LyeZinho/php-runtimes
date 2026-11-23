# Build environment for fully static Linux PHP CLI binaries (musl, no runtime dependencies).
#
# A glibc build on a rolling distro links against that distro's libicu/libxml2/openssl and
# only runs there. Alpine + static libraries produces one file that runs on any Linux x86_64
# (or aarch64) kernel, which is what `cleanserve` downloads and executes.
FROM alpine:3.21

RUN apk add --no-cache \
    build-base autoconf automake libtool bison re2c pkgconf linux-headers xz tar bash cmake \
    curl-dev curl-static openssl-dev openssl-libs-static \
    zlib-dev zlib-static \
    libxml2-dev libxml2-static xz-dev xz-static c-ares-dev \
    icu-dev icu-static icu-data-full \
    oniguruma-dev \
    bzip2-dev bzip2-static \
    sqlite-dev sqlite-static \
    freetype-dev freetype-static libpng-dev libpng-static \
    libjpeg-turbo-dev libjpeg-turbo-static libwebp-dev libwebp-static \
    libsodium-dev libsodium-static \
    gettext-dev gettext-static \
    brotli-dev brotli-static nghttp2-dev nghttp2-static zstd-dev zstd-static \
    libidn2-dev libidn2-static libunistring-dev libunistring-static libpsl-dev libpsl-static

# Alpine's static libicudata.a is a ~1 KB stub: the locale data lives in a separate .dat file.
# Linked as-is, `intl` loads but every Collator/NumberFormatter fails with U_FILE_ACCESS_ERROR,
# and frameworks that test extension_loaded('intl') (Yii's Formatter, ...) then break. Turn the
# full data file into a real static archive and let the PHP build pick it up first (-L /opt/icudata).
RUN set -e; \
    ICU_MAJOR="$(pkg-config --modversion icu-uc | cut -d. -f1)"; \
    DAT="$(ls /usr/share/icu/*/icudt*l.dat | head -n 1)"; \
    mkdir -p /opt/icudata && cd /opt/icudata; \
    genccode -d . -e "icudt${ICU_MAJOR}" "$DAT"; \
    for f in *.S *.c; do [ -f "$f" ] && gcc -c "$f"; done; \
    ar rcs libicudata.a ./*.o; \
    nm libicudata.a | grep -q "icudt${ICU_MAJOR}_dat" \
      || { echo "ICU data symbol icudt${ICU_MAJOR}_dat missing"; nm libicudata.a | head; exit 1; }; \
    ls -la /opt/icudata

# Alpine ships no static libzip, so build it. The archive hash was checked against two
# independent sources (libzip.org and the GitHub release).
ARG LIBZIP_VERSION=1.11.4
ARG LIBZIP_SHA256=8a247f57d1e3e6f6d11413b12a6f28a9d388de110adc0ec608d893180ed7097b
RUN wget -q "https://libzip.org/download/libzip-${LIBZIP_VERSION}.tar.xz" -O /tmp/libzip.tar.xz \
 && echo "${LIBZIP_SHA256}  /tmp/libzip.tar.xz" | sha256sum -c - \
 && mkdir /tmp/libzip && tar -xJf /tmp/libzip.tar.xz -C /tmp/libzip --strip-components=1 \
 && cmake -S /tmp/libzip -B /tmp/libzip/build -DCMAKE_BUILD_TYPE=Release \
      -DCMAKE_INSTALL_PREFIX=/usr -DBUILD_SHARED_LIBS=OFF \
      -DENABLE_OPENSSL=OFF -DENABLE_GNUTLS=OFF -DENABLE_MBEDTLS=OFF -DENABLE_COMMONCRYPTO=OFF \
      -DENABLE_LZMA=OFF -DENABLE_ZSTD=OFF -DENABLE_BZIP2=ON \
      -DBUILD_TOOLS=OFF -DBUILD_REGRESS=OFF -DBUILD_OSSFUZZ=OFF -DBUILD_EXAMPLES=OFF -DBUILD_DOC=OFF \
 && cmake --build /tmp/libzip/build -j"$(nproc)" && cmake --install /tmp/libzip/build \
 && rm -rf /tmp/libzip /tmp/libzip.tar.xz

WORKDIR /work
