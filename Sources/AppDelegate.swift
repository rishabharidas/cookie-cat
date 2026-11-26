import AppKit
import CoreGraphics

final class AppDelegate: NSObject, NSApplicationDelegate {
    private var statusItem: NSStatusItem!
    private var overlayWindow: OverlayWindow!

    private let physics = CatPhysics()
    private let renderer = CatRenderer()
    private let musicMonitor = MusicMonitor()

    private var timer: Timer?
    private var lastFrameInterval: TimeInterval = 1.0 / 30.0
    private var lastIconListeningState: Bool?
    private var statusMenuItem: NSMenuItem?

    // Persisted preferences
    private enum PrefKey {
        static let skin = "cookie.skin"
        static let eyeTheme = "cookie.eyeTheme"
        static let showBorder = "cookie.showBorder"
        static let highFPS = "cookie.highFPS"
        static let scale = "cookie.scale"
        static let musicMode = "cookie.musicMode"
        static let clickThrough = "cookie.clickThrough"
        static let posX = "cookie.posX"
        static let posY = "cookie.posY"
        static let hasPos = "cookie.hasPos"
    }

    private var currentSkinId: CatSkinId {
        get {
            let raw = UserDefaults.standard.string(forKey: PrefKey.skin) ?? CatSkinId.black.rawValue
            return CatSkinId(rawValue: raw) ?? .black
        }
        set {
            UserDefaults.standard.set(newValue.rawValue, forKey: PrefKey.skin)
        }
    }

    private var currentEyeTheme: EyeColorTheme {
        get {
            let raw = UserDefaults.standard.string(forKey: PrefKey.eyeTheme) ?? EyeColorTheme.chartreuse.rawValue
            return EyeColorTheme(rawValue: raw) ?? .chartreuse
        }
        set {
            UserDefaults.standard.set(newValue.rawValue, forKey: PrefKey.eyeTheme)
        }
    }

    private var showBorder: Bool {
        get {
            if UserDefaults.standard.object(forKey: PrefKey.showBorder) == nil {
                return true
            }
            return UserDefaults.standard.bool(forKey: PrefKey.showBorder)
        }
        set {
            UserDefaults.standard.set(newValue, forKey: PrefKey.showBorder)
        }
    }

    private var highFrameRate: Bool {
        get { UserDefaults.standard.bool(forKey: PrefKey.highFPS) }
        set {
            UserDefaults.standard.set(newValue, forKey: PrefKey.highFPS)
            physics.highFrameRate = newValue
        }
    }

    private var currentScale: CGFloat {
        get {
            let val = UserDefaults.standard.double(forKey: PrefKey.scale)
            return val > 0.2 ? CGFloat(val) : 1.0
        }
        set {
            UserDefaults.standard.set(Double(newValue), forKey: PrefKey.scale)
            resizeOverlayWindow()
        }
    }

    private var isClickThrough: Bool {
        get { UserDefaults.standard.bool(forKey: PrefKey.clickThrough) }
        set { UserDefaults.standard.set(newValue, forKey: PrefKey.clickThrough) }
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(.accessory)

        if let savedMode = UserDefaults.standard.string(forKey: PrefKey.musicMode),
           let mode = MusicMode(rawValue: savedMode) {
            musicMonitor.mode = mode
        }
        physics.highFrameRate = highFrameRate

        setupOverlayWindow()
        setupStatusItem()
        startAnimationLoop(interval: 1.0 / 30.0)
    }

    private func setupOverlayWindow() {
        let scale = currentScale
        let size = NSSize(
            width: CatRenderer.logicalSize.width * scale,
            height: CatRenderer.logicalSize.height * scale
        )
        let origin = initialWindowOrigin(for: size)

        overlayWindow = OverlayWindow(
            contentRect: NSRect(origin: origin, size: size),
            physics: physics,
            renderer: renderer,
            skinProvider: { [weak self] in
                SkinSpec.builtIn(self?.currentSkinId ?? .black)
            },
            eyeProvider: { [weak self] in
                self?.currentEyeTheme ?? .chartreuse
            },
            borderProvider: { [weak self] in
                self?.showBorder ?? true
            },
            scaleProvider: { [weak self] in
                self?.currentScale ?? 1.0
            },
            onPositionChanged: { origin in
                UserDefaults.standard.set(true, forKey: PrefKey.hasPos)
                UserDefaults.standard.set(Double(origin.x), forKey: PrefKey.posX)
                UserDefaults.standard.set(Double(origin.y), forKey: PrefKey.posY)
            },
            contextMenuProvider: { [weak self] in
                self?.buildMenu() ?? NSMenu()
            }
        )
        overlayWindow.orderFrontRegardless()
    }

