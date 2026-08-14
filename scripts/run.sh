#!/usr/bin/env bash
# Build a .app so macOS attaches the camera permission to Mirror2, not Terminal.
set -euo pipefail
cd "$(dirname "$0")/.."

APP_NAME=Mirror2
DIST=dist
APP="$DIST/$APP_NAME.app"

echo "→ cargo build"
cargo build

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp target/debug/mirror2 "$APP/Contents/MacOS/$APP_NAME"
cp resources/Info.plist "$APP/Contents/Info.plist"
chmod +x "$APP/Contents/MacOS/$APP_NAME"
touch "$APP"

echo "→ open $APP"
open "$APP"
