# Cookie Roadmap

Planned features, macOS polish, and cross-platform compatibility milestones for **Cookie**.

---

## 1. Music & Audio Experience
- [ ] **Audio-Reactive Beat Sync:** Sample real-time system audio amplitude/BPM so Cookie's head-bob and ear-bop match the music tempo.
- [ ] **Now Playing Info:** Show current track & artist in the right-click menu or hover tooltip (Apple Music / Spotify).
- [ ] **Headphone Styles:** Selectable headphone themes (Classic Dark, Silver Max, Pink Cat-Ear Headphones).

## 2. Cat Skins, Accessories & `.kurocat` Support
- [ ] **`.kurocat` Skin Importer:** Load community `.kurocat` v1 JSON skins via drag-and-drop or `Import Skin...`.
- [ ] **2.5D Accessories & Hats:** Toggleable glasses, collar/scarf, and seasonal hats projected in 3D head space.
- [ ] **More Expressions & Reactions:** Optional purring audio, pre-sleep yawn animation, and playful paw-swat when hovering near the face in hunting mode.

## 3. macOS Polish & Distribution
- [ ] **Universal Binary (`arm64` + `x86_64`):** Build both Apple Silicon and Intel slices via `lipo` in `build.sh`.
- [ ] **Launch at Login:** Add a `Launch at Login` menu toggle using `SMAppService.mainApp`.
- [ ] **Homebrew Cask & Update Check:** Distribute via Homebrew tap and add a lightweight GitHub Releases update check.
- [ ] **OBS / Streaming Mode:** Optional chroma-key / green-screen background mode for streamers.

## 4. Cross-Platform Compatibility (Windows & Linux)
- [ ] **Windows (`windows/`):** Native C++17 + Win32 + GDI+/Direct2D (`WS_EX_LAYERED` transparent overlay with per-pixel hit-testing), WASAPI/GSMTC music detection, and `Shell_NotifyIcon` tray.
- [ ] **Linux (`linux/` — Arch, Debian/Ubuntu, Wayland & X11):** Native C++17 + GTK3 + Cairo (RGBA transparent window + `cairo_region` input mask), MPRIS D-Bus / PipeWire music detection, and `StatusNotifier` / `AppIndicator3` tray, packaged as `.deb` and Arch `PKGBUILD`.
- [ ] **GitHub Actions CI:** Automated matrix builds for macOS `.zip`, Windows `.exe`, Debian `.deb`, and Arch `.tar.zst` on tagged releases.

## 5. Future / Advanced Animations (Post Multi-Platform)
- [ ] **2.5D Walking Gait:** Add a procedural side/3-quarter walking animation (leg stride, body bob, and tail balance) in `CatRenderer` + `CatPhysics` while smoothly moving the overlay window across the screen or between monitors.
- [ ] **Workspace / Space-Switch Walking:** Subscribe to workspace change events (`NSWorkspace.activeSpaceDidChangeNotification` on macOS, Virtual Desktop / EWMH / Wayland events on Windows & Linux) so Cookie walks into the newly active desktop/workspace instead of appearing statically.

