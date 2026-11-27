import AppKit
import CoreGraphics

/// Built-in cat skins, with Black Cat as default.
enum CatSkinId: String, CaseIterable {
    case black = "black"
    case britishBlue = "britishBlue"
    case shorthair = "shorthair"
    case orangeWhite = "orangeWhite"
    case white = "white"

    var displayName: String {
        switch self {
        case .black: return "Black Cat"
        case .britishBlue: return "British Blue-Grey"
        case .shorthair: return "Silver Shorthair"
        case .orangeWhite: return "Orange & White"
        case .white: return "White Cat"
        }
    }
}

/// Customizable eye colors (mix-and-match with any cat skin, just like KuroCat).
enum EyeColorTheme: String, CaseIterable {
    case chartreuse = "chartreuse"
    case amber = "amber"
    case sapphire = "sapphire"
    case emerald = "emerald"
    case rose = "rose"
    case violet = "violet"

    var displayName: String {
        switch self {
        case .chartreuse: return "Chartreuse Lime (KuroCat)"
        case .amber: return "Golden Amber"
        case .sapphire: return "Sapphire Blue"
        case .emerald: return "Emerald Green"
        case .rose: return "Ruby Rose"
        case .violet: return "Amethyst Violet"
        }
    }

    var primaryNSColor: NSColor {
        switch self {
        case .chartreuse: return NSColor(calibratedRed: 0.72, green: 0.95, blue: 0.24, alpha: 1.0)
        case .amber: return NSColor(calibratedRed: 0.96, green: 0.76, blue: 0.22, alpha: 1.0)
        case .sapphire: return NSColor(calibratedRed: 0.42, green: 0.74, blue: 0.96, alpha: 1.0)
        case .emerald: return NSColor(calibratedRed: 0.26, green: 0.84, blue: 0.56, alpha: 1.0)
        case .rose: return NSColor(calibratedRed: 0.96, green: 0.48, blue: 0.64, alpha: 1.0)
        case .violet: return NSColor(calibratedRed: 0.72, green: 0.54, blue: 0.96, alpha: 1.0)
        }
    }
}

struct CatPalette {
    let outline: CGColor
    let bodyFill: CGColor
    let bodyShade: CGColor
    let headFill: CGColor
    let chestFill: CGColor
    let earOuter: CGColor
    let earInner: CGColor
    let muzzleFill: CGColor?
    let nose: CGColor
    let mouth: CGColor
    let eyeSclera: CGColor
    let eyeIris: CGColor
    let eyePupil: CGColor
    let blush: CGColor
    let whisker: CGColor
    let pawPad: CGColor
    let headphoneBand: CGColor
    let headphoneCup: CGColor
    let headphoneCushion: CGColor
    let headphoneAccent: CGColor
}

struct CatShape {
    let bodyWidth: CGFloat
    let headScale: CGFloat
    let earSize: CGFloat
    let tailLength: CGFloat
    let tailThickness: CGFloat
    let cheekFullness: CGFloat
}

struct SkinSpec {
    let id: CatSkinId
    let palette: CatPalette
    let shape: CatShape

