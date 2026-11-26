#!/usr/bin/env bash
set -euo pipefail

APP_NAME="Cookie"
APP_BUNDLE="${APP_NAME}.app"
CONTENTS_DIR="${APP_BUNDLE}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"

rm -rf "${APP_BUNDLE}"
mkdir -p "${MACOS_DIR}" "${RESOURCES_DIR}"

cp Resources/Info.plist "${CONTENTS_DIR}/Info.plist"

swiftc \
  -O \
  -module-cache-path /tmp/swift-module-cache \
  -framework AppKit \
  -framework CoreGraphics \
  -framework CoreAudio \
  -framework AudioToolbox \
  Sources/SkinSpec.swift \
  Sources/MusicMonitor.swift \
  Sources/CatPhysics.swift \
  Sources/CatRenderer.swift \
  Sources/OverlayWindow.swift \
  Sources/AppDelegate.swift \
  Sources/main.swift \
  -o "${MACOS_DIR}/${APP_NAME}"

echo "Built ${APP_BUNDLE} successfully ($(du -sh "${MACOS_DIR}/${APP_NAME}" | cut -f1))."
