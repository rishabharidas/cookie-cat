import AppKit
import CoreGraphics

/// Exact 2.5D KuroCat cat renderer adapted from chanke17/KuroCat (MIT License),
/// without the keyboard/mouse desk, and extended with wireless headphones & music notes.
final class CatRenderer {
    static let logicalSize = CGSize(width: 300, height: 260)

    private let seatAnchor = CGPoint(x: 150, y: 214)

    // KuroCat exact palette helpers
    private static let catBlack = NSColor(calibratedRed: 0.08, green: 0.08, blue: 0.10, alpha: 1.0)
    private static let catBlackPaw = NSColor(calibratedRed: 0.065, green: 0.065, blue: 0.085, alpha: 1.0)
    private static let catBlackCrease = NSColor(calibratedRed: 0.34, green: 0.48, blue: 0.40, alpha: 0.72) // signature mint-sage crease
    private static let whiskerDefault = NSColor(calibratedRed: 0.55, green: 0.56, blue: 0.60, alpha: 0.72)

    private static let siameseCream = NSColor(calibratedRed: 0.945, green: 0.905, blue: 0.835, alpha: 1.0)
    private static let siameseLatte = NSColor(calibratedRed: 0.885, green: 0.825, blue: 0.735, alpha: 1.0)
    private static let siamesePoint = NSColor(calibratedRed: 0.22, green: 0.16, blue: 0.135, alpha: 1.0)
    private static let siameseMocha = NSColor(calibratedRed: 0.45, green: 0.34, blue: 0.28, alpha: 1.0)
    private static let siameseInnerEar = NSColor(calibratedRed: 0.50, green: 0.36, blue: 0.34, alpha: 1.0)
    private static let siameseOutline = NSColor(calibratedRed: 0.55, green: 0.45, blue: 0.36, alpha: 0.35)
    private static let siameseCrease = NSColor(calibratedRed: 0.38, green: 0.28, blue: 0.24, alpha: 0.55)
    private static let siameseWhisker = NSColor(calibratedRed: 0.32, green: 0.26, blue: 0.22, alpha: 0.70)

    private func bodyColor(_ skin: SkinSpec) -> NSColor {
        switch skin.id {
        case .black: return Self.catBlack
        case .siamese: return Self.siameseCream
        case .shorthair: return NSColor(calibratedRed: 0.96, green: 0.955, blue: 0.94, alpha: 1.0) // White base for dual-color pattern
        case .white: return NSColor(calibratedRed: 0.97, green: 0.97, blue: 0.98, alpha: 1.0)
        case .orangeWhite: return NSColor(calibratedRed: 0.965, green: 0.955, blue: 0.935, alpha: 1.0) // White base for dual-color pattern
        }
    }

    private func earTailColor(_ skin: SkinSpec) -> NSColor {
        switch skin.id {
        case .black: return Self.catBlack
        case .siamese: return Self.siamesePoint
        case .shorthair: return NSColor(calibratedRed: 0.68, green: 0.70, blue: 0.73, alpha: 1.0)
        case .white: return NSColor(calibratedRed: 0.95, green: 0.95, blue: 0.97, alpha: 1.0)
        case .orangeWhite: return NSColor(calibratedRed: 0.88, green: 0.68, blue: 0.46, alpha: 1.0)
        }
    }

    private func pawColor(_ skin: SkinSpec) -> NSColor {
        switch skin.id {
        case .black: return Self.catBlackPaw
        case .siamese: return Self.siamesePoint
        case .shorthair: return NSColor(calibratedRed: 0.96, green: 0.96, blue: 0.97, alpha: 1.0)
        case .white: return NSColor(calibratedRed: 0.99, green: 0.99, blue: 1.0, alpha: 1.0)
        case .orangeWhite: return NSColor(calibratedRed: 0.98, green: 0.97, blue: 0.95, alpha: 1.0)
        }
    }

    private func creaseColor(_ skin: SkinSpec) -> NSColor {
        switch skin.id {
        case .black: return Self.catBlackCrease
        case .siamese: return Self.siameseCrease
        case .shorthair: return NSColor(calibratedRed: 0.36, green: 0.38, blue: 0.42, alpha: 0.65)
        case .white: return NSColor(calibratedRed: 0.62, green: 0.62, blue: 0.68, alpha: 0.65)
        case .orangeWhite: return NSColor(calibratedRed: 0.52, green: 0.34, blue: 0.18, alpha: 0.55)
        }
    }

    /// Projects a 3D point on KuroCat's head ellipsoid rotated by (yawRad, pitchRad).
    private func projectHead3D(
        x: CGFloat, y: CGFloat, z: CGFloat,
        yawRad: CGFloat, pitchRad: CGFloat
    ) -> (pt: CGPoint, z: CGFloat) {
        let cy = cos(yawRad), sy = sin(yawRad)
        let x1 = x * cy + z * sy
        let z1 = -x * sy + z * cy

        let cp = cos(pitchRad), sp = sin(pitchRad)
        let y2 = y * cp + z1 * sp
        let z2 = -y * sp + z1 * cp

        return (CGPoint(x: x1, y: y2), z2)
    }

    /// Returns true only when `localPoint` (in unscaled 300x260 coordinates) is on the actual cat head or body.
    static func hitTest(localPoint p: CGPoint, physics: CatPhysics) -> Bool {
        let headCenter = CGPoint(x: 150 + physics.headYaw * 7.5, y: 122 + physics.headBobY)
        let hdx = (p.x - headCenter.x) / 45.0
        let hdy = (p.y - headCenter.y) / 38.0
        if hdx * hdx + hdy * hdy <= 1.0 {
            return true
        }

        let bdx = (p.x - 150.0) / 48.0
        let bdy = (p.y - 178.0) / 52.0
        if bdx * bdx + bdy * bdy <= 1.0 {
            return true
        }
        return false
    }

