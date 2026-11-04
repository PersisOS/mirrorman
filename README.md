<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/data/com.parchlinux.mirrorman.svg">
  <img alt="MirrorMan Logo" src="assets/data/com.parchlinux.mirrorman.svg" width="128" height="128">
</picture>

# MirrorMan

**Debian mirror and repository manager** for Persis OS, built with GTK4 and Libadwaita in Rust.

MirrorMan fetches the latest Debian mirrors, tests their speed, and generates an optimized repository sources configuration. It supports both traditional `/etc/apt/sources.list` and modern DEB822 format (`/etc/apt/sources.list.d/debian.sources`). It also manages APT repository components, edits APT settings, syncs packages, and cleans the package cache through a graphical or command-line interface.

## Features

- Fetch and filter Debian mirrors by country, protocol, and IP version
- Test mirror response times concurrently with worker threads
- Support for traditional `/etc/apt/sources.list` and DEB822 (`debian.sources`)
- Toggle standard repository components (`main`, `contrib`, `non-free`, `non-free-firmware`) and suites (`trixie-backports`, `persis-extra`, `liquorix`)
- Add custom Debian repositories with name and repository URL
- Edit APT options (Pipeline Depth, Retries, Install Recommends, Color output)
- Run system updates and clean package cache with PolicyKit privilege escalation
- Iran domestic mirrors for reliable connectivity
- Developer package manager mirrors (`mirrorman-dev` for pip, npm, cargo, go, gem, composer)
- Persian (Farsi) translation support

## Installation

**Persis OS / Debian:**

```bash
sudo apt install mirrorman mirrorman-cli
```

**Build from source:**

```bash
git clone https://github.com/PersisOS/mirrorman.git
cd mirrorman
cargo build --release
```

**Build Debian package (.deb):**

```bash
dpkg-buildpackage -us -uc -b
```

## CLI Usage

MirrorMan includes a headless CLI tool `mirrorman-cli`:

```bash
# Fetch mirrors and test latency
mirrorman-cli refresh
mirrorman-cli test

# Automatically select and save top mirrors
mirrorman-cli best-setup
mirrorman-cli save

# Sync APT repositories
mirrorman-cli sync

# Clean APT package cache
mirrorman-cli clean
```

## License

GNU General Public License v3.0
