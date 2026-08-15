#!/bin/bash
set -e

echo "=== Building Mirror2 macOS App Bundle & DMG ==="

APP_DIR="Mirror2.app"
CONTENTS="$APP_DIR/Contents"
MACOS="$CONTENTS/MacOS"
RESOURCES="$CONTENTS/Resources"

rm -rf "$APP_DIR" Mirror2.iconset dmg_staging
mkdir -p "$MACOS" "$RESOURCES" Mirror2.iconset

# 1. Generate AppIcon.icns from resources/icon-window.png
echo "Generating AppIcon.icns..."
sips -z 16 16     resources/icon-window.png --out Mirror2.iconset/icon_16x16.png > /dev/null
sips -z 32 32     resources/icon-window.png --out Mirror2.iconset/icon_16x16@2x.png > /dev/null
sips -z 32 32     resources/icon-window.png --out Mirror2.iconset/icon_32x32.png > /dev/null
sips -z 64 64     resources/icon-window.png --out Mirror2.iconset/icon_32x32@2x.png > /dev/null
sips -z 128 128   resources/icon-window.png --out Mirror2.iconset/icon_128x128.png > /dev/null
sips -z 256 256   resources/icon-window.png --out Mirror2.iconset/icon_128x128@2x.png > /dev/null
sips -z 256 256   resources/icon-window.png --out Mirror2.iconset/icon_256x256.png > /dev/null
sips -z 512 512   resources/icon-window.png --out Mirror2.iconset/icon_256x256@2x.png > /dev/null
sips -z 512 512   resources/icon-window.png --out Mirror2.iconset/icon_512x512.png > /dev/null
sips -z 1024 1024 resources/icon-window.png --out Mirror2.iconset/icon_512x512@2x.png > /dev/null
iconutil -c icns Mirror2.iconset -o "$RESOURCES/AppIcon.icns"
rm -rf Mirror2.iconset

# 2. Copy compiled release binary
echo "Copying release binary..."
cp target/release/mirror2 "$MACOS/Mirror2"
chmod +x "$MACOS/Mirror2"

# 3. Create Info.plist
echo "Writing Info.plist..."
cat << 'EOF' > "$CONTENTS/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>Mirror2</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>com.newtower.mirror2</string>
    <key>CFBundleName</key>
    <string>Mirror2</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSCameraUsageDescription</key>
    <string>Mirror2 requires camera access to capture live video frames and apply realtime analog effects.</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

# 4. Create DMG staging folder with Applications symlink
echo "Staging DMG contents..."
mkdir -p dmg_staging
cp -R "$APP_DIR" dmg_staging/
ln -s /Applications dmg_staging/Applications

# 5. Create DMG on Desktop
DMG_PATH="$HOME/Desktop/Mirror2.dmg"
rm -f "$DMG_PATH"
echo "Creating DMG at $DMG_PATH..."
hdiutil create -volname "Mirror2" -srcfolder dmg_staging -ov -format UDZO "$DMG_PATH"
rm -rf dmg_staging "$APP_DIR"

echo "=== Successfully built $DMG_PATH ==="