    func draw(
        in ctx: CGContext,
        skin: SkinSpec,
        physics: CatPhysics,
        eyeTheme: EyeColorTheme = .chartreuse,
        showBorder: Bool = true
    ) {
        ctx.saveGState()

        let seatX = seatAnchor.x
        let seatY = seatAnchor.y
        let breath = sin(physics.breathPhase) * 1.6
        let bodyTilt = physics.headYaw * 0.06
            + sin(physics.musicBeatPhase * 0.5) * 0.025 * physics.musicLevel
            + physics.dragSwayAngle
        let siameseLift: CGFloat = (skin.id == .siamese) ? -3.0 : 0.0

        let headCenter = CGPoint(
            x: seatX + bodyTilt * 36.0 + physics.headYaw * 7.5 - physics.sleepLevel * 16.0,
            y: 122.0 + siameseLift + physics.headBobY + physics.headPitch * 4.8 + breath * 1.1
        )

        // 1. Tail (behind body when awake; wraps around front when curled into sleep donut!)
        if physics.sleepLevel <= 0.45 {
            drawTail(in: ctx, seatX: seatX, seatY: seatY, physics: physics, skin: skin, showBorder: showBorder)
        }

        // 2. 3D-Projected Ears (behind unified silhouette)
        drawEarsLayer(in: ctx, center: headCenter, physics: physics, skin: skin, showBorder: showBorder)

        // 3. Unified C1-Continuous Head + Neck + Pear Torso Silhouette
        drawUnifiedSilhouette(
            in: ctx,
            seatX: seatX,
            seatY: seatY,
            headCenter: headCenter,
            breath: breath,
            physics: physics,
            bodyTilt: bodyTilt,
            skin: skin,
            showBorder: showBorder
        )

        // 4. 2.5D Turning Head Features: Luminous Eyes & Whiskers
        drawHeadFeatures(in: ctx, center: headCenter, physics: physics, skin: skin, eyeTheme: eyeTheme)

        // 5. Front Paws (Idle Sitting, locked to bodyTilt & squashY physics; tucked away in donut sleep)
        if physics.sleepLevel < 0.70 {
            ctx.saveGState()
            ctx.setAlpha(max(0.0, 1.0 - physics.sleepLevel * 1.5))
            drawFrontPaws(in: ctx, seatX: seatX, seatY: seatY, bodyTilt: bodyTilt, physics: physics, skin: skin, showBorder: showBorder)
            ctx.restoreGState()
        }

        // Wrapped donut tail drawn in front of lower body when curled up sleeping
        if physics.sleepLevel > 0.45 {
            drawTail(in: ctx, seatX: seatX, seatY: seatY, physics: physics, skin: skin, showBorder: showBorder)
        }

        // 6. Fitted Wireless Headphones when listening to music
        if physics.musicLevel > 0.02 {
            drawWirelessHeadphones(in: ctx, center: headCenter, physics: physics, skin: skin, eyeTheme: eyeTheme)
        }

        // 7. Idle limb actions (paw-licking / ear-scratching)
        drawIdleLimbs(in: ctx, seatX: seatX, seatY: seatY, headCenter: headCenter, physics: physics, skin: skin)

        // 8. Floating particles (music notes, sparkles/hearts, sleep z)
        drawParticles(in: ctx, physics: physics)

        ctx.restoreGState()
    }

    // MARK: - Tail (KuroCat low sweep out to the right with upward curl)

    private func drawTail(in ctx: CGContext, seatX: CGFloat, seatY: CGFloat, physics: CatPhysics, skin: SkinSpec, showBorder: Bool) {
        let p = physics.tailPhase
        let hunt = physics.huntingLevel
        let sleep = physics.sleepLevel
        let wag1 = sin(p) * (8.0 + hunt * 5.5) * (1.0 - sleep * 0.85)
        let wag2 = cos(p * 1.3) * (6.0 + hunt * 4.5) * (1.0 - sleep * 0.85)
        let huntDrop = hunt * 14.0

        // Awake sweeping tail vs. curled-up donut tail wrapped snugly around the front/chin
        let awakeRoot = CGPoint(x: seatX + 34, y: seatY - 6)
        let awakeC1   = CGPoint(x: seatX + 64, y: seatY + 2 + wag1 * 0.35 + huntDrop * 0.4)
        let awakeC2   = CGPoint(x: seatX + 66 + wag2 * 0.4, y: seatY - 26 + wag1 * 0.7 + huntDrop)
        let awakeTip  = CGPoint(x: seatX + 91 + wag2 * 0.5, y: seatY - 30 + wag1 * 0.85 + huntDrop)

        let sleepRoot = CGPoint(x: seatX + 52, y: seatY - 10)
        let sleepC1   = CGPoint(x: seatX + 66, y: seatY + 18)
        let sleepC2   = CGPoint(x: seatX + 14, y: seatY + 24)
        let sleepTip  = CGPoint(x: seatX - 28, y: seatY + 12)

        let root = CGPoint(x: lerp(awakeRoot.x, sleepRoot.x, sleep), y: lerp(awakeRoot.y, sleepRoot.y, sleep))
        let c1   = CGPoint(x: lerp(awakeC1.x, sleepC1.x, sleep),     y: lerp(awakeC1.y, sleepC1.y, sleep))
        let c2   = CGPoint(x: lerp(awakeC2.x, sleepC2.x, sleep),     y: lerp(awakeC2.y, sleepC2.y, sleep))
        let tip  = CGPoint(x: lerp(awakeTip.x, sleepTip.x, sleep),   y: lerp(awakeTip.y, sleepTip.y, sleep))

        let width: CGFloat = (skin.id == .siamese ? 9.5 : 11.0)
        ctx.saveGState()
        ctx.setLineCap(.round)
        ctx.setLineJoin(.round)

        if showBorder {
            ctx.setStrokeColor(skin.palette.outline)
            ctx.setLineWidth(width + 3.2)
            ctx.move(to: root)
            ctx.addCurve(to: tip, control1: c1, control2: c2)
            ctx.strokePath()
        }

        ctx.setStrokeColor(earTailColor(skin).cgColor)
        ctx.setLineWidth(width)
        ctx.move(to: root)
        ctx.addCurve(to: tip, control1: c1, control2: c2)
        ctx.strokePath()
        ctx.restoreGState()
    }

    // MARK: - Unified C1-Continuous Cat Silhouette (Crown -> Cheeks -> Plush Neck -> Pear Hips)