    private func initialWindowOrigin(for size: NSSize) -> NSPoint {
        if UserDefaults.standard.bool(forKey: PrefKey.hasPos) {
            let x = UserDefaults.standard.double(forKey: PrefKey.posX)
            let y = UserDefaults.standard.double(forKey: PrefKey.posY)
            let pt = NSPoint(x: x, y: y)
            if NSScreen.screens.contains(where: { $0.visibleFrame.intersects(NSRect(origin: pt, size: size)) }) {
                return pt
            }
        }
        let visible = NSScreen.main?.visibleFrame ?? NSRect(x: 0, y: 0, width: 1280, height: 800)
        return NSPoint(
            x: visible.maxX - size.width - 28,
            y: visible.minY + 12
        )
    }

    private func resizeOverlayWindow() {
        guard let win = overlayWindow else { return }
        let oldFrame = win.frame
        let scale = currentScale
        let newSize = NSSize(
            width: CatRenderer.logicalSize.width * scale,
            height: CatRenderer.logicalSize.height * scale
        )
        let newOrigin = NSPoint(
            x: oldFrame.midX - newSize.width * 0.5,
            y: oldFrame.minY
        )
        win.setFrame(NSRect(origin: newOrigin, size: newSize), display: true)
        win.catView.frame = NSRect(origin: .zero, size: newSize)
    }

