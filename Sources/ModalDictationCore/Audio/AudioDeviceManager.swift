import Foundation
import CoreAudio

public enum AudioDeviceError: Error {
    case coreAudioError(OSStatus)
    case deviceNotFound(String)
}

public struct AudioDevice: Sendable, Equatable, Identifiable {
    public let audioDeviceID: AudioDeviceID
    public let uid: String
    public let name: String
    public let sampleRate: Float64

    public var id: String { uid }
}

public final class AudioDeviceManager: Sendable {

    public init() {}

    public func listInputDevices() throws -> [AudioDevice] {
        let allDeviceIDs = try getAllDeviceIDs()
        return allDeviceIDs.compactMap { deviceID in
            guard hasInputStreams(deviceID) else { return nil }
            guard let uid = try? getStringProperty(deviceID, selector: kAudioDevicePropertyDeviceUID),
                  let name = try? getStringProperty(deviceID, selector: kAudioDevicePropertyDeviceNameCFString)
            else { return nil }
            let sampleRate = (try? getSampleRate(deviceID)) ?? 0
            return AudioDevice(audioDeviceID: deviceID, uid: uid, name: name, sampleRate: sampleRate)
        }
    }

    public func defaultInputDevice() throws -> AudioDevice {
        var address = AudioObjectPropertyAddress(
            mSelector: kAudioHardwarePropertyDefaultInputDevice,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain
        )
        var deviceID: AudioDeviceID = 0
        var size = UInt32(MemoryLayout<AudioDeviceID>.size)
        let status = AudioObjectGetPropertyData(
            AudioObjectID(kAudioObjectSystemObject), &address, 0, nil, &size, &deviceID
        )
        guard status == noErr else { throw AudioDeviceError.coreAudioError(status) }

        guard let uid = try? getStringProperty(deviceID, selector: kAudioDevicePropertyDeviceUID),
              let name = try? getStringProperty(deviceID, selector: kAudioDevicePropertyDeviceNameCFString)
        else { throw AudioDeviceError.deviceNotFound("default") }
        let sampleRate = (try? getSampleRate(deviceID)) ?? 0
        return AudioDevice(audioDeviceID: deviceID, uid: uid, name: name, sampleRate: sampleRate)
    }

    public func device(forUID uid: String) throws -> AudioDevice? {
        try listInputDevices().first { $0.uid == uid }
    }

    public func resolveDevice(preferredUID: String?) throws -> AudioDevice {
        if let uid = preferredUID, let device = try device(forUID: uid) {
            return device
        }
        return try defaultInputDevice()
    }

    private func getAllDeviceIDs() throws -> [AudioDeviceID] {
        var address = AudioObjectPropertyAddress(
            mSelector: kAudioHardwarePropertyDevices,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain
        )
        var size: UInt32 = 0
        var status = AudioObjectGetPropertyDataSize(
            AudioObjectID(kAudioObjectSystemObject), &address, 0, nil, &size
        )
        guard status == noErr else { throw AudioDeviceError.coreAudioError(status) }

        let count = Int(size) / MemoryLayout<AudioDeviceID>.size
        var deviceIDs = [AudioDeviceID](repeating: 0, count: count)
        status = AudioObjectGetPropertyData(
            AudioObjectID(kAudioObjectSystemObject), &address, 0, nil, &size, &deviceIDs
        )
        guard status == noErr else { throw AudioDeviceError.coreAudioError(status) }
        return deviceIDs
    }

    private func hasInputStreams(_ deviceID: AudioDeviceID) -> Bool {
        var address = AudioObjectPropertyAddress(
            mSelector: kAudioDevicePropertyStreamConfiguration,
            mScope: kAudioDevicePropertyScopeInput,
            mElement: kAudioObjectPropertyElementMain
        )
        var size: UInt32 = 0
        let status = AudioObjectGetPropertyDataSize(deviceID, &address, 0, nil, &size)
        guard status == noErr, size > 0 else { return false }

        let bufferListPointer = UnsafeMutablePointer<AudioBufferList>.allocate(
            capacity: Int(size) / MemoryLayout<AudioBufferList>.stride
        )
        defer { bufferListPointer.deallocate() }

        var mutableSize = size
        let dataStatus = AudioObjectGetPropertyData(
            deviceID, &address, 0, nil, &mutableSize, bufferListPointer
        )
        guard dataStatus == noErr else { return false }

        let bufferList = UnsafeMutableAudioBufferListPointer(bufferListPointer)
        return bufferList.contains { $0.mNumberChannels > 0 }
    }

    private func getStringProperty(
        _ deviceID: AudioDeviceID, selector: AudioObjectPropertySelector
    ) throws -> String {
        var address = AudioObjectPropertyAddress(
            mSelector: selector,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain
        )
        var size = UInt32(MemoryLayout<Unmanaged<CFString>>.size)
        let buffer = UnsafeMutableRawPointer.allocate(
            byteCount: Int(size), alignment: MemoryLayout<Unmanaged<CFString>>.alignment
        )
        defer { buffer.deallocate() }
        let status = AudioObjectGetPropertyData(deviceID, &address, 0, nil, &size, buffer)
        guard status == noErr else { throw AudioDeviceError.coreAudioError(status) }
        let cfString = buffer.assumingMemoryBound(to: Unmanaged<CFString>.self).pointee
        return cfString.takeRetainedValue() as String
    }

    private func getSampleRate(_ deviceID: AudioDeviceID) throws -> Float64 {
        var address = AudioObjectPropertyAddress(
            mSelector: kAudioDevicePropertyNominalSampleRate,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain
        )
        var sampleRate: Float64 = 0
        var size = UInt32(MemoryLayout<Float64>.size)
        let status = AudioObjectGetPropertyData(deviceID, &address, 0, nil, &size, &sampleRate)
        guard status == noErr else { throw AudioDeviceError.coreAudioError(status) }
        return sampleRate
    }
}
