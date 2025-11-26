# Battle Brothers CLI Patcher

A cross-platform command-line tool for patching Battle Brothers with the 4GB (Large Address Aware) patch and creating mod preload files. Works on Linux (with WINE) and Windows.

## Features

- **4GB/LAA Patch**: Allows the game to use up to 4GB of RAM instead of 2GB, necessary for heavy mod lists
- **Mod Preload**: Scans mods and creates the `~mod_msu_launcher.zip` preload file
- **Version Detection**: Automatically detects Steam, GOG, or already-patched versions
- **Cross-platform**: Works on Linux and Windows (no WINE GUI required!)

## Installation

### From Source

```bash
# Install Rust: https://www.rust-lang.org/tools/install
cargo install --git https://github.com/stream-enterer/MSU-Launcher
```

### Pre-built Binaries

Download from the [Releases](https://github.com/stream-enterer/MSU-Launcher/releases) page.

## Usage

```bash
# Show help
bb-patcher --help

# Apply 4GB patch (auto-detects game location via Steam)
bb-patcher patch4gb

# Apply 4GB patch with explicit path
bb-patcher patch4gb --path /path/to/Battle\ Brothers

# Create mod preload file
bb-patcher preload --path /path/to/Battle\ Brothers

# Run both patches
bb-patcher all --path /path/to/Battle\ Brothers

# Detect game version without making changes
bb-patcher detect --path /path/to/Battle\ Brothers

# Check if already patched
bb-patcher check --path /path/to/Battle\ Brothers

# Set game path (saved to config file)
bb-patcher set-path /path/to/Battle\ Brothers

# Show current configuration
bb-patcher config
```

## Steam Version Notes

The Steam version has DRM protection that must be removed before patching. Options:

1. **Recommended**: Use [Steamless](https://github.com/atom0s/Steamless) on Windows first, then use this tool
2. Use the `--skip-steam-drm` flag to patch anyway (may not work correctly)
3. Use the GOG version which has no DRM

## Building from Source

```bash
# Clone the repository
git clone https://github.com/stream-enterer/MSU-Launcher
cd MSU-Launcher

# Build release binary
cargo build --release

# Binary will be at target/release/bb-patcher
```

### Build without Steam auto-detection

If you don't want the Steam library detection feature:

```bash
cargo build --release --no-default-features
```

## License

See the original repository for license information.
