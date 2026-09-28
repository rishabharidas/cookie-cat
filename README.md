# 🐱 Cookie — 3D Desktop Cat Companion

[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Bevy](https://img.shields.io/badge/bevy-0.19-blue.svg?style=flat-square)](https://bevyengine.org/)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-brightgreen.svg?style=flat-square)]()
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg?style=flat-square)]()

**Cookie** is an adorable, lightweight, transparent, and always-on-top 3D procedural desktop pet companion built with [Rust](https://www.rust-lang.org/) and the [Bevy Engine](https://bevyengine.org/). 

Cookie roams along your screen dock, follows your mouse with curious gaze tracking, curls up for naps, reacts to petting, and stays out of your way as a silent background accessory.

---

## ✨ Features

* **🐾 Organic 3D Feline Locomotion & Physics**
  * Procedural 3D character with PBR materials and customizable coat colors (*Biscuit*, *White*, *Grey*).
  * 4-beat diagonal quadruped gait synchronized to movement speed.
  * Squash-and-stretch body kinematics for natural breathing and landing impacts.
  * 3-segment spring-damper physics tail with continuous sinusoidal wave propagation.

* **🧠 Organic Behavioral AI**
  * **Weighted state machine**: Cookie spends most of the day relaxing, looking around, grooming, stretching, and napping rather than constantly running.
  * **Curiosity & Gaze Tracking**: 3D head and neck articulate to look towards the cursor when attentive.
  * **Petting Reactions**: Click and interact with Cookie to trigger happy purring animations and floating heart particles.
  * **Deep Sleep Loaf**: Falls into a peaceful loaf pose when you're idle, with closed eyes and slow breathing.

* **🖥️ Native Multi-Display & Desktop Integration**
  * **Dynamic work-area & dock detection**: Automatically detects active monitor boundaries, Dock/Taskbar height, and DPI scale factors across arbitrary multi-monitor setups.
  * **True Drag Authority**: Pick up Cookie and place him anywhere on any screen without snaps or resistance.
  * **Background Accessory**: On macOS, Cookie operates as a background accessory with no Dock icon and no top menu bar takeover.

* **🎛️ Settings Dashboard & Native Tray**
  * **Right-Click System Tray Icon**: Access quick controls (Show/Hide, Pause/Resume, Open Dashboard, Exit) via status bar icon.
  * **Live Settings Dashboard**: Customize size (80%–150%), movement speed, idle frequencies, coat colors, and behavior toggles with automatic disk persistence.

---

## 🚀 Getting Started

### Prerequisites

* [Rust](https://www.rust-lang.org/tools/install) (1.85+ recommended, 2024 edition)
* Operating System: **macOS**, **Windows**, or **Linux**

### Installation & Run

1. **Clone the repository:**
   ```bash
   git clone https://github.com/rishabharidas/cookie.git
   cd cookie
   ```

2. **Run in development mode:**
   ```bash
   cargo run
   ```

3. **Build optimized release binary:**
   ```bash
   cargo run --release
   ```

---

## 🎮 Controls & Interactions

| Action | How to Trigger |
| :--- | :--- |
| **Pet Cookie** | **Left-click** on Cookie's window |
| **Move Cookie** | **Click and drag** anywhere on Cookie to place him on any screen |
| **Open Tray Menu** | **Right-click** the cat icon in the top menu bar / system tray |
| **Open Dashboard** | Select **Open Dashboard** from the tray menu |
| **Pause / Resume** | Toggle via Tray Menu or Settings Dashboard |

---

## 🏗️ Architecture & Module Map

Cookie is designed with clean, modular ECS (Entity-Component-System) plugins:

```
src/
├── main.rs            # Application entry, 3D studio lighting, window configuration
├── cat_model.rs       # 3D procedural chibi feline hierarchy, meshes, and PBR coats
├── cat_physics.rs     # Locomotion cycle, eye blinking, look tracking, whisker/tail springs
├── cat_ai.rs          # Behavioral AI state machine, roaming, curiosity, nap routines
├── platform.rs        # Native OS display queries, multi-monitor bounds, dock avoidance
├── tray.rs            # System tray integration with right-click context menu
├── dashboard.rs       # Secondary window UI for user settings & live state monitoring
├── window_control.rs  # OS window positioning, gravity, and manual drag authority
└── config.rs          # Persistent user settings and global state management
```

---

## 🧪 Testing

Run the automated test suite covering display math, physics easing, serialization, and state machines:

```bash
cargo test
```

---

## 🤝 Contributing

Contributions, feature suggestions, and pull requests are welcome!

### Ideas to explore:
* 🧶 Interactive toys (yarn ball, laser pointer, treat snacks)
* 🎩 Customizable hats and seasonal accessories
* 🐾 Novel animation routines (playful pounce, ear scratching)
* 🔊 Optional sound effects / purring audio

### Guidelines:
1. Fork the repo and create your feature branch: `git checkout -b feat/my-cool-feature`.
2. Ensure all tests pass: `cargo test`.
3. Format your code: `cargo fmt`.
4. Submit a Pull Request with a clear summary and screenshots/GIFs.

---

## 📜 License

This project is licensed under either the [MIT License](LICENSE) or the [Apache 2.0 License](LICENSE-APACHE) at your option.
