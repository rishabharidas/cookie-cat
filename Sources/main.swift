import AppKit
import CoreGraphics
import Foundation

if CommandLine.arguments.contains("--render-preview") {
    let outDir = "/tmp/cookie_preview"
    try? FileManager.default.createDirectory(atPath: outDir, withIntermediateDirectories: true)

    let renderer = CatRenderer()

    let scenarios: [(String, CatSkinId, (CatPhysics) -> Void)] = [
        ("idle", .black, { p in
            p.headYaw = 0.0; p.headPitch = 0.0
        }),
        ("orange_white", .orangeWhite, { p in
            p.headYaw = 0.20; p.headPitch = 0.05; p.eyeLookX = 0.2
        }),
        ("shorthair", .shorthair, { p in
            p.headYaw = -0.20; p.headPitch = 0.05; p.eyeLookX = -0.2
        }),
        ("look_left", .black, { p in
            p.headYaw = -0.65; p.headPitch = 0.12; p.eyeLookX = -0.6; p.musicLevel = 1.0
            p.particles = [
                FloatingParticle(kind: .noteDouble, x: 215, y: 78, vx: 0, vy: 0, age: 0.4, lifetime: 1.6, scale: 1.1, phase: 0, color: CGColor(red: 0.4, green: 0.85, blue: 1.0, alpha: 1.0)),
                FloatingParticle(kind: .noteSingle, x: 205, y: 60, vx: 0, vy: 0, age: 0.6, lifetime: 1.6, scale: 1.0, phase: 0, color: CGColor(red: 1.0, green: 0.52, blue: 0.76, alpha: 1.0)),
            ]
        }),
    ]

    for (name, skinId, configure) in scenarios {
        let skin = SkinSpec.builtIn(skinId)
        let physics = CatPhysics()
        configure(physics)

        let scale: CGFloat = 2.0
        let w = Int(CatRenderer.logicalSize.width * scale)
        let h = Int(CatRenderer.logicalSize.height * scale)

        let rep = NSBitmapImageRep(
            bitmapDataPlanes: nil,
            pixelsWide: w,
            pixelsHigh: h,
            bitsPerSample: 8,
            samplesPerPixel: 4,
            hasAlpha: true,
            isPlanar: false,
            colorSpaceName: .deviceRGB,
            bytesPerRow: 0,
            bitsPerPixel: 0
        )!
        let nsCtx = NSGraphicsContext(bitmapImageRep: rep)!
        NSGraphicsContext.saveGraphicsState()
        NSGraphicsContext.current = nsCtx
        let ctx = nsCtx.cgContext
        // Flip for CatRenderer coordinates
        ctx.translateBy(x: 0, y: CGFloat(h))
        ctx.scaleBy(x: scale, y: -scale)

        // Subtle dark slate background
        ctx.setFillColor(CGColor(red: 0.14, green: 0.15, blue: 0.18, alpha: 1.0))
        ctx.fill(CGRect(origin: .zero, size: CatRenderer.logicalSize))

        renderer.draw(in: ctx, skin: skin, physics: physics, eyeTheme: skinId == .orangeWhite ? .amber : .chartreuse, showBorder: true)
        NSGraphicsContext.restoreGraphicsState()

        if let png = rep.representation(using: .png, properties: [:]) {
            let path = "\(outDir)/\(name).png"
            try? png.write(to: URL(fileURLWithPath: path))
            print("Saved \(path)")
        }
    }
    exit(0)
}

let app = NSApplication.shared
let delegate = AppDelegate()
app.delegate = delegate
app.run()
