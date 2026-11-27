import AppKit
import AudioToolbox
import CoreAudio
import Foundation
import os

enum MusicMode: String, CaseIterable {
    case auto = "auto"
    case alwaysOn = "alwaysOn"
    case off = "off"

    var displayName: String {
        switch self {
        case .auto: return "Auto (System Audio)"
        case .alwaysOn: return "Always Listening"
        case .off: return "Off"
        }
    }
}

/// Detects active system audio output using CoreAudio's public
/// `kAudioDevicePropertyDeviceIsRunningSomewhere` property on the default output device.
/// Works across Spotify, Apple Music, browsers (YouTube, SoundCloud), VLC, etc.
final class MusicMonitor {
    var mode: MusicMode = .auto
    var beatSyncEnabled: Bool = false
    private(set) var isAudioPlaying: Bool = false
    private(set) var nowPlayingTitle: String?
    private(set) var nowPlayingArtist: String?

    var nowPlayingDisplay: String? {
        guard let title = nowPlayingTitle?.trimmingCharacters(in: .whitespacesAndNewlines), !title.isEmpty else {
            return nil
        }
        if let artist = nowPlayingArtist?.trimmingCharacters(in: .whitespacesAndNewlines), !artist.isEmpty {
            return "\(title) — \(artist)"
        }
        return title
    }

    private var lastPollTime: TimeInterval = 0
    private let pollInterval: TimeInterval = 0.4
    private var lastTrackPollTime: TimeInterval = 0
    private let trackPollInterval: TimeInterval = 4.0
    private let trackQueue = DispatchQueue(label: "com.cookie.nowplaying", qos: .utility)
    private var isQueryingTrack = false
    private var deniedBundleIDs = Set<String>()
    private let beatSync = BeatSyncEngine()

    var beatEnergy: CGFloat { beatSync.energy }
    var beatPulse: CGFloat { beatSync.pulse }
    var beatRateRadPerSec: CGFloat { beatSync.beatRateRadPerSec }