    private func drawUnifiedSilhouette(
        in ctx: CGContext,
        seatX: CGFloat,
        seatY: CGFloat,
        headCenter: CGPoint,
        breath: CGFloat,
        physics: CatPhysics,
        bodyTilt: CGFloat,
        skin: SkinSpec,
        showBorder: Bool
    ) {
        let yaw = physics.headYaw
        let pitch = physics.headPitch
        let hTilt = physics.headTilt

        let cH = cos(hTilt), sH = sin(hTilt)
        func H(_ lx: CGFloat, _ ly: CGFloat) -> CGPoint {
            CGPoint(x: headCenter.x + lx * cH - ly * sH, y: headCenter.y + lx * sH + ly * cH)
        }

        let sy = physics.squashY
        let sx = 1.0 + (1.0 - sy) * 0.55
        let cB = cos(bodyTilt), sB = sin(bodyTilt)
        func B(_ lx: CGFloat, _ ly: CGFloat) -> CGPoint {
            let rx = lx * sx, ry = ly * sy
            return CGPoint(x: seatX + rx * cB - ry * sB, y: seatY + rx * sB + ry * cB)
        }

        let isSiamese = (skin.id == .siamese)
        let sleep = physics.sleepLevel
        let cheekShift = yaw * 3.0
        let crownDrop = pitch * 2.0
        let cheekHalfW: CGFloat = 42.5
        let crownCtrlW: CGFloat = isSiamese ? 25.0 : 26.0
        let crownBaseY: CGFloat = isSiamese ? -29.5 : -30.0
        let cheekY: CGFloat = isSiamese ? 6.5 : 5.0
        let neckPinchXRight: CGFloat = (isSiamese ? 6.5 : 2.5) - sleep * 14.0
        let neckPinchXLeft: CGFloat = (isSiamese ? 6.5 : 2.5) - sleep * 3.5
        let neckCtrlY: CGFloat = (isSiamese ? 27.5 : 24.0) - sleep * 22.0
        let hipHalfW: CGFloat = (isSiamese ? 44.5 : 49.0) + sleep * 5.0
        let lHipHalfW: CGFloat = (isSiamese ? 44.5 : 49.0) - sleep * 6.0
        let waistCtrlW: CGFloat = (isSiamese ? 41.0 : 51.5) + sleep * 8.0
        let lWaistCtrlW: CGFloat = (isSiamese ? 41.0 : 51.5) - sleep * 6.0
        let waistCtrlY: CGFloat = (isSiamese ? -50.0 : -48.0) - sleep * 4.0
        let lWaistCtrlY: CGFloat = (isSiamese ? -50.0 : -48.0) + sleep * 34.0
        let botOuterW: CGFloat = (isSiamese ? 42.0 : 47.0) + sleep * 8.0
        let botInnerW: CGFloat = (isSiamese ? 32.0 : 36.0) + sleep * 10.0
        let botHalfW: CGFloat = (isSiamese ? 22.0 : 24.0) + sleep * 12.0
        let leftCheekX: CGFloat = -cheekHalfW + cheekShift * 0.4
        let rightCheekX: CGFloat = cheekHalfW + cheekShift * 0.4

        let crown  = H(cheekShift * 0.2, crownBaseY + crownDrop)
        let rCheek = H(rightCheekX, cheekY)
        let rHip   = B(hipHalfW, -14)
        let rBot   = B(botHalfW, 12)
        let lBot   = B(-botHalfW, 12)
        let lHip   = B(-lHipHalfW, -14)
        let lCheek = H(leftCheekX, cheekY)

        let cheekCtrlTopY: CGFloat = isSiamese ? -13.0 : -14.0
        let path = CGMutablePath()
        path.move(to: crown)
        path.addCurve(to: rCheek, control1: H(crownCtrlW + cheekShift * 0.3, crownBaseY + crownDrop), control2: H(rightCheekX + 1.5, cheekCtrlTopY))
        path.addCurve(to: rHip,   control1: H(rightCheekX - neckPinchXRight, neckCtrlY), control2: B(waistCtrlW, waistCtrlY + breath * 0.5))
        path.addCurve(to: rBot,   control1: B(botOuterW, 5), control2: B(botInnerW, 12))
        path.addLine(to: lBot)
        path.addCurve(to: lHip,   control1: B(-botInnerW, 12), control2: B(-lWaistCtrlW, 5))
        path.addCurve(to: lCheek, control1: B(-lWaistCtrlW, lWaistCtrlY + breath * 0.5), control2: H(leftCheekX + neckPinchXLeft, neckCtrlY))
        path.addCurve(to: crown,  control1: H(leftCheekX - 1.5, cheekCtrlTopY), control2: H(-crownCtrlW + cheekShift * 0.3, crownBaseY + crownDrop))
        path.closeSubpath()

        ctx.saveGState()
        if !isSiamese {
            ctx.setFillColor(bodyColor(skin).cgColor)
            ctx.addPath(path)
            ctx.fillPath()
            if skin.id == .orangeWhite || skin.id == .shorthair {
                ctx.saveGState()
                ctx.addPath(path)
                ctx.clip()

                let yawRad = yaw * 0.50, pitchRad = pitch * 0.36
                let patchColor = earTailColor(skin).cgColor
                let stripeColor = skin.id == .shorthair
                    ? NSColor(calibratedRed: 0.42, green: 0.45, blue: 0.48, alpha: 0.68).cgColor
                    : NSColor(calibratedRed: 0.76, green: 0.52, blue: 0.28, alpha: 0.65).cgColor

                func headPt(_ lx: CGFloat, _ ly: CGFloat) -> CGPoint {
                    let depth = 29.6 * sqrt(max(0, 1 - pow(lx / 46.0, 2) - pow(ly / 40.0, 2)))
                    let p = projectHead3D(x: lx, y: ly, z: depth, yawRad: yawRad, pitchRad: pitchRad).pt
                    return CGPoint(x: headCenter.x + p.x * cH - p.y * sH, y: headCenter.y + p.x * sH + p.y * cH)
                }

                // 1. Left & Right Head/Cheek Color Masks (forming KuroCat's inverted-V white blaze between the eyes!)
                for sign in [-1.0, 1.0] as [CGFloat] {
                    let mask = CGMutablePath()
                    mask.move(to: headPt(sign * 3.5, -36))
                    mask.addQuadCurve(to: headPt(sign * 9.0, -4), control: headPt(sign * 6.0, -18))
                    mask.addQuadCurve(to: headPt(sign * 26.0, 6), control: headPt(sign * 16.0, 5))
                    mask.addLine(to: headPt(sign * 48.0, 9))
                    mask.addLine(to: headPt(sign * 48.0, -38))
                    mask.closeSubpath()
                    ctx.setFillColor(patchColor)
                    ctx.addPath(mask)
                    ctx.fillPath()

                    // Tabby stripes on forehead & outer cheek
                    ctx.setStrokeColor(stripeColor)
                    ctx.setLineWidth(2.1)
                    ctx.setLineCap(.round)
                    let f0 = headPt(sign * 13.0, -25.0), f1 = headPt(sign * 10.5, -17.0)
                    ctx.move(to: f0); ctx.addLine(to: f1)
                    let f2 = headPt(sign * 22.0, -24.0), f3 = headPt(sign * 18.5, -16.5)
                    ctx.move(to: f2); ctx.addLine(to: f3)
                    let c0 = headPt(sign * 31.0, 0.5), c1 = headPt(sign * 41.0, 3.0)
                    ctx.move(to: c0); ctx.addLine(to: c1)
                    ctx.strokePath()
                }

                // 2. Left & Right Torso Saddle/Hip Patches (leaving entire chest, belly & front legs white!)
                for sign in [-1.0, 1.0] as [CGFloat] {
                    let hipPatch = CGMutablePath()
                    hipPatch.move(to: B(sign * 52, -50))
                    hipPatch.addQuadCurve(to: B(sign * 27, -22), control: B(sign * 34, -38))
                    hipPatch.addQuadCurve(to: B(sign * 34, 8), control: B(sign * 22, -4))
                    hipPatch.addLine(to: B(sign * 52, 10))
                    hipPatch.closeSubpath()
                    ctx.setFillColor(patchColor)
                    ctx.addPath(hipPatch)
                    ctx.fillPath()

                    // Subtle flank tabby stripe
                    ctx.setStrokeColor(stripeColor)
                    ctx.setLineWidth(2.4)
                    ctx.move(to: B(sign * 44, -22))
                    ctx.addQuadCurve(to: B(sign * 33, -16), control: B(sign * 38, -21))
                    ctx.strokePath()
                }

                ctx.restoreGState()
            }
        } else {
            ctx.addPath(path)
            ctx.clip()
            let cs = CGColorSpaceCreateDeviceRGB()
            let bodyColors = [Self.siameseCream.cgColor, Self.siameseLatte.cgColor] as CFArray
            if let grad = CGGradient(colorsSpace: cs, colors: bodyColors, locations: [0.15, 1.0]) {
                ctx.drawLinearGradient(grad, start: CGPoint(x: seatX, y: headCenter.y - 34), end: CGPoint(x: seatX, y: seatY + 18), options: [.drawsBeforeStartLocation, .drawsAfterEndLocation])
            }
            let yawRad = yaw * 0.50, pitchRad = pitch * 0.36
            ctx.saveGState()
            ctx.translateBy(x: headCenter.x, y: headCenter.y)
            ctx.rotate(by: hTilt)
            let maskCenter = projectHead3D(x: 0, y: 3.0, z: 27.0, yawRad: yawRad, pitchRad: pitchRad).pt
            let maskScaleX = max(0.76, cos(yawRad * 0.7))
            let maskScaleY = max(0.82, cos(pitchRad * 0.7))
            ctx.translateBy(x: maskCenter.x, y: maskCenter.y)
            ctx.scaleBy(x: maskScaleX * 1.24, y: maskScaleY * 0.88)
            let maskColors = [
                Self.siamesePoint.cgColor,
                Self.siamesePoint.withAlphaComponent(0.92).cgColor,
                Self.siameseMocha.withAlphaComponent(0.62).cgColor,
                Self.siameseMocha.withAlphaComponent(0.0).cgColor
            ] as CFArray
            if let maskGrad = CGGradient(colorsSpace: cs, colors: maskColors, locations: [0.0, 0.42, 0.74, 1.0]) {
                ctx.drawRadialGradient(maskGrad, startCenter: .zero, startRadius: 0, endCenter: .zero, endRadius: 31.5, options: [.drawsBeforeStartLocation])
            }
            ctx.restoreGState()
        }
        ctx.restoreGState()

        if showBorder {
            ctx.saveGState()
            ctx.setStrokeColor(skin.palette.outline)
            ctx.setLineWidth(1.6)
            ctx.addPath(path)
            ctx.strokePath()
            ctx.restoreGState()
        } else if isSiamese || skin.id == .white {
            ctx.saveGState()
            ctx.setStrokeColor(Self.siameseOutline.cgColor)
            ctx.setLineWidth(1.0)
            ctx.addPath(path)
            ctx.strokePath()
            ctx.restoreGState()
        }

        // Tucked hind paws peeking softly along bottom edge (tucked under when curled in donut sleep)
        if sleep < 0.65 {
            ctx.saveGState()
            ctx.setAlpha(max(0.0, 1.0 - sleep * 1.6))
            ctx.translateBy(x: seatX, y: seatY)
            ctx.rotate(by: bodyTilt)
            ctx.scaleBy(x: sx, y: sy)
            ctx.setFillColor(pawColor(skin).cgColor)

            let leftHindX: CGFloat = isSiamese ? -31.0 : -35.0
            let hindW: CGFloat = isSiamese ? 15.5 : 17.0
            let rightHindX: CGFloat = isSiamese ? 15.5 : 18.0
            let leftHindY: CGFloat = isSiamese ? 3.5 : 3.0
            let leftHindH: CGFloat = isSiamese ? 12.0 : 11.0
            let rightHindY: CGFloat = (skin.id == .black || isSiamese) ? 4.5 : 3.0
            let rightHindH: CGFloat = (skin.id == .black || isSiamese) ? 12.5 : 11.0

            ctx.fillEllipse(in: CGRect(x: leftHindX, y: leftHindY, width: hindW, height: leftHindH))
            if physics.idleAction != .scratching || physics.idleActionBlend < 0.02 {
                ctx.fillEllipse(in: CGRect(x: rightHindX, y: rightHindY, width: hindW, height: rightHindH))
            }

            let rimColor = isSiamese ? Self.siameseCream.withAlphaComponent(0.55) : creaseColor(skin).withAlphaComponent(0.58)
            ctx.setStrokeColor(rimColor.cgColor)
            ctx.setLineWidth(1.3)
            ctx.setLineCap(.round)
            ctx.addArc(center: CGPoint(x: leftHindX + hindW * 0.5, y: leftHindY + leftHindH * 0.5), radius: 7.5, startAngle: 0.25, endAngle: .pi - 0.25, clockwise: false)
            ctx.strokePath()
            if physics.idleAction != .scratching || physics.idleActionBlend < 0.02 {
                ctx.addArc(center: CGPoint(x: rightHindX + hindW * 0.5, y: rightHindY + rightHindH * 0.5), radius: 7.5, startAngle: 0.25, endAngle: .pi - 0.25, clockwise: false)
                ctx.strokePath()
            }
            ctx.restoreGState()
        }
    }

