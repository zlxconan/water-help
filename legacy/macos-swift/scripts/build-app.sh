#!/bin/bash
# 构建 WaterHelp.app：release 编译 → 生成图标 → 组装 bundle → ad-hoc 签名
set -euo pipefail
cd "$(dirname "$0")/.."

echo "==> swift build (release, 通用二进制 x86_64 + arm64)..."
swift build -c release --arch arm64 --arch x86_64

APP="dist/WaterHelp.app"
echo "==> 组装 $APP ..."
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

cp .build/apple/Products/Release/WaterHelp "$APP/Contents/MacOS/WaterHelp"

echo "==> 生成应用图标..."
swift scripts/make_icon.swift dist/AppIcon.iconset
iconutil -c icns dist/AppIcon.iconset -o "$APP/Contents/Resources/AppIcon.icns"

cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key><string>喝水助手</string>
    <key>CFBundleDisplayName</key><string>喝水助手</string>
    <key>CFBundleExecutable</key><string>WaterHelp</string>
    <key>CFBundleIdentifier</key><string>com.waterhelp.mac</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleShortVersionString</key><string>1.0.0</string>
    <key>CFBundleVersion</key><string>1</string>
    <key>CFBundleIconFile</key><string>AppIcon</string>
    <key>CFBundleIconName</key><string>AppIcon</string>
    <key>LSMinimumSystemVersion</key><string>13.0</string>
    <key>LSUIElement</key><true/>
    <key>NSPrincipalClass</key><string>NSApplication</string>
    <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

echo "==> ad-hoc 签名..."
codesign --force --sign - "$APP"

echo "==> 完成: $APP"
du -sh "$APP"
