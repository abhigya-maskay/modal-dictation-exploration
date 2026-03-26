import Foundation
@preconcurrency import AVFAudio
import CoreAudio

public actor AudioCaptureEngine {
    private let engine = AVAudioEngine()
    private let deviceManager = AudioDeviceManager()
    private var bufferContinuation: AsyncStream<AVAudioPCMBuffer>.Continuation?
    private var currentDeviceUID: String?
    private var configObserver: (any NSObjectProtocol)?

    public private(set) var bufferStream: AsyncStream<AVAudioPCMBuffer>?

    public init() {}

    public func start(deviceUID: String? = nil) throws {
        let device = try deviceManager.resolveDevice(preferredUID: deviceUID)

        var audioDeviceID = device.audioDeviceID
        let inputNode = engine.inputNode
        guard let audioUnit = inputNode.audioUnit else {
            throw AudioDeviceError.deviceNotFound(device.uid)
        }
        let status = AudioUnitSetProperty(
            audioUnit,
            kAudioOutputUnitProperty_CurrentDevice,
            kAudioUnitScope_Global,
            0,
            &audioDeviceID,
            UInt32(MemoryLayout<AudioDeviceID>.size)
        )
        guard status == noErr else { throw AudioDeviceError.coreAudioError(status) }

        var continuation: AsyncStream<AVAudioPCMBuffer>.Continuation!
        let stream = AsyncStream<AVAudioPCMBuffer> { continuation = $0 }
        bufferContinuation = continuation
        bufferStream = stream

        let format = inputNode.outputFormat(forBus: 0)
        inputNode.installTap(onBus: 0, bufferSize: 1024, format: format) { buffer, _ in
            continuation.yield(buffer)
        }

        try engine.start()

        currentDeviceUID = device.uid
        configObserver = NotificationCenter.default.addObserver(
            forName: .AVAudioEngineConfigurationChange,
            object: engine,
            queue: nil
        ) { [weak self] _ in
            guard let self else { return }
            Task { await self.handleConfigurationChange() }
        }
    }

    public func stop() {
        if let configObserver {
            NotificationCenter.default.removeObserver(configObserver)
            self.configObserver = nil
        }
        engine.stop()
        engine.inputNode.removeTap(onBus: 0)
        bufferContinuation?.finish()
        bufferContinuation = nil
        bufferStream = nil
        currentDeviceUID = nil
    }

    private func handleConfigurationChange() {
        let uid = currentDeviceUID
        stop()
        do {
            try start(deviceUID: uid)
        } catch {
            print("Audio engine restart failed: \(error)")
        }
    }

    public func switchDevice(uid: String) throws {
        stop()
        try start(deviceUID: uid)
    }
}