    // MARK: - 2.5D Ears Layer

    private func drawEarsLayer(in ctx: CGContext, center: CGPoint, physics: CatPhysics, skin: SkinSpec, showBorder: Bool) {
        ctx.saveGState()
        ctx.translateBy(x: center.x, y: center.y)
        ctx.rotate(by: physics.headTilt)

        let yawRad = physics.headYaw * 0.50
        let pitchRad = physics.headPitch * 0.36
        let fold = physics.huntingLevel * 0.22 + physics.sleepLevel * 0.14
        let twitchL = physics.earTwitchLeft * 0.14
        let twitchR = -physics.earTwitchRight * 0.14

        drawEar3D(in: ctx, isLeft: true, yawRad: yawRad, pitchRad: pitchRad, fold: fold, twitch: twitchL, skin: skin, showBorder: showBorder)
        drawEar3D(in: ctx, isLeft: false, yawRad: yawRad, pitchRad: pitchRad, fold: fold, twitch: twitchR, skin: skin, showBorder: showBorder)

        ctx.restoreGState()
    }

    private func drawEar3D(
        in ctx: CGContext,
        isLeft: Bool,
        yawRad: CGFloat,
        pitchRad: CGFloat,
        fold: CGFloat,
        twitch: CGFloat,
        skin: SkinSpec,
        showBorder: Bool
    ) {
        let sign: CGFloat = isLeft ? -1.0 : 1.0
        let isSiamese = (skin.id == .siamese)
        let foldAngle = sign * ((isSiamese ? 0.14 : 0.06) + fold * 0.34) + twitch
        let earLen: CGFloat = isSiamese ? 29.5 : 32.0

        let tipDx = sin(foldAngle) * earLen
        let tipDy = -cos(foldAngle) * earLen

        let innerBase3D = projectHead3D(x: sign * (isSiamese ? 10.0 : 9.0), y: isSiamese ? -16.5 : -16.0, z: 0.0, yawRad: yawRad, pitchRad: pitchRad).pt
        let outerBase3D = projectHead3D(x: sign * (isSiamese ? 35.5 : 35.0), y: isSiamese ? -4.5 : -5.0, z: -2.0, yawRad: yawRad, pitchRad: pitchRad).pt
        let tip3D       = projectHead3D(x: sign * (isSiamese ? 25.0 : 24.5) + tipDx, y: (isSiamese ? -18.5 : -20.0) + tipDy, z: -3.0, yawRad: yawRad, pitchRad: pitchRad).pt

        let tipSpread: CGFloat = isSiamese ? 4.2 : 3.8
        let tipLeft  = CGPoint(x: tip3D.x - sign * tipSpread, y: tip3D.y + 2.8)
        let tipRight = CGPoint(x: tip3D.x + sign * tipSpread, y: tip3D.y + 3.2)

        let ear = CGMutablePath()
        ear.move(to: innerBase3D)
        ear.addQuadCurve(to: tipLeft, control: CGPoint(x: (innerBase3D.x + tipLeft.x) * 0.5 - sign * 1.5, y: (innerBase3D.y + tipLeft.y) * 0.5 - 2.0))
        ear.addQuadCurve(to: tipRight, control: CGPoint(x: tip3D.x, y: tip3D.y - 2.2))
        ear.addQuadCurve(to: outerBase3D, control: CGPoint(x: (outerBase3D.x + tipRight.x) * 0.5 + sign * 2.5, y: (outerBase3D.y + tipRight.y) * 0.5 - 2.0))
        ear.closeSubpath()

        ctx.saveGState()
        ctx.setFillColor(earTailColor(skin).cgColor)
        ctx.addPath(ear)
        ctx.fillPath()
        if showBorder {
            ctx.setStrokeColor(skin.palette.outline)
            ctx.setLineWidth(1.6)
            ctx.addPath(ear)
            ctx.strokePath()
        }

        if isSiamese {
            let inI = CGPoint(x: innerBase3D.x + (tip3D.x - innerBase3D.x) * 0.24 + sign * 3.0, y: innerBase3D.y + (tip3D.y - innerBase3D.y) * 0.16)
            let outI = CGPoint(x: outerBase3D.x + (tip3D.x - outerBase3D.x) * 0.20 - sign * 3.8, y: outerBase3D.y + (tip3D.y - outerBase3D.y) * 0.18)
            let tipI = CGPoint(x: (innerBase3D.x + outerBase3D.x) * 0.16 + tip3D.x * 0.68, y: (innerBase3D.y + outerBase3D.y) * 0.14 + tip3D.y * 0.72 + 2.5)
            let inner = CGMutablePath()
            inner.move(to: inI)
            inner.addQuadCurve(to: tipI, control: CGPoint(x: (inI.x + tipI.x) * 0.5 - sign * 0.8, y: (inI.y + tipI.y) * 0.5))
            inner.addQuadCurve(to: outI, control: CGPoint(x: (outI.x + tipI.x) * 0.5 + sign * 1.2, y: (outI.y + tipI.y) * 0.5))
            inner.closeSubpath()
            ctx.setFillColor(Self.siameseInnerEar.cgColor)
            ctx.addPath(inner)
            ctx.fillPath()
        }
        ctx.restoreGState()
    }

