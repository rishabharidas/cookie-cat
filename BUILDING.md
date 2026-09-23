# Building & Compiling Desktop Cat Widget for Multi-OS

This guide covers how to build and compile the Desktop Cat Widget for **macOS**, **Windows**, and **Linux** from a single Rust codebase.

---

## 1. Prerequisites

Ensure you have Rust installed via [rustup](https://rustup.rs/):
```bash
rustc --version
cargo --version
```

---

## 2. macOS Build Instructions

### Native Development Run
To test and run locally on macOS:
```bash
cargo run
```

### Release Build
```bash
cargo build --release
```
The optimized executable will be located at:
`target/release/cookie`

### Optional: Packaging as a native macOS `.app` bundle
You can turn the binary into a native clickable macOS `.app` bundle using `cargo-bundle`:
```bash
# 1. Install cargo-bundle
cargo install cargo-bundle

# 2. Bundle the application
cargo bundle --release
```
This generates `target/release/bundle/osx/Desktop Cat.app` which you can drag to your `/Applications` folder!

---

## 3. Windows Build Instructions

### A. Compiling Natively on Windows
If building directly on a Windows PC:
```powershell
# In PowerShell / Command Prompt:
cargo run
cargo build --release
```
The binary will be located at:
`target\release\cookie.exe`

> **Note on Windows Transparency**:
> The code automatically configures `composite_alpha_mode: CompositeAlphaMode::Auto` for Windows, which utilizes the Windows Desktop Window Manager (DWM) for seamless borderless alpha transparency.

### B. Cross-Compiling for Windows from macOS or Linux
You can easily cross-compile a Windows `.exe` directly from macOS or Linux using `cargo-xwin`:
```bash
# 1. Install cargo-xwin and the Windows MSVC target
cargo install cargo-xwin
rustup target add x86_64-pc-windows-msvc

# 2. Compile for Windows 64-bit
cargo xwin build --release --target x86_64-pc-windows-msvc
```
Your Windows `.exe` will be ready at:
`target/x86_64-pc-windows-msvc/release/cookie.exe`

---

## 4. Linux Build Instructions

### A. Install Linux Dependencies
Bevy relies on standard multimedia and system libraries. On Ubuntu / Debian:
```bash
sudo apt update
sudo apt install -y \
    pkg-config \
    libasound2-dev \
    libudev-dev \
    libwayland-dev \
    libx11-dev \
    libxkbcommon-dev
```
On Arch Linux / Manjaro:
```bash
sudo pacman -S --needed pkgconf alsa-lib systemd wayland libx11 libxkbcommon
```
On Fedora:
```bash
sudo dnf install -y pkgconf-pkg-config alsa-lib-devel systemd-devel wayland-devel libX11-devel libxkbcommon-devel
```

### B. Build and Run
```bash
# Development
cargo run

# Optimized Release
cargo build --release
```
The binary will be located at:
`target/release/cookie`

> **Note on Linux Transparency**:
> On Linux, the app automatically uses `CompositeAlphaMode::PreMultiplied` with X11 / Wayland compositors (such as Picom, Mutter, KWin, or Sway) to provide native desktop transparency.

---

## 5. Automated CI/CD (GitHub Actions)

This repository includes a ready-to-use GitHub Actions workflow at [`.github/workflows/release.yml`](.github/workflows/release.yml).
Whenever you push a tag (e.g., `v0.1.0`), GitHub Actions automatically compiles:
- `desktop-cat-macos-arm64.tar.gz` (Apple Silicon)
- `desktop-cat-macos-x64.tar.gz` (Intel Mac)
- `desktop-cat-windows-x64.zip` (Windows 64-bit `.exe`)
- `desktop-cat-linux-x64.tar.gz` (Linux 64-bit)

and publishes them directly to your GitHub Releases page!

---

## 6. Controls & Shortcuts

| Action | Control | Description |
|---|---|---|
| **Drag & Reposition** | **Left Click + Drag** | Native OS window drag anywhere on the screen |
| **Pet the Cat** | **Left Click** | Bouncing happy reaction with floating `❤️` hearts |
| **Toggle 3D Model** | **`M`** | Switch between Blender 3D Model (`cat.glb`) and Stylized Procedural |
| **Cycle Coat** | **Right Click** or **`C`** | Cycle: Biscuit -> White -> Grey |
| **Cycle Size** | **`S`** | Cycle: Small (80%) -> Normal (100%) -> Large (130%) -> Extra Large (160%) |
| **Sleep / Nap** | **`Space`** | Toggles nap mode with floating `z Z Z` particles |
| **Help** | **`H`** | Print keyboard shortcuts and controls in terminal |
| **Quit** | **`Escape`** or **`Q`** | Close widget |
