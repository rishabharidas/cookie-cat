import AppKit
import CoreGraphics

struct FloatingParticle {
    enum Kind {
        case noteSingle   // ♪
        case noteDouble   // ♫
        case sparkle      // ✦
        case heart        // ♥
        case sleepZ       // z
    }

    var kind: Kind
    var x: CGFloat
    var y: CGFloat
    var vx: CGFloat
    var vy: CGFloat
    var age: CGFloat
    var lifetime: CGFloat
    var scale: CGFloat
    var phase: CGFloat
    var color: CGColor
}

enum IdleAction {
    case none
    case lickingPaw
    case scratching
}

/// Drives 2.5D gaze tracking, hunting/boba eyes, blinking, petting, sleep, idle grooming/scratching, and music vibe.
final class CatPhysics {
    // Head & eye orientation in normalized [-1, 1] space
    var headYaw: CGFloat = 0
    var headPitch: CGFloat = 0
    var headTilt: CGFloat = 0
    var eyeLookX: CGFloat = 0
    var eyeLookY: CGFloat = 0

    // Expressive state levels [0, 1]
    var blinkAmount: CGFloat = 0
    var huntingLevel: CGFloat = 0
    var pettingLevel: CGFloat = 0
    var yawnLevel: CGFloat = 0
    var sleepLevel: CGFloat = 0
    var musicLevel: CGFloat = 0

    // Natural cat idle actions (licking paw, scratching ear)
    var idleAction: IdleAction = .none
    var idleActionBlend: CGFloat = 0
    var idleActionPhase: CGFloat = 0

    // Continuous motion phases
    var breathPhase: CGFloat = 0
    var tailPhase: CGFloat = 0
    var musicBeatPhase: CGFloat = 0
    var earTwitchLeft: CGFloat = 0
    var earTwitchRight: CGFloat = 0

    // Body squash/stretch & drag inertia physics
    var squashY: CGFloat = 1.0
    var headBobY: CGFloat = 0
    var isDragging: Bool = false
    var dragStretch: CGFloat = 0
    var dragVelocity: CGPoint = .zero
    var dragSwayAngle: CGFloat = 0
    var highFrameRate: Bool = false

    // Particles (music notes, sparkles, sleep z)
    var particles: [FloatingParticle] = []

    // Internal timers
    private var lastCursorPos: NSPoint?
    private var lastCursorMoveTime: TimeInterval = ProcessInfo.processInfo.systemUptime
    private var lastUpdateTime: TimeInterval = ProcessInfo.processInfo.systemUptime
    private var blinkTimer: CGFloat = 2.8
    private var isBlinking: Bool = false
    private var blinkProgress: CGFloat = 0
    private var nextEarTwitchTimer: CGFloat = 4.5
    private var noteSpawnTimer: CGFloat = 0.2
    private var sleepZTimer: CGFloat = 0.5
    private var petSparkleTimer: CGFloat = 0.1
    private var nextIdleActionTimer: CGFloat = 6.5
    private var idleActionRemaining: CGFloat = 0
    private var yawnProgress: CGFloat = 0
    private var hasYawnedThisIdle: Bool = false

    /// Recommended frame interval for adaptive power saving (60fps if enabled, 30fps active, 10fps sleepy idle).
    var targetFrameInterval: TimeInterval {
        if sleepLevel > 0.85 && musicLevel < 0.05 && pettingLevel < 0.05 && !isDragging {
            return 1.0 / 10.0
        }
        return highFrameRate ? (1.0 / 60.0) : (1.0 / 30.0)
    }

    func triggerPet(atLocalPoint point: CGPoint) {
        pettingLevel = 1.0
        sleepLevel = 0.0
        lastCursorMoveTime = ProcessInfo.processInfo.systemUptime

        if particles.count < 24 {
            let isHeart = Bool.random()
            particles.append(FloatingParticle(
                kind: isHeart ? .heart : .sparkle,
                x: point.x + CGFloat.random(in: -14...14),
                y: point.y - 10,
                vx: CGFloat.random(in: -14...14),
                vy: CGFloat.random(in: -38 ... -24),
                age: 0,
                lifetime: 1.1,
                scale: CGFloat.random(in: 0.85...1.2),
                phase: CGFloat.random(in: 0...6.28),
                color: isHeart
                    ? CGColor(red: 1.0, green: 0.45, blue: 0.62, alpha: 1.0)
                    : CGColor(red: 1.0, green: 0.86, blue: 0.35, alpha: 1.0)
            ))
        }
    }