    static func builtIn(_ id: CatSkinId) -> SkinSpec {
        let defaultShape = CatShape(
            bodyWidth: 1.0,
            headScale: 1.0,
            earSize: 1.0,
            tailLength: 1.0,
            tailThickness: 1.0,
            cheekFullness: 1.0
        )

        switch id {
        case .black:
            return SkinSpec(
                id: .black,
                palette: CatPalette(
                    outline: rgba(0.36, 0.42, 0.46, 0.85), // Subtle cool-slate rim so Black Cat pops on black backgrounds
                    bodyFill: rgba(0.08, 0.08, 0.10, 1.0),
                    bodyShade: rgba(0.06, 0.06, 0.08, 1.0),
                    headFill: rgba(0.08, 0.08, 0.10, 1.0),
                    chestFill: rgba(0.065, 0.065, 0.085, 1.0),
                    earOuter: rgba(0.08, 0.08, 0.10, 1.0),
                    earInner: rgba(0.38, 0.28, 0.32, 1.0),
                    muzzleFill: nil,
                    nose: rgba(0.85, 0.50, 0.56, 1.0),
                    mouth: rgba(0.68, 0.68, 0.74, 1.0),
                    eyeSclera: rgba(0.98, 0.97, 0.94, 1.0),
                    eyeIris: rgba(0.72, 0.95, 0.24, 1.0),
                    eyePupil: rgba(0.05, 0.05, 0.07, 1.0),
                    blush: rgba(0.90, 0.45, 0.52, 0.36),
                    whisker: rgba(0.62, 0.64, 0.70, 0.78),
                    pawPad: rgba(0.84, 0.54, 0.60, 1.0),
                    headphoneBand: rgba(0.28, 0.31, 0.38, 1.0),
                    headphoneCup: rgba(0.20, 0.22, 0.28, 1.0),
                    headphoneCushion: rgba(0.36, 0.39, 0.46, 1.0),
                    headphoneAccent: rgba(0.40, 0.82, 0.98, 1.0)
                ),
                shape: defaultShape
            )

        case .britishBlue:
            // Adapted from kurocat black-grey.kurocat ("英短蓝猫" / British Shorthair Blue)
            return SkinSpec(
                id: .britishBlue,
                palette: CatPalette(
                    outline: rgba(0.24, 0.27, 0.31, 0.85),
                    bodyFill: hex(0x747D88),
                    bodyShade: hex(0x5E6670),
                    headFill: hex(0x747D88),
                    chestFill: hex(0x87909B),
                    earOuter: hex(0x666E78),
                    earInner: hex(0xA3989D),
                    muzzleFill: nil,
                    nose: hex(0x4D5259),
                    mouth: hex(0x4B5056),
                    eyeSclera: rgba(0.98, 0.98, 0.99, 1.0),
                    eyeIris: hex(0xE5A63B),
                    eyePupil: rgba(0.06, 0.06, 0.08, 1.0),
                    blush: rgba(0.88, 0.56, 0.62, 0.35),
                    whisker: hex(0xDCE2E8, alpha: 0.66),
                    pawPad: hex(0x5C5458),
                    headphoneBand: rgba(0.24, 0.26, 0.31, 1.0),
                    headphoneCup: rgba(0.16, 0.18, 0.22, 1.0),
                    headphoneCushion: rgba(0.32, 0.35, 0.40, 1.0),
                    headphoneAccent: hex(0xE5A63B)
                ),
                shape: CatShape(
                    bodyWidth: 1.15,
                    headScale: 1.0,
                    earSize: 0.90,
                    tailLength: 1.0,
                    tailThickness: 1.22,
                    cheekFullness: 1.14
                )
            )

        case .shorthair:
            return SkinSpec(
                id: .shorthair,
                palette: CatPalette(
                    outline: rgba(0.24, 0.26, 0.28, 0.85),
                    bodyFill: rgba(0.72, 0.74, 0.76, 1.0),
                    bodyShade: rgba(0.55, 0.57, 0.60, 1.0),
                    headFill: rgba(0.74, 0.76, 0.78, 1.0),
                    chestFill: rgba(0.96, 0.96, 0.97, 1.0),
                    earOuter: rgba(0.64, 0.66, 0.69, 1.0),
                    earInner: rgba(0.86, 0.66, 0.70, 1.0),
                    muzzleFill: rgba(0.96, 0.96, 0.97, 0.92),
                    nose: rgba(0.88, 0.55, 0.62, 1.0),
                    mouth: rgba(0.26, 0.28, 0.30, 1.0),
                    eyeSclera: rgba(1.0, 1.0, 1.0, 1.0),
                    eyeIris: rgba(0.32, 0.78, 0.52, 1.0),
                    eyePupil: rgba(0.06, 0.06, 0.08, 1.0),
                    blush: rgba(0.95, 0.52, 0.60, 0.40),
                    whisker: rgba(0.38, 0.40, 0.44, 0.72),
                    pawPad: rgba(0.88, 0.64, 0.70, 1.0),
                    headphoneBand: rgba(0.24, 0.26, 0.32, 1.0),
                    headphoneCup: rgba(0.20, 0.22, 0.27, 1.0),
                    headphoneCushion: rgba(0.36, 0.38, 0.44, 1.0),
                    headphoneAccent: rgba(0.40, 0.82, 0.98, 1.0)
                ),
                shape: defaultShape
            )

        case .orangeWhite:
            return SkinSpec(
                id: .orangeWhite,
                palette: CatPalette(
                    outline: rgba(0.34, 0.22, 0.14, 0.75),
                    bodyFill: rgba(0.93, 0.62, 0.28, 1.0),
                    bodyShade: rgba(0.80, 0.48, 0.18, 1.0),
                    headFill: rgba(0.93, 0.62, 0.28, 1.0),
                    chestFill: rgba(0.98, 0.97, 0.95, 1.0),
                    earOuter: rgba(0.91, 0.57, 0.23, 1.0),
                    earInner: rgba(0.95, 0.70, 0.74, 1.0),
                    muzzleFill: rgba(0.98, 0.97, 0.95, 0.94),
                    nose: rgba(0.92, 0.55, 0.60, 1.0),
                    mouth: rgba(0.28, 0.18, 0.12, 1.0),
                    eyeSclera: rgba(1.0, 1.0, 1.0, 1.0),
                    eyeIris: rgba(0.34, 0.78, 0.42, 1.0),
                    eyePupil: rgba(0.08, 0.07, 0.06, 1.0),
                    blush: rgba(0.98, 0.50, 0.58, 0.44),
                    whisker: rgba(0.55, 0.48, 0.40, 0.72),
                    pawPad: rgba(0.96, 0.68, 0.74, 1.0),
                    headphoneBand: rgba(0.24, 0.26, 0.32, 1.0),
                    headphoneCup: rgba(0.92, 0.44, 0.22, 1.0),
                    headphoneCushion: rgba(0.26, 0.26, 0.30, 1.0),
                    headphoneAccent: rgba(1.0, 0.86, 0.45, 1.0)
                ),
                shape: defaultShape
            )

        case .white:
            return SkinSpec(
                id: .white,
                palette: CatPalette(
                    outline: rgba(0.32, 0.30, 0.36, 0.75),
                    bodyFill: rgba(0.97, 0.97, 0.98, 1.0),
                    bodyShade: rgba(0.90, 0.90, 0.93, 1.0),
                    headFill: rgba(0.97, 0.97, 0.98, 1.0),
                    chestFill: rgba(0.99, 0.99, 1.0, 1.0),
                    earOuter: rgba(0.95, 0.95, 0.97, 1.0),
                    earInner: rgba(0.95, 0.72, 0.78, 1.0),
                    muzzleFill: nil,
                    nose: rgba(0.94, 0.56, 0.64, 1.0),
                    mouth: rgba(0.30, 0.28, 0.34, 1.0),
                    eyeSclera: rgba(1.0, 1.0, 1.0, 1.0),
                    eyeIris: rgba(0.24, 0.76, 0.68, 1.0),
                    eyePupil: rgba(0.08, 0.08, 0.10, 1.0),
                    blush: rgba(0.98, 0.56, 0.65, 0.45),
                    whisker: rgba(0.55, 0.55, 0.60, 0.75),
                    pawPad: rgba(0.96, 0.68, 0.75, 1.0),
                    headphoneBand: rgba(0.24, 0.26, 0.32, 1.0),
                    headphoneCup: rgba(0.94, 0.52, 0.64, 1.0),
                    headphoneCushion: rgba(0.28, 0.28, 0.32, 1.0),
                    headphoneAccent: rgba(1.0, 0.82, 0.88, 1.0)
                ),
                shape: defaultShape
            )
        }
    }
}

private func rgba(_ r: CGFloat, _ g: CGFloat, _ b: CGFloat, _ a: CGFloat) -> CGColor {
    CGColor(red: r, green: g, blue: b, alpha: a)
}

private func hex(_ value: Int, alpha: CGFloat = 1.0) -> CGColor {
    let r = CGFloat((value >> 16) & 0xFF) / 255.0
    let g = CGFloat((value >> 8) & 0xFF) / 255.0
    let b = CGFloat(value & 0xFF) / 255.0
    return CGColor(red: r, green: g, blue: b, alpha: alpha)
}
