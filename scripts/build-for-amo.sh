#!/bin/sh
# Rebuilds dist/firefox (and dist/chrome) exactly as published.
#
#   sh scripts/build-for-amo.sh        (or: npm run build-for-amo)
#
# Works on Linux (x86_64, aarch64) and macOS (arm64, x86_64).
# Needs curl, tar and a C linker for Rust's host tools
# (Ubuntu: sudo apt-get install -y build-essential curl).
#
# Pinned tools are downloaded into ./.tools; if rustup is missing it is
# installed into ~/.cargo and ~/.rustup (the official rustup installer).
set -eu
cd "$(dirname "$0")/.."

BINARYEN=133
BINDGEN=0.2.130
RUST=$(sed -n 's/^channel = "\(.*\)"$/\1/p' rust-toolchain.toml)
TOOLS="$PWD/.tools"
mkdir -p "$TOOLS"

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64)              BY=x86_64-linux;  WB=x86_64-unknown-linux-musl ;;
  Linux-aarch64 | Linux-arm64) BY=aarch64-linux; WB=aarch64-unknown-linux-gnu ;;
  Darwin-arm64)              BY=arm64-macos;   WB=aarch64-apple-darwin ;;
  Darwin-x86_64)             BY=x86_64-macos;  WB=x86_64-apple-darwin ;;
  *) echo "unsupported platform: $(uname -sm)" >&2; exit 1 ;;
esac

fetch() { curl --proto '=https' --tlsv1.2 -fsSL "$1"; }

# 1. Rust, at the version pinned in rust-toolchain.toml
if ! command -v rustup >/dev/null 2>&1; then
  if [ ! -x "$HOME/.cargo/bin/rustup" ]; then
    fetch https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain none
  fi
  PATH="$HOME/.cargo/bin:$PATH"
fi
rustup toolchain install "$RUST" --profile minimal --target wasm32-unknown-unknown

# 2. wasm-opt (binaryen) and wasm-bindgen, prebuilt from their GitHub releases
if [ ! -x "$TOOLS/binaryen-version_$BINARYEN/bin/wasm-opt" ]; then
  fetch "https://github.com/WebAssembly/binaryen/releases/download/version_$BINARYEN/binaryen-version_$BINARYEN-$BY.tar.gz" \
    | tar -xz -C "$TOOLS"
fi
if [ ! -x "$TOOLS/wasm-bindgen-$BINDGEN-$WB/wasm-bindgen" ]; then
  fetch "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/$BINDGEN/wasm-bindgen-$BINDGEN-$WB.tar.gz" \
    | tar -xz -C "$TOOLS"
fi
PATH="$TOOLS/binaryen-version_$BINARYEN/bin:$TOOLS/wasm-bindgen-$BINDGEN-$WB:$PATH"
export PATH

# 3. Build → dist/chrome, dist/firefox
cargo xtask build
echo "Done: dist/firefox"
