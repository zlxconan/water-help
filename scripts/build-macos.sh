#!/bin/bash
# 本地一键构建 macOS 通用版：双架构编译 → lipo 合并 → 组装 .app
set -euo pipefail
cd "$(dirname "$0")/.."

# rustup 安装的 cargo 不在 PATH 时兜底
export PATH="$HOME/.cargo/bin:$PATH"

cd src-tauri
cargo build --release --target x86_64-apple-darwin --target aarch64-apple-darwin
lipo -create \
  -output waterhelp-universal \
  target/x86_64-apple-darwin/release/waterhelp \
  target/aarch64-apple-darwin/release/waterhelp
cd ..

bash scripts/bundle-macos.sh src-tauri/waterhelp-universal
