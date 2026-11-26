import AudioToolbox
import CoreAudio
import Foundation

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
    private(set) var isAudioPlaying: Bool = false

    private var lastPollTime: TimeInterval = 0
    private let pollInterval: TimeInterval = 0.4

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
        guard mode == .auto else {
            isAudioPlaying = (mode == .alwaysOn)
            return
        }
        if now - lastPollTime < pollInterval {
            return
        }
        lastPollTime = now
        isAudioPlaying = Self.queryDefaultOutputDeviceRunning()
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
