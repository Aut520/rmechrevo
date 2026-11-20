#!/usr/bin/env bash
# Author: Aut520 <aut@autkc.com>
# Repository: https://github.com/Aut520/rmechrevo
# License: GPL-3.0

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$PROJECT_ROOT"

BASE_VERSION="${1:-}"
REVISION="${2:-}"

if [ -z "$BASE_VERSION" ]; then
    BASE_VERSION="$(grep '^version = ' Cargo.toml | head -n 1 | cut -d '"' -f 2)"
fi
BASE_VERSION="${BASE_VERSION#v}"
BASE_VERSION="${BASE_VERSION:-0.1.0}"

if [ -n "$REVISION" ]; then
    DEB_VERSION="${BASE_VERSION}-${REVISION}"
else
    DEB_VERSION="${BASE_VERSION}"
fi

ARCH="$(dpkg --print-architecture 2>/dev/null || echo amd64)"
PKG_NAME="rmechrevo_${DEB_VERSION}_${ARCH}"
PKG_DIR="build/deb/${PKG_NAME}"
DIST_DIR="dist"
OUTPUT_DEB="${DIST_DIR}/${PKG_NAME}.deb"

rm -rf "$PKG_DIR"
mkdir -p \
    "$DIST_DIR" \
    "$PKG_DIR/DEBIAN" \
    "$PKG_DIR/usr/bin" \
    "$PKG_DIR/usr/lib/systemd/system" \
    "$PKG_DIR/usr/share/applications" \
    "$PKG_DIR/usr/share/icons/hicolor/32x32/apps" \
    "$PKG_DIR/usr/share/icons/hicolor/64x64/apps" \
    "$PKG_DIR/usr/share/icons/hicolor/128x128/apps" \
    "$PKG_DIR/usr/share/icons/hicolor/scalable/apps"

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

INSTALLED_SIZE="$(du -sk "$PKG_DIR/usr" | cut -f1)"