    init() {
        let center = DistributedNotificationCenter.default()
        let names: [NSNotification.Name] = [
            NSNotification.Name("com.spotify.client.PlaybackStateChanged"),
            NSNotification.Name("com.apple.Music.playerInfo"),
            NSNotification.Name("com.apple.iTunes.playerInfo"),
        ]
        for name in names {
            center.addObserver(self, selector: #selector(handlePlayerNotification(_:)), name: name, object: nil)
        }
    }

    deinit {
        DistributedNotificationCenter.default().removeObserver(self)
        beatSync.stop()
    }

    @objc private func handlePlayerNotification(_ notification: Notification) {
        guard let info = notification.userInfo else { return }
        let state = (info["Player State"] as? String ?? "").lowercased()
        if state == "playing" {
            let title = info["Name"] as? String
            let artist = info["Artist"] as? String
            DispatchQueue.main.async { [weak self] in
                self?.nowPlayingTitle = title
                self?.nowPlayingArtist = artist
            }
        } else if state == "paused" || state == "stopped" {
            DispatchQueue.main.async { [weak self] in
                self?.nowPlayingTitle = nil
                self?.nowPlayingArtist = nil
            }
        }
    }

    var isListening: Bool {
        switch mode {
        case .auto:
            return isAudioPlaying
        case .alwaysOn:
            return true
        case .off:
            return false
        }
    }

    func update(now: TimeInterval) {
        if mode == .auto {
            if now - lastPollTime >= pollInterval {
                lastPollTime = now
                let running = Self.queryDefaultOutputDeviceRunning()
                if isAudioPlaying && !running {
                    nowPlayingTitle = nil
                    nowPlayingArtist = nil
                }
                isAudioPlaying = running
            }
        } else {
            isAudioPlaying = (mode == .alwaysOn)
            if mode == .off {
                nowPlayingTitle = nil
                nowPlayingArtist = nil
            }
        }

        if isListening && now - lastTrackPollTime >= trackPollInterval && !isQueryingTrack {
            lastTrackPollTime = now
            queryRunningPlayerTrack()
        }

        let shouldTapAudio = beatSyncEnabled && (mode == .auto && isAudioPlaying)
        beatSync.update(active: shouldTapAudio, now: now)
    }

    private func queryRunningPlayerTrack() {
        isQueryingTrack = true
        let runningBundleIDs = Set(NSWorkspace.shared.runningApplications.compactMap { $0.bundleIdentifier })
        let denied = deniedBundleIDs

        trackQueue.async { [weak self] in
            var foundTitle: String?
            var foundArtist: String?
            var newlyDenied: [String] = []

            let dedicatedPlayers: [(bundleID: String, appName: String)] = [
                ("com.spotify.client", "Spotify"),
                ("com.apple.Music", "Music"),
            ]
            for player in dedicatedPlayers where runningBundleIDs.contains(player.bundleID) && !denied.contains(player.bundleID) {
                let (result, wasDenied) = Self.runTrackAppleScript(appName: player.appName)
                if wasDenied { newlyDenied.append(player.bundleID) }
                if let result {
                    foundTitle = result.0
                    foundArtist = result.1
                    break
                }
            }

            if foundTitle == nil, runningBundleIDs.contains("com.apple.QuickTimePlayerX"), !denied.contains("com.apple.QuickTimePlayerX") {
                let (title, wasDenied) = Self.runQuickTimeScript()
                if wasDenied { newlyDenied.append("com.apple.QuickTimePlayerX") }
                if let title { foundTitle = title }
            }

            if foundTitle == nil {
                let chromiumBrowsers: [(bundleID: String, appName: String)] = [
                    ("com.brave.Browser", "Brave Browser"),
                    ("com.google.Chrome", "Google Chrome"),
                    ("company.thebrowser.Browser", "Arc"),
                    ("com.microsoft.edgemac", "Microsoft Edge"),
                    ("com.vivaldi.Vivaldi", "Vivaldi"),
                ]
                for browser in chromiumBrowsers where runningBundleIDs.contains(browser.bundleID) && !denied.contains(browser.bundleID) {
                    let (title, wasDenied) = Self.runChromiumTrackScript(appName: browser.appName)
                    if wasDenied { newlyDenied.append(browser.bundleID) }
                    if let title {
                        foundTitle = title
                        break
                    }
                }
            }

            if foundTitle == nil, runningBundleIDs.contains("com.apple.Safari"), !denied.contains("com.apple.Safari") {
                let (title, wasDenied) = Self.runSafariTrackScript()
                if wasDenied { newlyDenied.append("com.apple.Safari") }
                if let title { foundTitle = title }
            }

            DispatchQueue.main.async {
                guard let self else { return }
                self.isQueryingTrack = false
                for id in newlyDenied { self.deniedBundleIDs.insert(id) }
                if let title = foundTitle, !title.isEmpty {
                    self.nowPlayingTitle = title
                    self.nowPlayingArtist = foundArtist
                } else if self.nowPlayingTitle != nil && !self.isListening {
                    self.nowPlayingTitle = nil
                    self.nowPlayingArtist = nil
                }
            }
        }
    }

    private static func runTrackAppleScript(appName: String) -> ((String, String?)?, Bool) {
        let source = """
        tell application "\(appName)"
            if player state is playing then
                return (name of current track) & "|||" & (artist of current track)
            end if
        end tell
        return ""
        """
        var error: NSDictionary?
        guard let script = NSAppleScript(source: source) else { return (nil, false) }
        let output = script.executeAndReturnError(&error).stringValue ?? ""
        if let errNum = error?[NSAppleScript.errorNumber] as? Int, errNum == -1743 {
            return (nil, true)
        }
        guard error == nil, !output.isEmpty else { return (nil, false) }
        let parts = output.components(separatedBy: "|||")
        let title = parts.first ?? ""
        let artist = parts.count > 1 ? parts[1] : nil
        return ((title, artist), false)
    }

    private static func runQuickTimeScript() -> (String?, Bool) {
        let source = """
        tell application "QuickTime Player"
            if (count of documents) > 0 then
                if playing of document 1 is true then
                    return name of document 1
                end if
            end if
        end tell
        return ""
        """
        var error: NSDictionary?
        guard let script = NSAppleScript(source: source) else { return (nil, false) }
        let output = script.executeAndReturnError(&error).stringValue?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        if let errNum = error?[NSAppleScript.errorNumber] as? Int, errNum == -1743 {
            return (nil, true)
        }
        guard error == nil, !output.isEmpty else { return (nil, false) }
        return (output, false)
    }

    private static func runChromiumTrackScript(appName: String) -> (String?, Bool) {
        let source = """
        tell application "\(appName)"
            if (count of windows) > 0 then
                repeat with w in windows
                    repeat with t in tabs of w
                        set u to URL of t
                        if u contains "youtube.com/watch" or u contains "music.youtube.com" or u contains "open.spotify.com" or u contains "soundcloud.com" or u contains "music.apple.com" then
                            return (title of t) & "|||" & u
                        end if
                    end repeat
                end repeat
            end if
        end tell
        return ""
        """
        var error: NSDictionary?
        guard let script = NSAppleScript(source: source) else { return (nil, false) }
        let output = script.executeAndReturnError(&error).stringValue ?? ""
        if let errNum = error?[NSAppleScript.errorNumber] as? Int, errNum == -1743 {
            return (nil, true)
        }
        guard error == nil, !output.isEmpty else { return (nil, false) }
        let rawTitle = output.components(separatedBy: "|||").first ?? ""
        return (cleanBrowserMediaTitle(rawTitle), false)
    }

    private static func runSafariTrackScript() -> (String?, Bool) {
        let source = """
        tell application "Safari"
            if (count of windows) > 0 then
                repeat with w in windows
                    repeat with t in tabs of w
                        set u to URL of t
                        if u contains "youtube.com/watch" or u contains "music.youtube.com" or u contains "open.spotify.com" or u contains "soundcloud.com" or u contains "music.apple.com" then
                            return (name of t) & "|||" & u
                        end if
                    end repeat
                end repeat
            end if
        end tell
        return ""
        """
        var error: NSDictionary?
        guard let script = NSAppleScript(source: source) else { return (nil, false) }
        let output = script.executeAndReturnError(&error).stringValue ?? ""
        if let errNum = error?[NSAppleScript.errorNumber] as? Int, errNum == -1743 {
            return (nil, true)
        }
        guard error == nil, !output.isEmpty else { return (nil, false) }
        let rawTitle = output.components(separatedBy: "|||").first ?? ""
        return (cleanBrowserMediaTitle(rawTitle), false)
    }

    private static func cleanBrowserMediaTitle(_ raw: String) -> String? {
        var title = raw.trimmingCharacters(in: .whitespacesAndNewlines)
        let suffixes = [
            " - YouTube Music",
            " - YouTube",
            " | Spotify",
            " - SoundCloud",
            " - Apple Music",
        ]
        for suffix in suffixes where title.hasSuffix(suffix) {
            title = String(title.dropLast(suffix.count)).trimmingCharacters(in: .whitespacesAndNewlines)
        }
        let genericTitles: Set<String> = ["YouTube", "YouTube Music", "Spotify", "Spotify – Web Player", "SoundCloud", "Apple Music", ""]
        return genericTitles.contains(title) ? nil : title
    }

    private static func queryDefaultOutputDeviceRunning() -> Bool {
        var deviceID = AudioDeviceID(0)
        var size = UInt32(MemoryLayout<AudioDeviceID>.size)
        var address = AudioObjectPropertyAddress(
            mSelector: kAudioHardwarePropertyDefaultOutputDevice,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain
        )

        let err1 = AudioObjectGetPropertyData(
            AudioObjectID(kAudioObjectSystemObject),
            &address,
            0,
            nil,
            &size,
            &deviceID
        )
        guard err1 == noErr, deviceID != kAudioObjectUnknown else {
            return false
        }

        var isRunning: UInt32 = 0
        var runSize = UInt32(MemoryLayout<UInt32>.size)
        var runAddress = AudioObjectPropertyAddress(
            mSelector: kAudioDevicePropertyDeviceIsRunningSomewhere,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain
        )

        let err2 = AudioObjectGetPropertyData(
            deviceID,
            &runAddress,
            0,
            nil,
            &runSize,
            &isRunning
        )
        guard err2 == noErr else {
            return false
        }
        return isRunning != 0
    }
}

/// Real-time system audio amplitude & onset beat detector using CoreAudio Process Taps (macOS 14.2+).
/// Gracefully falls back to zero energy (default steady vibe) if unavailable or unpermitted.
private final class BeatSyncEngine {
    private var lock = os_unfair_lock()
    private var tapID = AudioObjectID(kAudioObjectUnknown)
    private var aggregateDeviceID = AudioObjectID(kAudioObjectUnknown)
    private var ioProcID: AudioDeviceIOProcID?
    private var isRunning = false
    private var didAttemptStart = false

