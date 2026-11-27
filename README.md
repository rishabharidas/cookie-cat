# Cookie — 2.5D Desktop Cat Companion (macOS)

A lightweight, transparent, always-on-top 2.5D desktop cat companion built natively in **Swift + AppKit + CoreGraphics** (~260 KB binary).

## Acknowledgments

Big thanks to **[chanke17/KuroCat](https://github.com/chanke17/KuroCat)**!  
Cookie's 2.5D spherical dome head projection, unified C1-continuous cat silhouette, expressive eyes, dual-color coat patterns, and sitting pose are directly inspired by and adapted from **KuroCat** (MIT License).

## Features

- **2.5D Gaze Tracking & Hunting Eyes**: Spherical dome head projection follows your cursor across screens; fast cursor movement triggers boba hunting pupils.
- **Music Listening Mode**: Detects active system audio via CoreAudio and puts fitted wireless headphones on Cookie with floating `♪` / `♫` notes and a menu bar badge.
- **Natural Cat Interactions**:
  - **Pet**: Click Cookie for happy closed eyes and sparkles/hearts.
  - **Idle Behaviors**: Periodic paw-licking, ear-scratching, blinking, and sleepy `z` mode.
  - **Body-Only Drag & Physics**: Transparent areas pass clicks through; dragging Cookie from its body triggers pendulum sway and stretch physics.
- **Customization (Right-Click Menu Bar Icon or Cat)**:
  - **5 Skins**: Black Cat, British Blue-Grey, Silver Shorthair, Orange & White, White Cat.
  - **6 Eye Colors**: Chartreuse Lime, Golden Amber, Sapphire Blue, Emerald Green, Ruby Rose, Amethyst Violet.
  - **5 Sizes**: `0.6x` to `1.5x`, plus Rim Border toggle (for dark backgrounds), High Frame Rate (60fps), and Click-Through mode (hold `⌥ Option` to drag).

## Build & Run (macOS)

```bash
./build.sh
open Cookie.app
```

## License

MIT License. See [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md) for KuroCat MIT License attribution.
