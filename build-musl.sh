#!/usr/bin/env bash
set -euo pipefail

TARGET="x86_64-unknown-linux-musl"

echo "Installing Musl target: ${TARGET}"
rustup target add "${TARGET}"

echo "Building workspace (static musl): ${TARGET}"
cargo build --workspace --profile release-musl --target "${TARGET}"

echo "Stripping binaries (best-effort)"
find "target/${TARGET}/release-musl" \
  -maxdepth 1 \
  -type f \
  -executable \
  -name 'pagi-*' \
  -exec strip {} \; 2>/dev/null || true

echo "Collecting artifacts into dist/musl"
mkdir -p dist/musl
cp "target/${TARGET}/release-musl"/pagi-* dist/musl/ 2>/dev/null || true

echo "Static musl binaries ready in: dist/musl/"