    private var lpSample: Float = 0
    private var avgBassEnergy: Float = 0.01
    private var rawEnergy: Float = 0
    private var rawPulse: Float = 0
    private var lastBeatTime: TimeInterval = 0
    private var smoothedBPM: Double = 118.0
    private var lastAudioSignalTime: TimeInterval = 0

    private(set) var energy: CGFloat = 0
    private(set) var pulse: CGFloat = 0
    private(set) var beatRateRadPerSec: CGFloat = 6.2

    private var lastUpdateTime: TimeInterval = 0

    func update(active: Bool, now: TimeInterval) {
        if active {
            if !isRunning && !didAttemptStart {
                start()
            }
        } else if isRunning || didAttemptStart {
            stop()
            didAttemptStart = false
        }

        let dt = lastUpdateTime > 0 ? max(0.001, min(now - lastUpdateTime, 0.1)) : 0.033
        lastUpdateTime = now

        os_unfair_lock_lock(&lock)
        let e = CGFloat(rawEnergy)
        let p = CGFloat(rawPulse)
        rawPulse *= Float(exp(-dt * 10.0))
        let hasSignal = (now - lastAudioSignalTime) < 1.2
        let bpm = smoothedBPM
        os_unfair_lock_unlock(&lock)

        let decay = CGFloat(exp(-dt * 9.0))
        energy = max(e, energy * decay)
        pulse = max(p, pulse * CGFloat(exp(-dt * 11.0)))
        if hasSignal {
            beatRateRadPerSec = CGFloat((bpm / 60.0) * 2.0 * .pi)
        } else {
            beatRateRadPerSec = 6.2
        }
    }

