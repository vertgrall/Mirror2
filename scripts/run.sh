#!/usr/bin/env bash
# Build a .app so macOS attaches the camera permission to Mirror2, not Terminal.
set -euo pipefail
cd "$(dirname "$0")/.."

APP_NAME=Mirror2
DIST=dist
APP="$DIST/$APP_NAME.app"

if [[ ! -f resources/AppIcon.icns ]]; then
  echo "→ building AppIcon.icns"
  ./scripts/build-icon.sh
fi

echo "→ cargo build"
cargo build

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp target/debug/mirror2 "$APP/Contents/MacOS/$APP_NAME"
cp resources/Info.plist "$APP/Contents/Info.plist"
cp resources/AppIcon.icns "$APP/Contents/Resources/AppIcon.icns"
chmod +x "$APP/Contents/MacOS/$APP_NAME"
touch "$APP"

echo "→ open $APP"
open "$APP"