    private func setupStatusItem() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        if let button = statusItem.button {
            button.target = self
            button.action = #selector(statusItemClicked(_:))
            button.sendAction(on: [.leftMouseUp, .rightMouseUp])
        }
        updateStatusBarIcon(isListening: musicMonitor.isListening)
    }

    @objc private func statusItemClicked(_ sender: AnyObject?) {
        let isRightClick = NSApp.currentEvent?.type == .rightMouseUp
        let menu = isRightClick ? buildSystemMenu() : buildCustomizationMenu()
        statusItem.menu = menu
        statusItem.button?.performClick(nil)
        statusItem.menu = nil
    }

    private func updateStatusBarIcon(isListening: Bool) {
        if lastIconListeningState == isListening, statusItem.button?.image != nil {
            return
        }
        lastIconListeningState = isListening

        let width: CGFloat = isListening ? 27 : 20
        let height: CGFloat = 18
        let img = NSImage(size: NSSize(width: width, height: height), flipped: true) { _ in
            guard let ctx = NSGraphicsContext.current?.cgContext else { return false }
            ctx.setFillColor(NSColor.black.cgColor)

            // Smooth, curvy cat face silhouette (one continuous rounded Bezier path)
            let face = CGMutablePath()
            // Top crown center
            face.move(to: CGPoint(x: 9.5, y: 5.2))
            // Right crown -> rounded right ear -> right temple
            face.addQuadCurve(to: CGPoint(x: 13.0, y: 3.4), control: CGPoint(x: 11.4, y: 5.2))
            face.addQuadCurve(to: CGPoint(x: 15.2, y: 2.2), control: CGPoint(x: 14.3, y: 1.9))
            face.addQuadCurve(to: CGPoint(x: 16.0, y: 6.2), control: CGPoint(x: 16.1, y: 3.2))
            // Right cheek -> rounded chin
            face.addCurve(
                to: CGPoint(x: 9.5, y: 16.2),
                control1: CGPoint(x: 18.0, y: 11.4),
                control2: CGPoint(x: 14.6, y: 16.2)
            )
            // Left cheek -> left temple
            face.addCurve(
                to: CGPoint(x: 3.0, y: 6.2),
                control1: CGPoint(x: 4.4, y: 16.2),
                control2: CGPoint(x: 1.0, y: 11.4)
            )
            // Left rounded ear -> top crown center
            face.addQuadCurve(to: CGPoint(x: 3.8, y: 2.2), control: CGPoint(x: 2.9, y: 3.2))
            face.addQuadCurve(to: CGPoint(x: 6.0, y: 3.4), control: CGPoint(x: 4.7, y: 1.9))
            face.addQuadCurve(to: CGPoint(x: 9.5, y: 5.2), control: CGPoint(x: 7.6, y: 5.2))
            face.closeSubpath()

            ctx.addPath(face)
            ctx.fillPath()

            // Cutout cute cat face expression (works in both Light & Dark Mode)
            ctx.saveGState()
            ctx.setBlendMode(.clear)
            ctx.setLineCap(.round)
            ctx.setLineJoin(.round)

            if isListening {
                // Happy ^ ^ eyes when listening to music
                ctx.setStrokeColor(NSColor.black.cgColor)
                ctx.setLineWidth(1.35)
                ctx.move(to: CGPoint(x: 5.2, y: 10.2))
                ctx.addQuadCurve(to: CGPoint(x: 7.6, y: 10.2), control: CGPoint(x: 6.4, y: 8.5))
                ctx.move(to: CGPoint(x: 11.4, y: 10.2))
                ctx.addQuadCurve(to: CGPoint(x: 13.8, y: 10.2), control: CGPoint(x: 12.6, y: 8.5))
                ctx.strokePath()
            } else {
                // Cute round eyes
                ctx.fillEllipse(in: CGRect(x: 5.3, y: 8.7, width: 2.3, height: 2.6))
                ctx.fillEllipse(in: CGRect(x: 11.4, y: 8.7, width: 2.3, height: 2.6))
            }

            // Tiny soft oval nose + `ω` cat mouth
            ctx.fillEllipse(in: CGRect(x: 8.7, y: 11.1, width: 1.6, height: 1.1))
            ctx.setStrokeColor(NSColor.black.cgColor)
            ctx.setLineWidth(1.05)
            ctx.move(to: CGPoint(x: 7.4, y: 12.9))
            ctx.addQuadCurve(to: CGPoint(x: 9.5, y: 12.9), control: CGPoint(x: 8.45, y: 14.1))
            ctx.addQuadCurve(to: CGPoint(x: 11.6, y: 12.9), control: CGPoint(x: 10.55, y: 14.1))
            ctx.strokePath()
            ctx.restoreGState()

            // Music note badge when listening
            if isListening {
                let attrs: [NSAttributedString.Key: Any] = [
                    .font: NSFont.systemFont(ofSize: 11, weight: .bold),
                    .foregroundColor: NSColor.black,
                ]
                NSString(string: "♪").draw(at: CGPoint(x: 18.2, y: 2.0), withAttributes: attrs)
            }
            return true
        }
        img.isTemplate = true
        statusItem.button?.image = img
        statusItem.button?.toolTip = isListening
            ? "Cookie — Listening ♪ (Left-click: Customize • Right-click: Options)"
            : "Cookie (Left-click: Customize • Right-click: Options)"
    }

    private func buildCustomizationMenu() -> NSMenu {
        let menu = NSMenu()
        appendStatusHeader(to: menu)
        appendCustomizationItems(to: menu)
        return menu
    }

    private func buildSystemMenu() -> NSMenu {
        let menu = NSMenu()
        appendStatusHeader(to: menu)
        appendSystemItems(to: menu)
        return menu
    }

    private func buildMenu() -> NSMenu {
        let menu = NSMenu()
        appendStatusHeader(to: menu)
        appendCustomizationItems(to: menu)
        menu.addItem(.separator())
        appendSystemItems(to: menu)
        return menu
    }

    private func appendStatusHeader(to menu: NSMenu) {
        let statusText = musicMonitor.isListening ? "Cookie • Listening ♪" : "Cookie • Watching Mouse"
        let header = NSMenuItem(title: statusText, action: nil, keyEquivalent: "")
        header.isEnabled = false
        statusMenuItem = header
        menu.addItem(header)
        menu.addItem(.separator())
    }

    private func appendCustomizationItems(to menu: NSMenu) {
        // Music Mode submenu
        let musicSubmenu = NSMenu()
        for mode in MusicMode.allCases {
            let item = NSMenuItem(title: mode.displayName, action: #selector(selectMusicMode(_:)), keyEquivalent: "")
            item.target = self
            item.representedObject = mode.rawValue
            item.state = (musicMonitor.mode == mode) ? .on : .off
            musicSubmenu.addItem(item)
        }
        let musicItem = NSMenuItem(title: "Music & Headphones", action: nil, keyEquivalent: "")
        musicItem.submenu = musicSubmenu
        menu.addItem(musicItem)

        // Skin submenu
        let skinSubmenu = NSMenu()
        for skin in CatSkinId.allCases {
            let item = NSMenuItem(title: skin.displayName, action: #selector(selectSkin(_:)), keyEquivalent: "")
            item.target = self
            item.representedObject = skin.rawValue
            item.state = (currentSkinId == skin) ? .on : .off
            skinSubmenu.addItem(item)
        }
        let skinItem = NSMenuItem(title: "Cat Skin", action: nil, keyEquivalent: "")
        skinItem.submenu = skinSubmenu
        menu.addItem(skinItem)

        // Eye Color submenu
        let eyeSubmenu = NSMenu()
        for eye in EyeColorTheme.allCases {
            let item = NSMenuItem(title: eye.displayName, action: #selector(selectEyeTheme(_:)), keyEquivalent: "")
            item.target = self
            item.representedObject = eye.rawValue
            item.state = (currentEyeTheme == eye) ? .on : .off
            eyeSubmenu.addItem(item)
        }
        let eyeItem = NSMenuItem(title: "Eye Color", action: nil, keyEquivalent: "")
        eyeItem.submenu = eyeSubmenu
        menu.addItem(eyeItem)

        // Size submenu (5 scales like KuroCat)
        let sizeSubmenu = NSMenu()
        let sizes: [(String, CGFloat)] = [
            ("Tiny (0.6x)", 0.6),
            ("Small (0.8x)", 0.8),
            ("Normal (1.0x)", 1.0),
            ("Large (1.25x)", 1.25),
            ("XL (1.5x)", 1.5),
        ]
        for (label, val) in sizes {
            let item = NSMenuItem(title: label, action: #selector(selectScale(_:)), keyEquivalent: "")
            item.target = self
            item.representedObject = NSNumber(value: Double(val))
            item.state = abs(currentScale - val) < 0.05 ? .on : .off
            sizeSubmenu.addItem(item)
        }
        let sizeItem = NSMenuItem(title: "Size", action: nil, keyEquivalent: "")
        sizeItem.submenu = sizeSubmenu
        menu.addItem(sizeItem)

        menu.addItem(.separator())

        // Rim / Border toggle for dark backgrounds
        let borderItem = NSMenuItem(
            title: "Rim Border (Visible on Dark BG)",
            action: #selector(toggleBorder),
            keyEquivalent: ""
        )
        borderItem.target = self
        borderItem.state = showBorder ? .on : .off
        menu.addItem(borderItem)
    }

    private func appendSystemItems(to menu: NSMenu) {
        // High FPS toggle
        let fpsItem = NSMenuItem(
            title: "High Frame Rate (60fps)",
            action: #selector(toggleHighFPS),
            keyEquivalent: ""
        )
        fpsItem.target = self
        fpsItem.state = highFrameRate ? .on : .off
        menu.addItem(fpsItem)

        // Click-through toggle
        let clickItem = NSMenuItem(
            title: "Click-Through (Hold ⌥ to Drag)",
            action: #selector(toggleClickThrough),
            keyEquivalent: ""
        )
        clickItem.target = self
        clickItem.state = isClickThrough ? .on : .off
        menu.addItem(clickItem)

        let resetItem = NSMenuItem(title: "Reset Position", action: #selector(resetPosition), keyEquivalent: "")
        resetItem.target = self
        menu.addItem(resetItem)

        menu.addItem(.separator())

        let quitItem = NSMenuItem(title: "Quit Cookie", action: #selector(quitApp), keyEquivalent: "q")
        quitItem.target = self
        menu.addItem(quitItem)
    }

    @objc private func selectMusicMode(_ sender: NSMenuItem) {
        guard let raw = sender.representedObject as? String,
              let mode = MusicMode(rawValue: raw) else { return }
        musicMonitor.mode = mode
        UserDefaults.standard.set(mode.rawValue, forKey: PrefKey.musicMode)
        refreshMenusAndIcon()
    }

    @objc private func selectSkin(_ sender: NSMenuItem) {
        guard let raw = sender.representedObject as? String,
              let skin = CatSkinId(rawValue: raw) else { return }
        currentSkinId = skin
        refreshMenusAndIcon()
    }

    @objc private func selectEyeTheme(_ sender: NSMenuItem) {
        guard let raw = sender.representedObject as? String,
              let eye = EyeColorTheme(rawValue: raw) else { return }
        currentEyeTheme = eye
        refreshMenusAndIcon()
    }

    @objc private func toggleBorder() {
        showBorder = !showBorder
        refreshMenusAndIcon()
    }

    @objc private func toggleHighFPS() {
        highFrameRate = !highFrameRate
        refreshMenusAndIcon()
    }

    @objc private func selectScale(_ sender: NSMenuItem) {
        guard let num = sender.representedObject as? NSNumber else { return }
        currentScale = CGFloat(num.doubleValue)
        refreshMenusAndIcon()
    }

    @objc private func toggleClickThrough() {
        isClickThrough = !isClickThrough
        refreshMenusAndIcon()
    }

    @objc private func resetPosition() {
        UserDefaults.standard.set(false, forKey: PrefKey.hasPos)
        let origin = initialWindowOrigin(for: overlayWindow.frame.size)
        overlayWindow.setFrameOrigin(origin)
    }

    @objc private func quitApp() {
        NSApp.terminate(nil)
    }

    private func refreshMenusAndIcon() {
        updateStatusBarIcon(isListening: musicMonitor.isListening)
    }

    private func startAnimationLoop(interval: TimeInterval) {
        timer?.invalidate()
        lastFrameInterval = interval
        let t = Timer(timeInterval: interval, repeats: true) { [weak self] _ in
            self?.tick()
        }
        RunLoop.main.add(t, forMode: .common)
        timer = t
    }

    private func tick() {
        let now = ProcessInfo.processInfo.systemUptime
        musicMonitor.update(now: now)

        let listening = musicMonitor.isListening
        if lastIconListeningState != listening {
            refreshMenusAndIcon()
        }

        // Click-through everywhere outside the actual cat head/body (hold Option to override click-through)
        let optionHeld = NSEvent.modifierFlags.contains(.option)
        let cursor = NSEvent.mouseLocation
        let winFrame = overlayWindow.frame
        let scale = max(currentScale, 0.1)
        let localX = (cursor.x - winFrame.minX) / scale
        // CatView is flipped (origin top-left), AppKit screen origin is bottom-left
        let localY = (winFrame.maxY - cursor.y) / scale
        let onCatBody = CatRenderer.hitTest(localPoint: CGPoint(x: localX, y: localY), physics: physics)

        let shouldIgnoreMouse = (isClickThrough && !optionHeld) || (!onCatBody && !physics.isDragging && !optionHeld)
        overlayWindow.ignoresMouseEvents = shouldIgnoreMouse

        physics.update(
            now: now,
            catWindowFrame: winFrame,
            cursorGlobalPos: cursor,
            isListeningMusic: listening
        )

        overlayWindow.catView.needsDisplay = true

        // Adaptive FPS (30fps active, 10fps sleepy idle)
        let desiredInterval = physics.targetFrameInterval
        if abs(desiredInterval - lastFrameInterval) > 0.01 {
            startAnimationLoop(interval: desiredInterval)
        }
    }
}
