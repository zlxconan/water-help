#!/bin/bash
# 把已编译好的 waterhelp 通用二进制组装成 macOS .app（本地与 CI 共用）
# 用法: bash scripts/bundle-macos.sh [二进制路径，默认 src-tauri/waterhelp-universal]
set -euo pipefail
cd "$(dirname "$0")/.."

BIN="${1:-src-tauri/waterhelp-universal}"
if [ ! -f "$BIN" ]; then
  echo "找不到二进制 $BIN，请先编译（见 scripts/build-macos.sh）" >&2
  exit 1
fi

APP="dist/WaterHelp.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

cp "$BIN" "$APP/Contents/MacOS/WaterHelp"
cp src-tauri/icons/icon.icns "$APP/Contents/Resources/AppIcon.icns"

cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key><string>喝水助手</string>
    <key>CFBundleDisplayName</key><string>喝水助手</string>
    <key>CFBundleExecutable</key><string>WaterHelp</string>
    <key>CFBundleIdentifier</key><string>com.waterhelp.desktop</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleShortVersionString</key><string>1.0.0</string>
    <key>CFBundleVersion</key><string>1</string>
    <key>CFBundleIconFile</key><string>AppIcon</string>
    <key>CFBundleIconName</key><string>AppIcon</string>
    <key>LSMinimumSystemVersion</key><string>10.15</string>
    <key>NSPrincipalClass</key><string>NSApplication</string>
    <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

codesign --force --sign - "$APP"
echo "==> 完成: $APP"
du -sh "$APP"
