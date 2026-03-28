@preconcurrency import AVFAudio
import FluidAudio

public actor DictationRecognizer {

    private let transcriber: any BatchTranscriber
    private let converter: AudioConverter
    private let silenceTimeout: Double
    private let silenceThresholdRMS: Float

    private var sampleBuffer: [Float] = []
    private var lastSpeechTime: Date = .now
    private var processingTask: Task<Void, Never>?

    private var resultContinuation: AsyncStream<ASRResult>.Continuation?

    public init(
        transcriber: any BatchTranscriber,
        silenceTimeout: Double = 0.3,
        silenceThresholdRMS: Float = 0.01
    ) {
        self.transcriber = transcriber
        self.converter = AudioConverter()
        self.silenceTimeout = silenceTimeout
        self.silenceThresholdRMS = silenceThresholdRMS
    }

    public func start(audio: AsyncStream<AVAudioPCMBuffer>) -> AsyncStream<ASRResult> {
        let (stream, continuation) = AsyncStream.makeStream(of: ASRResult.self)
        resultContinuation = continuation

        processingTask = Task {
            for await buffer in audio {
                await self.processBuffer(buffer)
            }
            guard !Task.isCancelled else { return }
            await self.transcribeAccumulated()
            self.resultContinuation?.finish()
        }

        return stream
    }

    public func stop() async {
        processingTask?.cancel()
        processingTask = nil
        await transcribeAccumulated()
        resultContinuation?.finish()
        resultContinuation = nil
    }

    private func processBuffer(_ buffer: AVAudioPCMBuffer) async {
        let energy = rms(of: buffer)

        do {
            let samples = try converter.resampleBuffer(buffer)
            sampleBuffer.append(contentsOf: samples)
        } catch {
            return
        }

        if energy >= silenceThresholdRMS {
            lastSpeechTime = .now
            return
        }

        let silenceDuration = Date.now.timeIntervalSince(lastSpeechTime)
        if silenceDuration >= silenceTimeout && !sampleBuffer.isEmpty {
            await transcribeAccumulated()
        }
    }

    private static let minimumSamples = 16_000 // 1 second at 16kHz

    private func transcribeAccumulated() async {
        let samples = sampleBuffer
        sampleBuffer.removeAll(keepingCapacity: true)

        guard samples.count >= Self.minimumSamples else { return }

        do {
            let result = try await transcriber.transcribe(samples)
            resultContinuation?.yield(result)
        } catch {
            return
        }
    }

    private func rms(of buffer: AVAudioPCMBuffer) -> Float {
        guard let channelData = buffer.floatChannelData else { return 0 }
        let count = Int(buffer.frameLength)
        guard count > 0 else { return 0 }

        let samples = channelData[0]
        var sumOfSquares: Float = 0
        for i in 0..<count {
            sumOfSquares += samples[i] * samples[i]
        }
        return sqrtf(sumOfSquares / Float(count))
    }
}
