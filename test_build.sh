#!/bin/bash
set -e

echo "=== Testing QR2Pic build ==="
echo "Current directory: $(pwd)"

echo "=== 1. Checking Rust toolchain ==="
rustc --version
cargo --version

echo "=== 2. Updating cargo dependencies ==="
cargo update

echo "=== 3. Building in debug mode ==="
cargo build --verbose

echo "=== 4. Type checking ==="
cargo check

echo "=== 5. Building in release mode ==="
cargo build --release

echo "=== 6. Checking binary size ==="
ls -lh target/debug/qr2pic target/release/qr2pic 2>/dev/null || echo "Binaries not found"

echo "=== Build successful! ==="
echo "To run the server (with proper environment variables):"
echo "  cp .env.example .env"
echo "  # Edit .env with your actual configuration"
echo "  cargo run"