    func update(
        now: TimeInterval,
        catWindowFrame: NSRect,
        cursorGlobalPos: NSPoint,
        isListeningMusic: Bool,
        audioEnergy: CGFloat = 0,
        beatPulse: CGFloat = 0,
        beatRate: CGFloat = 6.2
    ) {
        let rawDt = now - lastUpdateTime
        lastUpdateTime = now
        let dt = CGFloat(min(max(rawDt, 0.001), 0.1))

        // 1. Cursor delta & velocity for hunting mode + idle sleep
        var cursorSpeed: CGFloat = 0
        if let prev = lastCursorPos {
            let dx = cursorGlobalPos.x - prev.x
            let dy = cursorGlobalPos.y - prev.y
            let dist = hypot(dx, dy)
            cursorSpeed = dist / max(dt, 0.001)
            if dist > 1.5 {
                lastCursorMoveTime = now
            }
        }
        lastCursorPos = cursorGlobalPos

        let idleSeconds = now - lastCursorMoveTime

        // Hunting / boba eyes trigger on fast mouse movement
        if cursorSpeed > 950 {
            huntingLevel = min(1.0, huntingLevel + dt * 6.5)
        } else {
            huntingLevel = max(0.0, huntingLevel - dt * 0.85)
        }

        // Hover-petting: gently moving the mouse around the cat's head pets it!
        let scale = max(catWindowFrame.width / 300.0, 0.1)
        let localX = (cursorGlobalPos.x - catWindowFrame.minX) / scale
        let localY = (catWindowFrame.maxY - cursorGlobalPos.y) / scale
        let headCx = 150.0 + headYaw * 7.5
        let headCy = 122.0 + headBobY
        let hdx = (localX - headCx) / 46.0
        let hdy = (localY - headCy) / 40.0
        let onHead = (hdx * hdx + hdy * hdy) <= 1.0

        if onHead && cursorSpeed > 25 && cursorSpeed < 750 && !isDragging {
            pettingLevel = min(1.0, pettingLevel + dt * 3.2)
            sleepLevel = max(0.0, sleepLevel - dt * 3.0)
            yawnLevel = 0
            hasYawnedThisIdle = false
        } else {
            // Petting decay
            pettingLevel = max(0.0, pettingLevel - dt * 0.9)
        }

        if idleSeconds < 5.0 {
            hasYawnedThisIdle = false
            yawnProgress = 0
            yawnLevel = 0
        }

        // Pre-sleep yawn (~28s idle) right before curling up to sleep at 30s
        let canRest = !isListeningMusic && pettingLevel < 0.1 && !isDragging
        if canRest && idleSeconds > 28.0 && idleSeconds <= 30.2 && !hasYawnedThisIdle {
            yawnProgress += dt / 1.8
            if yawnProgress >= 1.0 {
                yawnProgress = 0
                yawnLevel = 0
                hasYawnedThisIdle = true
            } else {
                yawnLevel = sin(yawnProgress * .pi)
            }
        } else {
            yawnLevel = max(0.0, yawnLevel - dt * 4.0)
        }

        // Curled-up donut sleep when idle for > 30s and not listening to music
        let shouldSleep = (idleSeconds > 30.0) && canRest && yawnLevel < 0.1
        let targetSleep: CGFloat = shouldSleep ? 1.0 : 0.0
        sleepLevel = lerp(sleepLevel, targetSleep, dt * (shouldSleep ? 0.85 : 3.5))

        // Music mode smooth transition
        let targetMusic: CGFloat = isListeningMusic ? 1.0 : 0.0
        musicLevel = lerp(musicLevel, targetMusic, dt * 4.5)

        // 2. 2.5D Gaze Tracking toward mouse cursor
        // Window center in global screen coordinates (AppKit bottom-left origin)
        let catCenterX = catWindowFrame.midX
        let catHeadCenterY = catWindowFrame.minY + catWindowFrame.height * 0.56

        let normX = clamp((cursorGlobalPos.x - catCenterX) / 420.0, -1.0, 1.0)
        // AppKit Y increases upward; in head space pitch > 0 means looking down
        let normY = clamp((catHeadCenterY - cursorGlobalPos.y) / 340.0, -1.0, 1.0)

        let gazeWeight = (1.0 - sleepLevel * 0.95) * (1.0 - idleActionBlend) * (1.0 - yawnLevel)
        var targetYaw = normX * gazeWeight - sleepLevel * 0.15
        let tiltAmp = 0.055 + audioEnergy * 0.045
        var targetTilt = (normX * 0.10 * gazeWeight) + sin(musicBeatPhase * 0.5) * tiltAmp * musicLevel - (sleepLevel * 0.16)
        var targetPitch = (normY * gazeWeight) + (sleepLevel * 0.22) - (yawnLevel * 0.28) + beatPulse * 0.045 * musicLevel

        if idleActionBlend > 0.01 {
            switch idleAction {
            case .lickingPaw:
                targetYaw += -0.22 * idleActionBlend
                targetPitch += 0.24 * idleActionBlend
                targetTilt += -0.14 * idleActionBlend
            case .scratching:
                targetYaw += 0.24 * idleActionBlend
                targetPitch += 0.16 * idleActionBlend
                targetTilt += (0.26 + sin(idleActionPhase * 18.0) * 0.05) * idleActionBlend
            case .none:
                break
            }
        }

        let followSpeed: CGFloat = (huntingLevel > 0.3 ? 16.0 : 10.5)
        headYaw = smoothLerp(headYaw, targetYaw, speed: followSpeed, dt: dt)
        headPitch = smoothLerp(headPitch, targetPitch, speed: followSpeed, dt: dt)

        // Drag pendulum sway when holding & moving the cat
        let targetSway = isDragging ? clamp(-dragVelocity.x * 0.00045, -0.30, 0.30) : 0.0
        dragSwayAngle = smoothLerp(dragSwayAngle, targetSway, speed: isDragging ? 14.0 : 8.0, dt: dt)
        dragVelocity.x = smoothLerp(dragVelocity.x, 0, speed: 10.0, dt: dt)
        dragVelocity.y = smoothLerp(dragVelocity.y, 0, speed: 10.0, dt: dt)

        headTilt = smoothLerp(headTilt, targetTilt + dragSwayAngle * 0.65, speed: 9.0, dt: dt)

        eyeLookX = smoothLerp(eyeLookX, normX * gazeWeight, speed: 15.0, dt: dt)
        eyeLookY = smoothLerp(eyeLookY, normY * gazeWeight, speed: 15.0, dt: dt)

        // 3. Breathing, tail wave, and music beat
        breathPhase += dt * (sleepLevel > 0.5 ? 1.4 : 2.2)
        tailPhase += dt * (1.9 + huntingLevel * 4.2 + musicLevel * (1.4 + audioEnergy * 1.4))
        if musicLevel > 0.05 {
            musicBeatPhase += dt * beatRate
        }

        let breathSquash = sin(breathPhase) * 0.012
        let huntCrouch = -huntingLevel * 0.042
        let beatAmp = 0.012 + audioEnergy * 0.018 + beatPulse * 0.018
        let musicBob = (0.5 - 0.5 * cos(musicBeatPhase)) * beatAmp * musicLevel
        let sleepLoaf = sleepLevel * 0.32
        let yawnStretch = yawnLevel * 0.05
        dragStretch = smoothLerp(dragStretch, isDragging ? 0.14 : 0.0, speed: 10.0, dt: dt)
        squashY = 1.0 + breathSquash + huntCrouch - musicBob + dragStretch - sleepLoaf + yawnStretch
        // Coupled directly to squashY; head settles cozily down into curled donut when sleeping
        headBobY = (1.0 - squashY) * 86.0 + sleepLevel * 20.0 - yawnLevel * 4.0

        // 3.5 Natural cat idle actions (scratching ear / licking paw)
        if huntingLevel > 0.35 || sleepLevel > 0.4 || pettingLevel > 0.3 {
            idleActionRemaining = 0
        }
        if idleActionRemaining > 0 {
            idleActionRemaining -= dt
            idleActionPhase += dt
            idleActionBlend = lerp(idleActionBlend, 1.0, dt * 7.5)
            if idleActionRemaining <= 0 {
                nextIdleActionTimer = CGFloat.random(in: 9.0...17.0)
            }
        } else {
            idleActionBlend = lerp(idleActionBlend, 0.0, dt * 6.5)
            if idleActionBlend < 0.02 {
                idleAction = .none
                idleActionPhase = 0
            }
            if sleepLevel < 0.3 && huntingLevel < 0.25 && pettingLevel < 0.2 {
                nextIdleActionTimer -= dt
                if nextIdleActionTimer <= 0 {
                    idleAction = Bool.random() ? .lickingPaw : .scratching
                    idleActionRemaining = CGFloat.random(in: 2.8...4.0)
                    idleActionPhase = 0
                }
            }
        }

        // 4. Blinking
        if isBlinking {
            blinkProgress += dt * 7.5
            if blinkProgress >= 1.0 {
                isBlinking = false
                blinkProgress = 0
                blinkAmount = 0
                blinkTimer = CGFloat.random(in: 2.2...5.4)
            } else {
                blinkAmount = sin(blinkProgress * .pi)
            }
        } else {
            blinkTimer -= dt
            if blinkTimer <= 0 && sleepLevel < 0.5 && pettingLevel < 0.3 {
                isBlinking = true
                blinkProgress = 0
            }
        }

        // 5. Ear twitches
        nextEarTwitchTimer -= dt
        if nextEarTwitchTimer <= 0 {
            nextEarTwitchTimer = CGFloat.random(in: 3.0...7.5)
            if Bool.random() {
                earTwitchLeft = 1.0
            } else {
                earTwitchRight = 1.0
            }
        }
        earTwitchLeft = max(0, earTwitchLeft - dt * 5.0)
        earTwitchRight = max(0, earTwitchRight - dt * 5.0)

        // 6. Spawn & update floating particles
        updateParticles(dt: dt, isListeningMusic: isListeningMusic)
    }

