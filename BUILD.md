# Build Instructions

This document describes how to build Flashgrep from source.

## Prerequisites

- **Rust** (1.70 or later): [Install Rust](https://rustup.rs/)
- **Git**: For cloning the repository
- **CMake** (Windows): Some dependencies require CMake

## Quick Build

```bash
# Clone repository
git clone https://github.com/nnlgsakib/flashgrep
cd flashgrep

# Build release binary
cargo build --release

# Binary location
./target/release/flashgrep
```

## Platform-Specific Instructions

### Linux

```bash
# Install dependencies (Ubuntu/Debian)
sudo apt-get update
sudo apt-get install -y build-essential cmake pkg-config

# Clone and build
git clone https://github.com/nnlgsakib/flashgrep
cd flashgrep
cargo build --release

# Install system-wide
sudo cp target/release/flashgrep /usr/local/bin/
```

### macOS

```bash
# Install dependencies
brew install cmake

# Clone and build
git clone https://github.com/nnlgsakib/flashgrep
cd flashgrep
cargo build --release

# Install
cp target/release/flashgrep /usr/local/bin/
```

### Windows

```powershell
# Install Visual Studio Build Tools
# Or use Visual Studio Community with C++ tools

# Clone and build
git clone https://github.com/nnlgsakib/flashgrep
cd flashgrep
cargo build --release

# Binary location
.\target\release\flashgrep.exe

# Add to PATH (optional)
# Copy to a directory in your PATH or add target\release to PATH
```

## Build Options

### Debug Build

For development and debugging:

```bash
cargo build
```

Binary: `./target/debug/flashgrep`

### Release Build

Optimized for performance:

```bash
cargo build --release
```

Binary: `./target/release/flashgrep`

**Always use release builds for indexing** - they are 10-20x faster.

### Static Binary

For distribution without dependencies:

```bash
# Linux (musl)
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl

# Windows (static CRT)
RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-pc-windows-msvc
```

## Cross-Compilation

### From Linux to Windows

```bash
# Install cross-compilation toolchain
sudo apt-get install -y mingw-w64

# Add Windows target
rustup target add x86_64-pc-windows-gnu

# Build
 cargo build --release --target x86_64-pc-windows-gnu
```

### From macOS to Linux

```bash
# Install cross tool
cargo install cross

# Build using Docker container
cross build --release --target x86_64-unknown-linux-gnu
```

## Optimization Settings

The `Cargo.toml` includes optimized release settings:

```toml
[profile.release]
opt-level = 3          # Maximum optimization
lto = true             # Link-time optimization
codegen-units = 1      # Single codegen unit for better optimization
panic = "abort"        # Smaller binary size
strip = true           # Strip debug symbols
```

## Binary Size Optimization

To create a smaller binary:

```bash
# Build with size optimization
cargo build --release

# Strip symbols (already done by config)
strip target/release/flashgrep

# Compress with UPX (optional)
upx --best target/release/flashgrep
```

## Testing

### Unit Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Integration Tests

```bash
# Create test repository
mkdir -p /tmp/test-repo
cd /tmp/test-repo
git init

# Create some test files
echo 'fn main() { println!("Hello"); }' > main.rs

# Index and test
flashgrep index
flashgrep start
```

## Troubleshooting

### Build Failures

**Error: `linker cc not found`**
```bash
# Ubuntu/Debian
sudo apt-get install build-essential

# macOS
xcode-select --install
```

**Error: `cmake not found`**
```bash
# Ubuntu/Debian
sudo apt-get install cmake

# macOS
brew install cmake

# Windows
choco install cmake
```

**Error: `sqlite3 not found`**
```bash
# Ubuntu/Debian
sudo apt-get install libsqlite3-dev

# macOS (bundled with system)
# No action needed

# Windows (bundled with rusqlite)
# No action needed
```

### Slow Build Times

Release builds take longer due to optimizations:

```bash
# Use all CPU cores
cargo build --release -j$(nproc)  # Linux

# Or manually specify
cargo build --release -j8
```

### Memory Issues During Build

```bash
# Reduce parallel jobs
cargo build --release -j2

# Or use debug build first
cargo build
```

## Packaging

### Create Distribution Archive

```bash
# Build release
cargo build --release

# Create archive
tar czvf flashgrep-v0.1.0-x86_64-linux.tar.gz \
  -C target/release flashgrep \
  -C ../.. README.md LICENSE

# For Windows
7z a flashgrep-v0.1.0-x86_64-windows.zip \
  .\target\release\flashgrep.exe \
  README.md LICENSE
```

### Docker Build

```dockerfile
# Dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/flashgrep /usr/local/bin/
ENTRYPOINT ["flashgrep"]
```

```bash
# Build Docker image
docker build -t flashgrep:latest .

# Run
docker run -v $(pwd):/workspace -w /workspace flashgrep index
```

## CI/CD

### GitHub Actions

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        target:
          - x86_64-unknown-linux-gnu
          - x86_64-apple-darwin
          - x86_64-pc-windows-msvc
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: ${{ matrix.target }}
      - run: cargo build --release --target ${{ matrix.target }}
      - uses: actions/upload-artifact@v3
        with:
          name: flashgrep-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/flashgrep*
```

## Verification

After building, verify the binary:

```bash
# Check version
./target/release/flashgrep --version

# Check help
./target/release/flashgrep --help

# Test indexing
mkdir -p /tmp/flashgrep-test
cd /tmp/flashgrep-test
echo 'fn main() {}' > test.rs
./target/release/flashgrep index

# Check if index was created
ls -la .flashgrep/
```

## Development Builds

For development with fast compile times:

```bash
# Build with debug symbols but some optimizations
cargo build --profile=release-with-debug

# Binary location
./target/release-with-debug/flashgrep
```

## Minimum Supported Rust Version (MSRV)

- **Current MSRV**: 1.70.0
- **Tested on**: 1.70.0, 1.75.0, stable

To check your Rust version:
```bash
rustc --version
```

## Next Steps

After building:
1. Test the binary: `./target/release/flashgrep --version`
2. Try indexing: `flashgrep index`
3. Read the [README](README.md) for usage instructions
4. Configure your project with `.flashgrepignore`
