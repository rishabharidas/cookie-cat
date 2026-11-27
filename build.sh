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
if [ -f "Resources/AppIcon.icns" ]; then
  cp Resources/AppIcon.icns "${RESOURCES_DIR}/AppIcon.icns"
fi

SOURCES=(
  Sources/SkinSpec.swift
  Sources/MusicMonitor.swift
  Sources/CatPhysics.swift
  Sources/CatRenderer.swift
  Sources/OverlayWindow.swift
  Sources/AppDelegate.swift
  Sources/main.swift
)

FRAMEWORKS=(
  -framework AppKit
  -framework CoreGraphics
  -framework CoreAudio
  -framework AudioToolbox
  -framework ServiceManagement
)

X86_TARGET="x86_64-apple-macos15.0"
if [[ "$(xcode-select -p 2>/dev/null)" == *"Xcode.app"* ]]; then
  X86_TARGET="x86_64-apple-macos12.0"
fi

swiftc -O -target arm64-apple-macos12.0 -module-cache-path /tmp/swift-module-cache "${FRAMEWORKS[@]}" "${SOURCES[@]}" -o "${MACOS_DIR}/${APP_NAME}-arm64"
swiftc -O -target "${X86_TARGET}" -module-cache-path /tmp/swift-module-cache "${FRAMEWORKS[@]}" "${SOURCES[@]}" -o "${MACOS_DIR}/${APP_NAME}-x86_64"

lipo -create -output "${MACOS_DIR}/${APP_NAME}" "${MACOS_DIR}/${APP_NAME}-arm64" "${MACOS_DIR}/${APP_NAME}-x86_64"
rm -f "${MACOS_DIR}/${APP_NAME}-arm64" "${MACOS_DIR}/${APP_NAME}-x86_64"

codesign --force --deep --sign - "${APP_BUNDLE}" >/dev/null 2>&1 || true

# Ad-hoc sign so macOS Gatekeeper does not report the bundle as "damaged"
codesign --force --deep --sign - "${APP_BUNDLE}"

echo "Built ${APP_BUNDLE} successfully ($(du -sh "${MACOS_DIR}/${APP_NAME}" | cut -f1))."