    private func start() {
        didAttemptStart = true
        guard #available(macOS 14.2, *) else { return }

        let tapDesc = CATapDescription(stereoGlobalTapButExcludeProcesses: [])
        tapDesc.muteBehavior = .unmuted
        tapDesc.isPrivate = true
        tapDesc.name = "CookieBeatTap"

        var newTapID = AudioObjectID(kAudioObjectUnknown)
        let tapErr = AudioHardwareCreateProcessTap(tapDesc, &newTapID)
        guard tapErr == noErr, newTapID != kAudioObjectUnknown else { return }

        let tapUID = tapDesc.uuid.uuidString
        let aggDesc: [String: Any] = [
            kAudioAggregateDeviceNameKey: "CookieBeatAggregate",
            kAudioAggregateDeviceUIDKey: UUID().uuidString,
            kAudioAggregateDeviceIsPrivateKey: true,
            kAudioAggregateDeviceTapAutoStartKey: true,
            kAudioAggregateDeviceTapListKey: [
                [
                    kAudioSubTapUIDKey: tapUID,
                    kAudioSubTapDriftCompensationKey: true,
                ]
            ],
        ]

        var newAggID = AudioObjectID(kAudioObjectUnknown)
        let aggErr = AudioHardwareCreateAggregateDevice(aggDesc as CFDictionary, &newAggID)
        guard aggErr == noErr, newAggID != kAudioObjectUnknown else {
            AudioHardwareDestroyProcessTap(newTapID)
            return
        }

        var procID: AudioDeviceIOProcID?
        let selfPtr = Unmanaged.passUnretained(self).toOpaque()
        let procErr = AudioDeviceCreateIOProcIDWithBlock(&procID, newAggID, nil) { _, inInputData, _, _, _ in
            let engine = Unmanaged<BeatSyncEngine>.fromOpaque(selfPtr).takeUnretainedValue()
            engine.processInput(inInputData)
        }
        guard procErr == noErr, let procID else {
            AudioHardwareDestroyAggregateDevice(newAggID)
            AudioHardwareDestroyProcessTap(newTapID)
            return
        }

        let startErr = AudioDeviceStart(newAggID, procID)
        guard startErr == noErr else {
            AudioDeviceDestroyIOProcID(newAggID, procID)
            AudioHardwareDestroyAggregateDevice(newAggID)
            AudioHardwareDestroyProcessTap(newTapID)
            return
        }

        tapID = newTapID
        aggregateDeviceID = newAggID
        ioProcID = procID
        isRunning = true
    }