    private func updateParticles(dt: CGFloat, isListeningMusic: Bool) {
        // Music notes when listening
        if isListeningMusic && musicLevel > 0.35 {
            noteSpawnTimer -= dt
            if noteSpawnTimer <= 0 {
                noteSpawnTimer = CGFloat.random(in: 0.45...0.85)
                let side: CGFloat = Bool.random() ? 1.0 : -1.0
                let isDouble = Bool.random()
                let noteColors: [CGColor] = [
                    CGColor(red: 0.40, green: 0.85, blue: 1.00, alpha: 1.0),
                    CGColor(red: 1.00, green: 0.52, blue: 0.76, alpha: 1.0),
                    CGColor(red: 0.65, green: 0.95, blue: 0.58, alpha: 1.0),
                    CGColor(red: 1.00, green: 0.84, blue: 0.38, alpha: 1.0),
                ]
                particles.append(FloatingParticle(
                    kind: isDouble ? .noteDouble : .noteSingle,
                    x: 150 + side * CGFloat.random(in: 48...64),
                    y: 96 + CGFloat.random(in: -10...10),
                    vx: side * CGFloat.random(in: 8...18),
                    vy: CGFloat.random(in: -34 ... -22),
                    age: 0,
                    lifetime: 1.65,
                    scale: CGFloat.random(in: 0.9...1.25),
                    phase: CGFloat.random(in: 0...6.28),
                    color: noteColors.randomElement()!
                ))
            }
        }

        // Petting hearts & sparkles (on both click-pet and hover-pet)
        if pettingLevel > 0.55 {
            petSparkleTimer -= dt
            if petSparkleTimer <= 0 {
                petSparkleTimer = 0.24
                let isHeart = Bool.random()
                particles.append(FloatingParticle(
                    kind: isHeart ? .heart : .sparkle,
                    x: 150 + CGFloat.random(in: -35...35),
                    y: 78 + CGFloat.random(in: -10...10),
                    vx: CGFloat.random(in: -12...12),
                    vy: CGFloat.random(in: -34 ... -20),
                    age: 0,
                    lifetime: 1.05,
                    scale: CGFloat.random(in: 0.85...1.20),
                    phase: CGFloat.random(in: 0...6.28),
                    color: isHeart
                        ? CGColor(red: 1.0, green: 0.45, blue: 0.62, alpha: 1.0)
                        : CGColor(red: 1.0, green: 0.88, blue: 0.40, alpha: 1.0)
                ))
            }
        }

        // Sleep z particles
        if sleepLevel > 0.75 {
            sleepZTimer -= dt
            if sleepZTimer <= 0 {
                sleepZTimer = 1.25
                particles.append(FloatingParticle(
                    kind: .sleepZ,
                    x: 184,
                    y: 88,
                    vx: 10,
                    vy: -20,
                    age: 0,
                    lifetime: 2.0,
                    scale: 1.0,
                    phase: CGFloat.random(in: 0...6.28),
                    color: CGColor(red: 0.78, green: 0.86, blue: 1.0, alpha: 1.0)
                ))
            }
        }

        // Advance and prune
        for i in particles.indices {
            particles[i].age += dt
            particles[i].x += (particles[i].vx + sin(particles[i].age * 3.5 + particles[i].phase) * 9.0) * dt
            particles[i].y += particles[i].vy * dt
        }
        particles.removeAll { $0.age >= $0.lifetime }
    }
}

private func lerp(_ a: CGFloat, _ b: CGFloat, _ t: CGFloat) -> CGFloat {
    let clamped = min(max(t, 0), 1)
    return a + (b - a) * clamped
}

private func smoothLerp(_ a: CGFloat, _ b: CGFloat, speed: CGFloat, dt: CGFloat) -> CGFloat {
    let t = 1.0 - exp(-speed * dt)
    return a + (b - a) * t
}

private func clamp(_ v: CGFloat, _ lo: CGFloat, _ hi: CGFloat) -> CGFloat {
    min(max(v, lo), hi)
}