    // MARK: - 2.5D Head Features (Luminous Eyes & Whiskers)

    private func drawHeadFeatures(in ctx: CGContext, center: CGPoint, physics: CatPhysics, skin: SkinSpec, eyeTheme: EyeColorTheme) {
        ctx.saveGState()
        ctx.translateBy(x: center.x, y: center.y)
        ctx.rotate(by: physics.headTilt)

        let yawRad = physics.headYaw * 0.50
        let pitchRad = physics.headPitch * 0.36

        drawEye3D(in: ctx, isLeft: true, yawRad: yawRad, pitchRad: pitchRad, physics: physics, skin: skin, eyeTheme: eyeTheme)
        drawEye3D(in: ctx, isLeft: false, yawRad: yawRad, pitchRad: pitchRad, physics: physics, skin: skin, eyeTheme: eyeTheme)

        // Delicate small nose & `ω` mouth on all skins (projected in 3D head space)
        let nose = projectHead3D(x: 0, y: 8.2, z: 29.0, yawRad: yawRad, pitchRad: pitchRad).pt
        let mouthL = projectHead3D(x: -4.2, y: 12.2, z: 28.4, yawRad: yawRad, pitchRad: pitchRad).pt
        let mouthM = projectHead3D(x: 0, y: 11.2, z: 28.8, yawRad: yawRad, pitchRad: pitchRad).pt
        let mouthR = projectHead3D(x: 4.2, y: 12.2, z: 28.4, yawRad: yawRad, pitchRad: pitchRad).pt
        ctx.saveGState()
        let nPath = CGMutablePath()
        nPath.move(to: CGPoint(x: nose.x - 2.3, y: nose.y - 1.1))
        nPath.addQuadCurve(to: CGPoint(x: nose.x + 2.3, y: nose.y - 1.1), control: CGPoint(x: nose.x, y: nose.y - 1.6))
        nPath.addQuadCurve(to: CGPoint(x: nose.x, y: nose.y + 1.5), control: CGPoint(x: nose.x + 1.8, y: nose.y + 0.6))
        nPath.addQuadCurve(to: CGPoint(x: nose.x - 2.3, y: nose.y - 1.1), control: CGPoint(x: nose.x - 1.8, y: nose.y + 0.6))
        nPath.closeSubpath()
        ctx.setFillColor(skin.palette.nose)
        ctx.addPath(nPath)
        ctx.fillPath()
        if physics.yawnLevel > 0.12 {
            // Cute rounded oval pink yawn mouth
            let yw = 7.4 * physics.yawnLevel
            let yh = 9.4 * physics.yawnLevel
            let yawnRect = CGRect(x: mouthM.x - yw * 0.5, y: mouthM.y - 1.0, width: yw, height: yh)
            ctx.setFillColor(CGColor(red: 0.90, green: 0.44, blue: 0.54, alpha: 1.0))
            ctx.fillEllipse(in: yawnRect)
            ctx.setStrokeColor(skin.palette.outline)
            ctx.setLineWidth(1.1)
            ctx.strokeEllipse(in: yawnRect)
        } else {
            ctx.setStrokeColor(skin.palette.mouth)
            ctx.setAlpha(0.75)
            ctx.setLineWidth(1.1)
            ctx.setLineCap(.round)
            ctx.move(to: mouthL)
            ctx.addQuadCurve(to: mouthM, control: CGPoint(x: (mouthL.x + mouthM.x) * 0.5, y: mouthL.y + 1.6))
            ctx.addQuadCurve(to: mouthR, control: CGPoint(x: (mouthM.x + mouthR.x) * 0.5, y: mouthR.y + 1.6))
            ctx.strokePath()
        }
        ctx.restoreGState()

        drawWhiskers3D(in: ctx, yawRad: yawRad, pitchRad: pitchRad, physics: physics, skin: skin)
        ctx.restoreGState()
    }

