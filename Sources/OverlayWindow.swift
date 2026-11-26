import AppKit
import CoreGraphics

/// Transparent, always-on-top, non-activating overlay panel hosting the 2.5D cat.
final class OverlayWindow: NSPanel {
    let catView: CatView

    init(
        contentRect: NSRect,
        physics: CatPhysics,
        renderer: CatRenderer,
        skinProvider: @escaping () -> SkinSpec,
        eyeProvider: @escaping () -> EyeColorTheme,
        borderProvider: @escaping () -> Bool,
        scaleProvider: @escaping () -> CGFloat,
        onPositionChanged: @escaping (NSPoint) -> Void,
        contextMenuProvider: @escaping () -> NSMenu
    ) {
        self.catView = CatView(
            frame: NSRect(origin: .zero, size: contentRect.size),
            physics: physics,
            renderer: renderer,
            skinProvider: skinProvider,
            eyeProvider: eyeProvider,
            borderProvider: borderProvider,
            scaleProvider: scaleProvider,
            onPositionChanged: onPositionChanged,
            contextMenuProvider: contextMenuProvider
        )

        super.init(
            contentRect: contentRect,
            styleMask: [.borderless, .nonactivatingPanel],
            backing: .buffered,
            defer: false
        )

        isOpaque = false
        backgroundColor = .clear
        hasShadow = false
        level = .floating
        isMovableByWindowBackground = false
        collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary, .ignoresCycle]
        hidesOnDeactivate = false
        ignoresMouseEvents = false

        contentView = catView
    }

    override var canBecomeKey: Bool { false }
    override var canBecomeMain: Bool { false }
}

final class CatView: NSView {
    private let physics: CatPhysics
    private let renderer: CatRenderer
    private let skinProvider: () -> SkinSpec
    private let eyeProvider: () -> EyeColorTheme
    private let borderProvider: () -> Bool
    private let scaleProvider: () -> CGFloat
    private let onPositionChanged: (NSPoint) -> Void
    private let contextMenuProvider: () -> NSMenu

    private var dragStartScreenLoc: NSPoint?
    private var lastDragScreenLoc: NSPoint?
    private var lastDragTime: TimeInterval = 0
    private var dragStartWindowOrigin: NSPoint?
    private var didDrag: Bool = false

    override var isFlipped: Bool { true }

    init(
        frame: NSRect,
        physics: CatPhysics,
        renderer: CatRenderer,
        skinProvider: @escaping () -> SkinSpec,
        eyeProvider: @escaping () -> EyeColorTheme,
        borderProvider: @escaping () -> Bool,
        scaleProvider: @escaping () -> CGFloat,
        onPositionChanged: @escaping (NSPoint) -> Void,
        contextMenuProvider: @escaping () -> NSMenu
    ) {
        self.physics = physics
        self.renderer = renderer
        self.skinProvider = skinProvider
        self.eyeProvider = eyeProvider
        self.borderProvider = borderProvider
        self.scaleProvider = scaleProvider
        self.onPositionChanged = onPositionChanged
        self.contextMenuProvider = contextMenuProvider
        super.init(frame: frame)
        wantsLayer = true
        layer?.backgroundColor = NSColor.clear.cgColor
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func draw(_ dirtyRect: NSRect) {
        guard let ctx = NSGraphicsContext.current?.cgContext else { return }
        ctx.clear(bounds)
        let scale = scaleProvider()
        ctx.saveGState()
        ctx.scaleBy(x: scale, y: scale)
        renderer.draw(
            in: ctx,
            skin: skinProvider(),
            physics: physics,
            eyeTheme: eyeProvider(),
            showBorder: borderProvider()
        )
        ctx.restoreGState()
    }

    override func mouseDown(with event: NSEvent) {
        let loc = convert(event.locationInWindow, from: nil)
        let scale = max(scaleProvider(), 0.1)
        let catPt = CGPoint(x: loc.x / scale, y: loc.y / scale)
        guard CatRenderer.hitTest(localPoint: catPt, physics: physics) else {
            dragStartScreenLoc = nil
            return
        }
        let now = ProcessInfo.processInfo.systemUptime
        dragStartScreenLoc = NSEvent.mouseLocation
        lastDragScreenLoc = dragStartScreenLoc
        lastDragTime = now
        dragStartWindowOrigin = window?.frame.origin
        didDrag = false
    }

    override func mouseDragged(with event: NSEvent) {
        guard let startScreen = dragStartScreenLoc,
              let startOrigin = dragStartWindowOrigin,
              let win = window else { return }

        let now = ProcessInfo.processInfo.systemUptime
        let current = NSEvent.mouseLocation
        let dx = current.x - startScreen.x
        let dy = current.y - startScreen.y
        if hypot(dx, dy) > 3.0 {
            didDrag = true
            physics.isDragging = true
        }
        if let prev = lastDragScreenLoc {
            let dt = max(now - lastDragTime, 0.008)
            physics.dragVelocity = CGPoint(
                x: (current.x - prev.x) / CGFloat(dt),
                y: (current.y - prev.y) / CGFloat(dt)
            )
        }
        lastDragScreenLoc = current
        lastDragTime = now

        let newOrigin = NSPoint(x: startOrigin.x + dx, y: startOrigin.y + dy)
        win.setFrameOrigin(newOrigin)
    }

    override func mouseUp(with event: NSEvent) {
        physics.isDragging = false
        guard dragStartScreenLoc != nil else { return }
        if didDrag, let origin = window?.frame.origin {
            onPositionChanged(origin)
        } else {
            // Single click = pet the cat!
            let loc = convert(event.locationInWindow, from: nil)
            let scale = max(scaleProvider(), 0.1)
            physics.triggerPet(atLocalPoint: CGPoint(x: loc.x / scale, y: loc.y / scale))
        }
        dragStartScreenLoc = nil
        lastDragScreenLoc = nil
        dragStartWindowOrigin = nil
        didDrag = false
    }

    override func rightMouseDown(with event: NSEvent) {
        let loc = convert(event.locationInWindow, from: nil)
        let scale = max(scaleProvider(), 0.1)
        let catPt = CGPoint(x: loc.x / scale, y: loc.y / scale)
        guard CatRenderer.hitTest(localPoint: catPt, physics: physics) else { return }
        let menu = contextMenuProvider()
        NSMenu.popUpContextMenu(menu, with: event, for: self)
    }
}