    func stop() {
        if let procID = ioProcID, aggregateDeviceID != kAudioObjectUnknown {
            AudioDeviceStop(aggregateDeviceID, procID)
            AudioDeviceDestroyIOProcID(aggregateDeviceID, procID)
        }
        ioProcID = nil
        if aggregateDeviceID != kAudioObjectUnknown {
            AudioHardwareDestroyAggregateDevice(aggregateDeviceID)
            aggregateDeviceID = AudioObjectID(kAudioObjectUnknown)
        }
        if tapID != kAudioObjectUnknown {
            if #available(macOS 14.2, *) {
                AudioHardwareDestroyProcessTap(tapID)
            }
            tapID = AudioObjectID(kAudioObjectUnknown)
        }
        isRunning = false

        os_unfair_lock_lock(&lock)
        rawEnergy = 0
        rawPulse = 0
        lastAudioSignalTime = 0
        os_unfair_lock_unlock(&lock)
        energy = 0
        pulse = 0
        beatRateRadPerSec = 6.2
    }

    private func processInput(_ bufferListPtr: UnsafePointer<AudioBufferList>) {
        let bufferList = UnsafeMutableAudioBufferListPointer(UnsafeMutablePointer(mutating: bufferListPtr))
        var sumSq: Float = 0
        var bassSumSq: Float = 0
        var count: Int = 0
        var lp = lpSample

        for buffer in bufferList {
            guard let data = buffer.mData, buffer.mDataByteSize >= 4 else { continue }
            let sampleCount = Int(buffer.mDataByteSize) / MemoryLayout<Float>.size
            let samples = data.bindMemory(to: Float.self, capacity: sampleCount)
            let stride = max(1, Int(buffer.mNumberChannels))
            var i = 0
            while i < sampleCount {
                let s = samples[i]
                sumSq += s * s
                lp += 0.08 * (s - lp)
                bassSumSq += lp * lp
                count += 1
                i += stride
            }
        }
        lpSample = lp
        guard count > 0 else { return }

        let rms = sqrt(sumSq / Float(count))
        let bassRms = sqrt(bassSumSq / Float(count))
        let scaledEnergy = min(1.0, rms * 5.0)
        let now = ProcessInfo.processInfo.systemUptime

        os_unfair_lock_lock(&lock)
        rawEnergy = scaledEnergy
        if rms > 0.002 {
            lastAudioSignalTime = now
        }
        avgBassEnergy = avgBassEnergy * 0.94 + bassRms * 0.06
        if bassRms > max(0.014, avgBassEnergy * 1.42) && (now - lastBeatTime) > 0.24 {
            rawPulse = 1.0
            let interval = now - lastBeatTime
            if interval >= 0.35 && interval <= 1.0 {
                let instantBPM = 60.0 / interval
                smoothedBPM = smoothedBPM * 0.78 + instantBPM * 0.22
            }
            lastBeatTime = now
        }
        os_unfair_lock_unlock(&lock)
    }
}