    private func drawEye3D(
        in ctx: CGContext,
        isLeft: Bool,
        yawRad: CGFloat,
        pitchRad: CGFloat,
        physics: CatPhysics,
        skin: SkinSpec,
        eyeTheme: EyeColorTheme
    ) {
        let sign: CGFloat = isLeft ? -1.0 : 1.0
        let eyeColor = eyeTheme.primaryNSColor

        let proj = projectHead3D(x: sign * 17.2, y: -2.0, z: 27.5, yawRad: yawRad, pitchRad: pitchRad)
        let eyeNormalAngle = yawRad + sign * 0.34
        let foreshortenX = max(0.64, min(1.06, cos(eyeNormalAngle) / cos(0.34)))
        let foreshortenY = max(0.78, min(1.03, cos(pitchRad * 0.85)))

        ctx.saveGState()
        ctx.translateBy(x: proj.pt.x, y: proj.pt.y)
        ctx.scaleBy(x: foreshortenX, y: foreshortenY)

        // Happy closed eyes when petting, yawning, idle grooming/scratching, or sleeping
        let isHappy = physics.pettingLevel > 0.35 || physics.idleAction != .none || physics.yawnLevel > 0.35
        if isHappy || physics.sleepLevel > 0.65 {
            ctx.setStrokeColor(eyeColor.cgColor)
            // Delicate, thinner eyelids for sleeping (1.45px) and soft squints (1.85px)
            ctx.setLineWidth(physics.sleepLevel > 0.5 ? 1.45 : 1.85)
            ctx.setLineCap(.round)
            ctx.move(to: CGPoint(x: -11.5, y: 1.0))
            ctx.addQuadCurve(to: CGPoint(x: 11.5, y: 1.0), control: CGPoint(x: sign * 1.5, y: 3.4))
            ctx.strokePath()
            ctx.restoreGState()
            return
        }

        let hunt = physics.huntingLevel
        ctx.rotate(by: sign * (0.06 * (1.0 - hunt * 0.6)))

        let isSiameseEye = (skin.id == .siamese)
        let rx: CGFloat = (isSiameseEye ? 11.4 + hunt * 1.3 : 12.0 + hunt * 1.1)
        let ry: CGFloat = (isSiameseEye ? 9.8 + hunt * 2.0 : 10.8 + hunt * 1.8)
        let eyeRect = CGRect(x: -rx, y: -ry, width: rx * 2, height: ry * 2)

        ctx.saveGState()
        ctx.addEllipse(in: eyeRect)
        ctx.clip()

        let closure = physics.blinkAmount
        if closure >= 1 {
            ctx.restoreGState()
            ctx.restoreGState()
            return
        }
        if closure > 0 {
            ctx.clip(to: CGRect(x: -rx - 2, y: -ry + 2 * ry * closure, width: 2 * rx + 4, height: 2 * ry * (1 - closure)))
        }

        ctx.setFillColor(eyeColor.cgColor)
        ctx.fill(eyeRect)

        let px = physics.eyeLookX * (4.2 - hunt * 1.2)
        let py = physics.eyeLookY * (3.6 - hunt * 1.0)
        let normalW: CGFloat = 5.0
        let normalH: CGFloat = 13.5
        let huntDiameter: CGFloat = 18.8
        let pupilW = normalW + (huntDiameter - normalW) * hunt
        let pupilH = normalH + (huntDiameter - normalH) * hunt
        let pupilRect = CGRect(x: px - pupilW * 0.5, y: py - pupilH * 0.5, width: pupilW, height: pupilH)

        ctx.setFillColor(Self.catBlack.cgColor)
        ctx.fillEllipse(in: pupilRect)

        if hunt > 0.25 {
            let glintAlpha = min(1.0, (hunt - 0.25) / 0.55) * 0.92
            ctx.setFillColor(NSColor(calibratedWhite: 1.0, alpha: glintAlpha).cgColor)
            let glintR: CGFloat = 2.1 + hunt * 0.9
            ctx.fillEllipse(in: CGRect(x: px - pupilW * 0.22 - glintR * 0.5, y: py - pupilH * 0.22 - glintR * 0.5, width: glintR * 2, height: glintR * 2))
        }

        ctx.restoreGState()
        ctx.restoreGState()
    }

    private func drawWhiskers3D(in ctx: CGContext, yawRad: CGFloat, pitchRad: CGFloat, physics: CatPhysics, skin: SkinSpec) {
        let wiggle = sin(physics.breathPhase * 2.0) * (physics.pettingLevel * 1.2)
        ctx.saveGState()
        let wColor = (skin.id == .siamese ? Self.siameseWhisker : Self.whiskerDefault).withAlphaComponent(0.68)
        ctx.setStrokeColor(wColor.cgColor)
        ctx.setLineWidth(1.0)
        ctx.setLineCap(.round)

        for sign in [-1.0, 1.0] as [CGFloat] {
            let rows: [(CGFloat, CGFloat, CGFloat)] = [
                (10.5, 36.5, 8.8 + wiggle),
                (14.2, 37.8, 14.2 + wiggle * 0.5),
                (18.0, 35.5, 19.6 - wiggle * 0.4),
            ]
            for (y0, xEnd, yEnd) in rows {
                let p0 = projectHead3D(x: sign * 24.0, y: y0, z: 23.0, yawRad: yawRad, pitchRad: pitchRad).pt
                let p2 = projectHead3D(x: sign * xEnd, y: yEnd, z: 14.0, yawRad: yawRad, pitchRad: pitchRad).pt
                let ctrl = CGPoint(x: (p0.x + p2.x) * 0.5, y: (p0.y + p2.y) * 0.5 - 0.8)
                ctx.move(to: p0)
                ctx.addQuadCurve(to: p2, control: ctrl)
            }
        }
        ctx.strokePath()
        ctx.restoreGState()
    }

    // MARK: - Front Paws (KuroCat Idle Sitting)

    private func drawFrontPaws(in ctx: CGContext, seatX: CGFloat, seatY: CGFloat, bodyTilt: CGFloat, physics: CatPhysics, skin: SkinSpec, showBorder: Bool) {
        let isSiamese = (skin.id == .siamese)
        let idleTopY: CGFloat = isSiamese ? -31 : -30
        let idleTopX: CGFloat = isSiamese ? 16.5 : 18.0
        let idleBotX: CGFloat = isSiamese ? 15.0 : 16.0
        let dangleY: CGFloat = physics.dragStretch * 22.0
        let chestParallax: CGFloat = physics.headYaw * 3.5

        let sy = physics.squashY
        let sx = 1.0 + (1.0 - sy) * 0.55

        ctx.saveGState()
        ctx.translateBy(x: seatX, y: seatY)
        ctx.rotate(by: bodyTilt)
        ctx.scaleBy(x: sx, y: sy)

        let leftLifting = (physics.idleAction == .lickingPaw && physics.idleActionBlend > 0.02)
        if !leftLifting {
            drawIdleFrontLeg(
                in: ctx,
                top: CGPoint(x: -idleTopX + chestParallax, y: idleTopY),
                bottom: CGPoint(x: -idleBotX + chestParallax * 0.4, y: 12 + dangleY),
                isLeft: true,
                skin: skin,
                showBorder: showBorder
            )
        }
        drawIdleFrontLeg(
            in: ctx,
            top: CGPoint(x: idleTopX + chestParallax, y: idleTopY),
            bottom: CGPoint(x: idleBotX + chestParallax * 0.4, y: 12 + dangleY),
            isLeft: false,
            skin: skin,
            showBorder: showBorder
        )
        ctx.restoreGState()
    }