cat << EOF > "$PKG_DIR/DEBIAN/control"
Package: rmechrevo
Version: ${DEB_VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Maintainer: Aut520 <aut@autkc.com>
Homepage: https://github.com/Aut520/rmechrevo
Depends: libc6, libwebkit2gtk-4.1-0, libgtk-3-0, libayatana-appindicator3-1, librsvg2-2, systemd
Installed-Size: ${INSTALLED_SIZE}
Description: RMechrevo - Native Rust Linux Hardware Control Center
 Native Rust Linux hardware control center and privileged daemon for Uniwill (INOU0000) laptops.
 Supports performance profiles, custom power/fan tuning, single-zone RGB backlight, battery charge limits, GPU MUX, and S3/S4 sleep fixes.
EOF

cat << 'EOF' > "$PKG_DIR/DEBIAN/preinst"
#!/bin/sh
set -e

STATE_DAEMON="/run/rmechrevo-daemon.was_running"
STATE_GUI="/run/rmechrevo-gui.was_running"

if systemctl is-active --quiet rmechrevo-daemon.service 2>/dev/null || pgrep -x rmechrevo-daemon >/dev/null 2>&1; then
    touch "$STATE_DAEMON"
    systemctl stop rmechrevo-daemon.service >/dev/null 2>&1 || true
    pkill -x -TERM rmechrevo-daemon >/dev/null 2>&1 || true
    for _ in 1 2 3 4 5 6; do
        pgrep -x rmechrevo-daemon >/dev/null 2>&1 || break
        sleep 0.5
    done
    pgrep -x rmechrevo-daemon >/dev/null 2>&1 && pkill -9 -x rmechrevo-daemon >/dev/null 2>&1 || true
else
    rm -f "$STATE_DAEMON"
fi

GUI_PIDS=$(pgrep -x rmechrevo 2>/dev/null || true)
if [ -n "$GUI_PIDS" ]; then
    rm -f "$STATE_GUI"
    for PID in $GUI_PIDS; do
        if [ -d "/proc/$PID" ]; then
            GUI_USER=$(stat -c '%U' "/proc/$PID" 2>/dev/null || true)
            ENV_FILE="/proc/$PID/environ"
            if [ -f "$ENV_FILE" ] && [ -n "$GUI_USER" ]; then
                W_DISP=$(tr '\0' '\n' < "$ENV_FILE" 2>/dev/null | grep '^WAYLAND_DISPLAY=' | head -n 1 | cut -d= -f2- || true)
                DISP=$(tr '\0' '\n' < "$ENV_FILE" 2>/dev/null | grep '^DISPLAY=' | head -n 1 | cut -d= -f2- || true)
                XDG_RUN=$(tr '\0' '\n' < "$ENV_FILE" 2>/dev/null | grep '^XDG_RUNTIME_DIR=' | head -n 1 | cut -d= -f2- || true)
                DBUS_ADDR=$(tr '\0' '\n' < "$ENV_FILE" 2>/dev/null | grep '^DBUS_SESSION_BUS_ADDRESS=' | head -n 1 | cut -d= -f2- || true)
                DESK=$(tr '\0' '\n' < "$ENV_FILE" 2>/dev/null | grep '^XDG_CURRENT_DESKTOP=' | head -n 1 | cut -d= -f2- || true)
                XAUTH=$(tr '\0' '\n' < "$ENV_FILE" 2>/dev/null | grep '^XAUTHORITY=' | head -n 1 | cut -d= -f2- || true)

                RESTORE_SCRIPT="/run/rmechrevo-restore-${PID}.sh"
                cat << RESTORE_EOF > "$RESTORE_SCRIPT"
#!/bin/sh
if [ "$GUI_USER" = "root" ]; then
    WAYLAND_DISPLAY='$W_DISP' DISPLAY='$DISP' XDG_RUNTIME_DIR='$XDG_RUN' DBUS_SESSION_BUS_ADDRESS='$DBUS_ADDR' XDG_CURRENT_DESKTOP='$DESK' XAUTHORITY='$XAUTH' nohup /usr/bin/rmechrevo >/dev/null 2>&1 &
else
    su - "$GUI_USER" -c "export WAYLAND_DISPLAY='$W_DISP' DISPLAY='$DISP' XDG_RUNTIME_DIR='$XDG_RUN' DBUS_SESSION_BUS_ADDRESS='$DBUS_ADDR' XDG_CURRENT_DESKTOP='$DESK' XAUTHORITY='$XAUTH'; nohup /usr/bin/rmechrevo >/dev/null 2>&1 &" || true
fi
RESTORE_EOF
                chmod 700 "$RESTORE_SCRIPT"
                echo "$RESTORE_SCRIPT" >> "$STATE_GUI"
            fi
        fi
    done
    touch "$STATE_GUI"

    pkill -x -TERM rmechrevo >/dev/null 2>&1 || true
    for _ in 1 2 3 4 5 6; do
        pgrep -x rmechrevo >/dev/null 2>&1 || break
        sleep 0.5
    done
    pgrep -x rmechrevo >/dev/null 2>&1 && pkill -9 -x rmechrevo >/dev/null 2>&1 || true
else
    rm -f "$STATE_GUI"
fi

exit 0
EOF
chmod 755 "$PKG_DIR/DEBIAN/preinst"

cat << 'EOF' > "$PKG_DIR/DEBIAN/postinst"
#!/bin/sh
set -e

STATE_DAEMON="/run/rmechrevo-daemon.was_running"
STATE_GUI="/run/rmechrevo-gui.was_running"

if [ "$1" = "configure" ]; then
    systemctl daemon-reload >/dev/null 2>&1 || true

    if [ -f "$STATE_DAEMON" ]; then
        systemctl restart rmechrevo-daemon.service >/dev/null 2>&1 || true
        rm -f "$STATE_DAEMON"
    elif [ -z "$2" ]; then
        systemctl enable --now rmechrevo-daemon.service >/dev/null 2>&1 || true
    fi

    if [ -f "$STATE_GUI" ]; then
        while IFS= read -r SH_FILE || [ -n "$SH_FILE" ]; do
            if [ -f "$SH_FILE" ] && [ -x "$SH_FILE" ]; then
                /bin/sh "$SH_FILE" >/dev/null 2>&1 || true
                rm -f "$SH_FILE"
            fi
        done < "$STATE_GUI"
        rm -f "$STATE_GUI"
    fi

    command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database -q /usr/share/applications || true
    command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -q /usr/share/icons/hicolor || true
fi

exit 0
EOF
chmod 755 "$PKG_DIR/DEBIAN/postinst"

cat << 'EOF' > "$PKG_DIR/DEBIAN/prerm"
#!/bin/sh
set -e

if [ "$1" = "remove" ] || [ "$1" = "deconfigure" ]; then
    systemctl stop rmechrevo-daemon.service >/dev/null 2>&1 || true
    systemctl disable rmechrevo-daemon.service >/dev/null 2>&1 || true
    pkill -x -TERM rmechrevo-daemon >/dev/null 2>&1 || true
    pkill -x -TERM rmechrevo >/dev/null 2>&1 || true
fi

exit 0
EOF
chmod 755 "$PKG_DIR/DEBIAN/prerm"

cat << 'EOF' > "$PKG_DIR/DEBIAN/postrm"
#!/bin/sh
set -e

if [ "$1" = "purge" ] || [ "$1" = "remove" ]; then
    rm -f /run/rmechrevo-daemon.was_running /run/rmechrevo-gui.was_running /run/rmechrevo-restore-*.sh
    systemctl daemon-reload >/dev/null 2>&1 || true
    command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database -q /usr/share/applications || true
fi

exit 0
EOF
chmod 755 "$PKG_DIR/DEBIAN/postrm"

dpkg-deb --build --root-owner-group "$PKG_DIR" "$OUTPUT_DEB"
(
    cd "$DIST_DIR"
    shopt -s nullglob
    files=(*.deb *.pkg.tar.zst)
    if [ ${#files[@]} -gt 0 ]; then
        sha256sum "${files[@]}" > SHA256SUMS.txt
    fi
)

echo "Built: ${OUTPUT_DEB}"
echo "SHA256: ${DIST_DIR}/SHA256SUMS.txt"
