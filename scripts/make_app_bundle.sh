#!/usr/bin/env bash
# Creates tslime.app — a minimal macOS app bundle wrapping the gui-featured binary.
# Usage: ./scripts/make_app_bundle.sh [--debug]
set -euo pipefail

PROFILE="release"
CARGO_FLAGS="--release"
if [[ "${1:-}" == "--debug" ]]; then
    PROFILE="debug"
    CARGO_FLAGS=""
fi

BINARY_SRC="target/${PROFILE}/tslime"
APP_DIR="tslime.app"
CONTENTS="${APP_DIR}/Contents"
MACOS="${CONTENTS}/MacOS"

echo "Building tslime with --features gui (${PROFILE})..."
cargo build ${CARGO_FLAGS} --features gui

echo "Creating app bundle at ${APP_DIR}..."
rm -rf "${APP_DIR}"
mkdir -p "${MACOS}"

cp "${BINARY_SRC}" "${MACOS}/tslime"

cat > "${CONTENTS}/Info.plist" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
    "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>tslime</string>
    <key>CFBundleIdentifier</key>
    <string>com.tslime.app</string>
    <key>CFBundleName</key>
    <string>tslime</string>
    <key>CFBundleDisplayName</key>
    <string>tslime</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
</dict>
</plist>
EOF

echo "Done: ${APP_DIR}"
echo "Double-click ${APP_DIR} in Finder to launch the GUI window."