    private func drawIdleFrontLeg(in ctx: CGContext, top: CGPoint, bottom: CGPoint, isLeft: Bool, skin: SkinSpec, showBorder: Bool) {
        let sign: CGFloat = isLeft ? -1.0 : 1.0
        ctx.saveGState()

        ctx.setFillColor(pawColor(skin).cgColor)
        // Full seamless leg-to-paw fill on dual-color cats so legs & chest blend with zero gap
        if skin.id == .orangeWhite || skin.id == .shorthair || skin.id == .white {
            let legFillRect = CGRect(x: bottom.x - 8.0, y: top.y, width: 16.0, height: bottom.y - top.y + 5.0)
            ctx.addPath(CGPath(roundedRect: legFillRect, cornerWidth: 6.5, cornerHeight: 6.5, transform: nil))
            ctx.fillPath()
        } else {
            let cuff = CGRect(x: bottom.x - 7.5, y: bottom.y - 13, width: 15, height: 18)
            let cuffPath = CGPath(roundedRect: cuff, cornerWidth: 6.5, cornerHeight: 6.5, transform: nil)
            ctx.addPath(cuffPath)
            ctx.fillPath()
        }

        // Smooth bottom U-rim on paw tip (never a closed box around the top!)
        if showBorder {
            ctx.setStrokeColor(skin.palette.outline)
            ctx.setLineWidth(1.3)
            ctx.setLineCap(.round)
            ctx.addArc(center: CGPoint(x: bottom.x, y: bottom.y - 1.0), radius: 6.8, startAngle: 0.15, endAngle: .pi - 0.15, clockwise: false)
            ctx.strokePath()
        }

        ctx.setStrokeColor(creaseColor(skin).cgColor)
        ctx.setLineWidth(1.45)
        ctx.setLineCap(.round)

        ctx.move(to: CGPoint(x: top.x + sign * 8.5, y: top.y))
        ctx.addQuadCurve(to: CGPoint(x: bottom.x + sign * 7.5, y: bottom.y - 1), control: CGPoint(x: top.x + sign * 9.5, y: (top.y + bottom.y) * 0.5))

        ctx.move(to: CGPoint(x: top.x - sign * 7.0, y: top.y + 4))
        ctx.addQuadCurve(to: CGPoint(x: bottom.x - sign * 6.0, y: bottom.y - 1), control: CGPoint(x: top.x - sign * 6.5, y: (top.y + bottom.y) * 0.5))
        ctx.strokePath()
        ctx.restoreGState()
    }

    // MARK: - Fitted Wireless Headphones (Matching Reference Image)

    private func cupPoint(side: CGFloat, yaw: CGFloat, yawRad: CGFloat, pitchRad: CGFloat) -> CGPoint {
        var pt = projectHead3D(x: side * 44.5, y: -1.0, z: -2.0, yawRad: yawRad, pitchRad: pitchRad).pt
        // Match the silhouette's cheekShift (yaw * 3.0) so cups stay pinned to the outer cheek edges
        let cheekEdge = side * 43.5 + yaw * 3.0
        if side < 0 {
            pt.x = min(pt.x, cheekEdge)
        } else {
            pt.x = max(pt.x, cheekEdge)
        }
        return pt
    }

    private func drawWirelessHeadphones(in ctx: CGContext, center: CGPoint, physics: CatPhysics, skin: SkinSpec, eyeTheme: EyeColorTheme) {
        ctx.saveGState()
        ctx.translateBy(x: center.x, y: center.y)
        ctx.rotate(by: physics.headTilt)
        ctx.setAlpha(physics.musicLevel)

        let yaw = physics.headYaw
        let yawRad = yaw * 0.50
        let pitchRad = physics.headPitch * 0.36
        let farSign: CGFloat = yaw >= 0 ? 1.0 : -1.0
        let nearSign: CGFloat = -farSign

        let leftCup = cupPoint(side: -1, yaw: yaw, yawRad: yawRad, pitchRad: pitchRad)
        let rightCup = cupPoint(side: 1, yaw: yaw, yawRad: yawRad, pitchRad: pitchRad)

        drawWirelessCup(side: farSign, cup: farSign < 0 ? leftCup : rightCup, physics: physics, skin: skin, eyeTheme: eyeTheme, in: ctx)

        // Headband uses z: 0..2 so pitch never pushes the band down over the eyes
        let crownShift = yaw * 1.5
        let bandPts: [CGPoint] = [
            leftCup,
            CGPoint(x: -31 + crownShift, y: -28),
            CGPoint(x: crownShift, y: -34.5),
            CGPoint(x: 31 + crownShift, y: -28),
            rightCup,
        ]
        let bandPath = smoothOpenPath(through: bandPts)
        ctx.addPath(bandPath)
        ctx.setStrokeColor(CGColor(red: 0.10, green: 0.11, blue: 0.14, alpha: 1.0))
        ctx.setLineWidth(6.2)
        ctx.setLineCap(.round)
        ctx.strokePath()

        ctx.addPath(bandPath)
        ctx.setStrokeColor(CGColor(red: 0.24, green: 0.26, blue: 0.31, alpha: 1.0))
        ctx.setLineWidth(4.0)
        ctx.strokePath()

        // Crisp subtle top highlight arc on headband
        let hlPts = [
            CGPoint(x: -16 + crownShift, y: -32.8),
            CGPoint(x: crownShift, y: -34.8),
            CGPoint(x: 16 + crownShift, y: -32.8),
        ]
        ctx.addPath(smoothOpenPath(through: hlPts))
        ctx.setStrokeColor(CGColor(red: 0.85, green: 0.88, blue: 0.92, alpha: 0.70))
        ctx.setLineWidth(1.1)
        ctx.strokePath()

        drawWirelessCup(side: nearSign, cup: nearSign < 0 ? leftCup : rightCup, physics: physics, skin: skin, eyeTheme: eyeTheme, in: ctx)
        ctx.restoreGState()
    }

