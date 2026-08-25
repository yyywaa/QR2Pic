#!/bin/bash
set -e

echo "=== Installing system dependencies for QR2Pic ==="
echo "This script installs pkg-config and OpenSSL development libraries"
echo "Only needed if you want to use native-tls instead of rustls"

sudo apt update
sudo apt install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    curl \
    build-essential

echo "=== Dependencies installed ==="
echo "Now you can build with native-tls (if configured)"