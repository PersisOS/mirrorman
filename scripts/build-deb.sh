#!/usr/bin/env bash
set -euo pipefail

# Build script for Debian package (.deb)
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

VERSION=$(grep -m1 '^version = ' crates/mirrorman/Cargo.toml | cut -d '"' -f 2)
ARCH="$(dpkg --print-architecture 2>/dev/null || echo "amd64")"
BUILD_DIR="${PROJECT_ROOT}/target/debian-pkg"
OUTPUT_DIR="${PROJECT_ROOT}/target/debian-dist"

echo "[*] Building MirrorMan v${VERSION} for Debian (${ARCH})..."

# Check if dpkg-buildpackage is available
if command -v dpkg-buildpackage >/dev/null 2>&1; then
    echo "[*] Using dpkg-buildpackage..."
    dpkg-buildpackage -us -uc -b
    mkdir -p "${OUTPUT_DIR}"
    mv ../mirrorman*.deb ../mirrorman*.buildinfo ../mirrorman*.changes "${OUTPUT_DIR}/" 2>/dev/null || true
    echo "[+] Packages generated in ${OUTPUT_DIR}/"
    exit 0
fi

echo "[*] dpkg-buildpackage not found; compiling release binaries via cargo..."
cargo build --release --workspace

mkdir -p "${OUTPUT_DIR}"
rm -rf "${BUILD_DIR}"

# 1. Package: mirrorman
echo "[*] Assembling mirrorman package..."
PKG_ROOT="${BUILD_DIR}/mirrorman"
mkdir -p "${PKG_ROOT}/DEBIAN"
mkdir -p "${PKG_ROOT}/usr/bin"
mkdir -p "${PKG_ROOT}/usr/share/applications"
mkdir -p "${PKG_ROOT}/usr/share/icons/hicolor/scalable/apps"
mkdir -p "${PKG_ROOT}/usr/share/metainfo"
mkdir -p "${PKG_ROOT}/usr/share/polkit-1/actions"
mkdir -p "${PKG_ROOT}/usr/share/dbus-1/system-services"
mkdir -p "${PKG_ROOT}/usr/share/dbus-1/system.d"
mkdir -p "${PKG_ROOT}/usr/lib/systemd/system"
mkdir -p "${PKG_ROOT}/usr/lib/systemd/user"

# Binaries
cp target/release/mirrorman "${PKG_ROOT}/usr/bin/"
cp target/release/mirrorman-helper "${PKG_ROOT}/usr/bin/"
cp target/release/mirrorman-dev "${PKG_ROOT}/usr/bin/"

# Assets
cp assets/data/com.parchlinux.mirrorman.desktop "${PKG_ROOT}/usr/share/applications/"
cp assets/data/com.parchlinux.mirrorman.dev.desktop "${PKG_ROOT}/usr/share/applications/"
cp assets/data/com.parchlinux.mirrorman.svg "${PKG_ROOT}/usr/share/icons/hicolor/scalable/apps/"
cp assets/data/com.parchlinux.mirrorman.dev.svg "${PKG_ROOT}/usr/share/icons/hicolor/scalable/apps/"
cp assets/data/com.parchlinux.mirrorman.metainfo.xml "${PKG_ROOT}/usr/share/metainfo/"
cp assets/data/com.parchlinux.mirrorman.policy "${PKG_ROOT}/usr/share/polkit-1/actions/"
cp assets/data/com.parchlinux.mirrorman.Helper.service "${PKG_ROOT}/usr/share/dbus-1/system-services/"
cp assets/data/com.parchlinux.mirrorman-helper.conf "${PKG_ROOT}/usr/share/dbus-1/system.d/"
cp assets/data/mirrorman-helper.service "${PKG_ROOT}/usr/lib/systemd/system/"
cp assets/data/mirrorman-refresh.service "${PKG_ROOT}/usr/lib/systemd/user/"
cp assets/data/mirrorman-refresh.timer "${PKG_ROOT}/usr/lib/systemd/user/"

cat <<EOF > "${PKG_ROOT}/DEBIAN/control"
Package: mirrorman
Version: ${VERSION}
Section: admin
Priority: optional
Architecture: ${ARCH}
Depends: pkexec, dbus, apt
Recommends: mirrorman-cli
Maintainer: Persis OS Team <info@persisos.org>
Description: Debian mirror and repository manager
 MirrorMan is a GTK4 / Libadwaita mirror and repository management application
 for Debian GNU/Linux and Persis OS.
EOF

dpkg-deb --build --root-owner-group "${PKG_ROOT}" "${OUTPUT_DIR}/mirrorman_${VERSION}_${ARCH}.deb"

# 2. Package: mirrorman-cli
echo "[*] Assembling mirrorman-cli package..."
CLI_ROOT="${BUILD_DIR}/mirrorman-cli"
mkdir -p "${CLI_ROOT}/DEBIAN"
mkdir -p "${CLI_ROOT}/usr/bin"
mkdir -p "${CLI_ROOT}/usr/share/man/man1"

cp target/release/mirrorman-cli "${CLI_ROOT}/usr/bin/"
cp assets/man/mirrorman-cli.1 "${CLI_ROOT}/usr/share/man/man1/"

cat <<EOF > "${CLI_ROOT}/DEBIAN/control"
Package: mirrorman-cli
Version: ${VERSION}
Section: admin
Priority: optional
Architecture: ${ARCH}
Depends: dbus, apt
Maintainer: Persis OS Team <info@persisos.org>
Description: Debian mirror and repository manager CLI
 Headless command-line interface for MirrorMan on Debian GNU/Linux and Persis OS.
EOF

dpkg-deb --build --root-owner-group "${CLI_ROOT}" "${OUTPUT_DIR}/mirrorman-cli_${VERSION}_${ARCH}.deb"

echo "[+] Successfully built Debian packages in ${OUTPUT_DIR}/:"
ls -lh "${OUTPUT_DIR}"/*.deb