    private func drawWirelessCup(side: CGFloat, cup: CGPoint, physics: CatPhysics, skin: SkinSpec, eyeTheme: EyeColorTheme, in ctx: CGContext) {
        let depthScale = 1.0 - side * physics.headYaw * 0.14
        let pulse = 1.0 + (0.5 - 0.5 * cos(physics.musicBeatPhase)) * 0.04
        let w: CGFloat = 7.4 * max(0.75, depthScale) * pulse
        let h: CGFloat = 17.0 * pulse
        let ringColor = eyeTheme.primaryNSColor.cgColor

        ctx.saveGState()
        ctx.translateBy(x: cup.x, y: cup.y)
        ctx.rotate(by: -side * 0.08)

        // Dark outer earcup shell
        let cupRect = CGRect(x: -w * 0.65, y: -h * 0.52, width: w * 1.3, height: h * 1.04)
        ctx.setFillColor(CGColor(red: 0.15, green: 0.17, blue: 0.21, alpha: 1.0))
        ctx.fillEllipse(in: cupRect)
        ctx.setStrokeColor(CGColor(red: 0.28, green: 0.31, blue: 0.36, alpha: 0.9))
        ctx.setLineWidth(1.2)
        ctx.strokeEllipse(in: cupRect)

        // Thin glowing ring matching the cat's eye color
        let ringRect = CGRect(x: -w * 0.36 + side * 0.8, y: -h * 0.34, width: w * 0.72, height: h * 0.68)
        ctx.setStrokeColor(ringColor)
        ctx.setLineWidth(1.4)
        ctx.strokeEllipse(in: ringRect)

        ctx.restoreGState()
    }

    // MARK: - Idle Limbs (Licking Paw / Scratching)

    private func drawIdleLimbs(in ctx: CGContext, seatX: CGFloat, seatY: CGFloat, headCenter: CGPoint, physics: CatPhysics, skin: SkinSpec) {
        let blend = physics.idleActionBlend
        guard blend > 0.02 else { return }

        let yawRad = physics.headYaw * 0.50
        let pitchRad = physics.headPitch * 0.36
        ctx.saveGState()

        switch physics.idleAction {
        case .lickingPaw:
            let start = CGPoint(x: seatX - 18, y: seatY - 30)
            let mLocal = projectHead3D(x: -5, y: 14, z: 26, yawRad: yawRad, pitchRad: pitchRad).pt
            let mouth = CGPoint(x: headCenter.x + mLocal.x, y: headCenter.y + mLocal.y)
            let bob = sin(physics.idleActionPhase * 11.0) * 2.5 * blend
            let pawPos = CGPoint(x: lerp(seatX - 16, mouth.x, blend), y: lerp(seatY + 12, mouth.y + 4.0 + bob, blend))

            // Smoothly animates the SAME left front leg from resting position up to mouth
            ctx.setStrokeColor(skin.palette.outline)
            ctx.setLineWidth(13.0)
            ctx.setLineCap(.round)
            ctx.move(to: start)
            ctx.addQuadCurve(to: pawPos, control: CGPoint(x: start.x - 9 * blend, y: (start.y + pawPos.y) * 0.5))
            ctx.strokePath()

            ctx.setStrokeColor(pawColor(skin).cgColor)
            ctx.setLineWidth(10.5)
            ctx.move(to: start)
            ctx.addQuadCurve(to: pawPos, control: CGPoint(x: start.x - 9 * blend, y: (start.y + pawPos.y) * 0.5))
            ctx.strokePath()

            ctx.setFillColor(pawColor(skin).cgColor)
            ctx.fillEllipse(in: CGRect(x: pawPos.x - 7.5, y: pawPos.y - 6.0, width: 15, height: 12))

        case .scratching:
            let hip = CGPoint(x: seatX + 26, y: seatY - 6)
            let cLocal = projectHead3D(x: 42, y: -4, z: 8, yawRad: yawRad, pitchRad: pitchRad).pt
            let target = CGPoint(x: headCenter.x + cLocal.x, y: headCenter.y + cLocal.y)
            let kick = sin(physics.idleActionPhase * 22.0) * 6.0 * blend
            let footPos = CGPoint(x: lerp(seatX + 26, target.x + 4, blend), y: lerp(seatY + 10, target.y + kick, blend))

            // Smoothly animates the SAME right hind leg from resting tuck up to cheek/ear
            ctx.setStrokeColor(skin.palette.outline)
            ctx.setLineWidth(12.5)
            ctx.setLineCap(.round)
            ctx.move(to: hip)
            ctx.addQuadCurve(to: footPos, control: CGPoint(x: hip.x + 22 * blend, y: (hip.y + footPos.y) * 0.55))
            ctx.strokePath()

            ctx.setStrokeColor(bodyColor(skin).cgColor)
            ctx.setLineWidth(10.0)
            ctx.move(to: hip)
            ctx.addQuadCurve(to: footPos, control: CGPoint(x: hip.x + 22 * blend, y: (hip.y + footPos.y) * 0.55))
            ctx.strokePath()

            ctx.setFillColor(pawColor(skin).cgColor)
            ctx.fillEllipse(in: CGRect(x: footPos.x - 7.0, y: footPos.y - 5.5, width: 14, height: 11))

        case .none:
            break
        }
        ctx.restoreGState()
    }

    // MARK: - Particles & Helpers

    private func drawParticles(in ctx: CGContext, physics: CatPhysics) {
        for p in physics.particles {
            let progress = p.age / p.lifetime
            let alpha = progress < 0.2 ? (progress / 0.2) : max(0, 1.0 - (progress - 0.2) / 0.8)

            ctx.saveGState()
            ctx.setAlpha(alpha)

            let symbol: String
            switch p.kind {
            case .noteSingle: symbol = "♪"
            case .noteDouble: symbol = "♫"
            case .sparkle: symbol = "✦"
            case .heart: symbol = "♥"
            case .sleepZ: symbol = "z"
            }

            let fontSize = 15.0 * p.scale
            let font = NSFont.systemFont(ofSize: fontSize, weight: .bold)
            let nsColor = NSColor(cgColor: p.color) ?? .white
            let attrs: [NSAttributedString.Key: Any] = [.font: font, .foregroundColor: nsColor]
            let attrStr = NSAttributedString(string: symbol, attributes: attrs)

            NSGraphicsContext.saveGraphicsState()
            NSGraphicsContext.current = NSGraphicsContext(cgContext: ctx, flipped: true)
            attrStr.draw(at: CGPoint(x: p.x - fontSize * 0.35, y: p.y - fontSize * 0.5))
            NSGraphicsContext.restoreGraphicsState()

            ctx.restoreGState()
        }
    }

    private func smoothOpenPath(through points: [CGPoint]) -> CGPath {
        let path = CGMutablePath()
        guard points.count >= 2 else { return path }

        path.move(to: points[0])
        if points.count == 2 {
            path.addLine(to: points[1])
            return path
        }
        for i in 1..<(points.count - 1) {
            let mid = CGPoint(x: (points[i].x + points[i + 1].x) * 0.5, y: (points[i].y + points[i + 1].y) * 0.5)
            path.addQuadCurve(to: mid, control: points[i])
        }
        path.addLine(to: points[points.count - 1])
        return path
    }
}

private func lerp(_ a: CGFloat, _ b: CGFloat, _ t: CGFloat) -> CGFloat {
    a + (b - a) * t
}
