#!/usr/bin/env bash
# Author: Aut520 <aut@autkc.com>
# Repository: https://github.com/Aut520/rmechrevo
# License: GPL-3.0

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$PROJECT_ROOT"

BASE_VERSION="${1:-}"
PKGREL="${2:-1}"

if [ -z "$BASE_VERSION" ]; then
    BASE_VERSION="$(grep '^version = ' Cargo.toml | head -n 1 | cut -d '"' -f 2)"
fi
BASE_VERSION="${BASE_VERSION#v}"
BASE_VERSION="${BASE_VERSION:-0.1.0}"
BASE_VERSION="${BASE_VERSION//-/.}"
PKGREL="${PKGREL:-1}"

ARCH="$(uname -m)"
PKGVER="${BASE_VERSION}-${PKGREL}"
PKG_NAME="rmechrevo-${PKGVER}-${ARCH}"
PKG_DIR="build/arch/${PKG_NAME}"
DIST_DIR="dist"
OUTPUT_PKG="${DIST_DIR}/${PKG_NAME}.pkg.tar.zst"

command -v zstd >/dev/null 2>&1 || {
    echo "缺少 zstd，请安装：sudo apt install -y zstd (或 pacman -S zstd)"
    exit 1
}

rm -rf "$PKG_DIR"
mkdir -p \
    "$DIST_DIR" \
    "$PKG_DIR/usr/bin" \
    "$PKG_DIR/usr/lib/systemd/system" \
    "$PKG_DIR/usr/share/applications" \
    "$PKG_DIR/usr/share/icons/hicolor/32x32/apps" \
    "$PKG_DIR/usr/share/icons/hicolor/64x64/apps" \
    "$PKG_DIR/usr/share/icons/hicolor/128x128/apps" \
    "$PKG_DIR/usr/share/icons/hicolor/scalable/apps" \
    "$PKG_DIR/usr/share/licenses/rmechrevo"

if [ ! -f "target/release/rmechrevo" ] || [ ! -f "target/release/rmechrevo-daemon" ]; then
    cargo build --release --bin rmechrevo --bin rmechrevo-daemon
fi

install -m 755 target/release/rmechrevo "$PKG_DIR/usr/bin/rmechrevo"
install -m 755 target/release/rmechrevo-daemon "$PKG_DIR/usr/bin/rmechrevo-daemon"
install -m 644 packaging/linux/rmechrevo-daemon.service "$PKG_DIR/usr/lib/systemd/system/rmechrevo-daemon.service"
install -m 644 packaging/linux/rmechrevo.desktop "$PKG_DIR/usr/share/applications/rmechrevo.desktop"
install -m 644 icons/32x32.png "$PKG_DIR/usr/share/icons/hicolor/32x32/apps/rmechrevo.png"
install -m 644 icons/64x64.png "$PKG_DIR/usr/share/icons/hicolor/64x64/apps/rmechrevo.png"
install -m 644 icons/128x128.png "$PKG_DIR/usr/share/icons/hicolor/128x128/apps/rmechrevo.png"
install -m 644 icons/icon.svg "$PKG_DIR/usr/share/icons/hicolor/scalable/apps/rmechrevo.svg"
install -m 644 LICENSE "$PKG_DIR/usr/share/licenses/rmechrevo/LICENSE"
install -m 644 packaging/arch/rmechrevo.install "$PKG_DIR/.INSTALL"

INSTALLED_BYTES="$(du -sb "$PKG_DIR/usr" | cut -f1)"
BUILDDATE="$(date +%s)"

cat << EOF > "$PKG_DIR/.PKGINFO"
pkgname = rmechrevo
pkgbase = rmechrevo
pkgver = ${PKGVER}
pkgdesc = Native Rust Linux hardware control center and privileged daemon for Uniwill (INOU0000) laptops
url = https://github.com/Aut520/rmechrevo
builddate = ${BUILDDATE}
packager = Aut520 <aut@autkc.com>
size = ${INSTALLED_BYTES}
arch = ${ARCH}
license = GPL-3.0-only
depend = glibc
depend = gtk3
depend = webkit2gtk-4.1
depend = libayatana-appindicator
depend = librsvg
depend = systemd
optdepend = acpi_call-dkms: Required for EC/ACPI register read/write via /proc/acpi/call
optdepend = nvidia-utils: GPU frequency offset and S4 hibernate VRAM preservation
EOF

tar --owner=0 --group=0 --numeric-owner -C "$PKG_DIR" -cf - .PKGINFO .INSTALL usr | zstd -f -q -19 -T0 -o "$OUTPUT_PKG"

(
    cd "$DIST_DIR"
    shopt -s nullglob
    files=(*.deb *.pkg.tar.zst)
    if [ ${#files[@]} -gt 0 ]; then
        sha256sum "${files[@]}" > SHA256SUMS.txt
    fi
)

echo "Built (Arch experimental): ${OUTPUT_PKG}"
echo "SHA256: ${DIST_DIR}/SHA256SUMS.txt"
