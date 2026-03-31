@preconcurrency import AVFAudio
import FluidAudio
import os

public actor HybridRecognizer {

    private static let logger = Logger(subsystem: "ModalDictation", category: "HybridRecognizer")

    private let streamingTranscriber: any StreamingTranscriber
    private let batchTranscriber: any BatchTranscriber
    private let converter: AudioConverter

    private var sampleBuffer: [Float] = []
    private var processingTask: Task<Void, Never>?
    private var eouTask: Task<Void, Never>?
    private var resultContinuation: AsyncStream<ASRResult>.Continuation?

    public init(
        streamingTranscriber: any StreamingTranscriber,
        batchTranscriber: any BatchTranscriber
    ) {
        self.streamingTranscriber = streamingTranscriber
        self.batchTranscriber = batchTranscriber
        self.converter = AudioConverter()
    }

    // Two-task pipeline: processingTask accumulates samples and feeds the streaming transcriber.
    // On EOU events, eouTask triggers batch transcription of accumulated audio.
    // Graceful shutdown: processingTask finishes the EOU stream, waits for eouTask to drain,
    // then flushes remaining samples. stop() cancels both tasks and flushes independently.
    public func start(audio: AsyncStream<AVAudioPCMBuffer>) async -> AsyncStream<ASRResult> {
        let (stream, continuation) = AsyncStream.makeStream(of: ASRResult.self)
        resultContinuation = continuation

        let (eouEvents, eouContinuation) = AsyncStream.makeStream(of: Void.self)

        // EOU text is discarded — batch transcriber re-processes accumulated audio for higher accuracy.
        await streamingTranscriber.setEouCallback { _ in
            eouContinuation.yield()
        }

        eouTask = Task {
            for await _ in eouEvents {
                guard !Task.isCancelled else { break }
                await self.transcribeAccumulated()
            }
        }

        processingTask = Task {
            for await buffer in audio {
                guard !Task.isCancelled else { break }
                await self.processBuffer(buffer)
            }
            guard !Task.isCancelled else { return }
            eouContinuation.finish()
            await self.eouTask?.value
            _ = try? await self.streamingTranscriber.finish()
            await self.transcribeAccumulated()
            self.resultContinuation?.finish()
            self.resultContinuation = nil
        }

        return stream
    }

    private func processBuffer(_ buffer: AVAudioPCMBuffer) async {
        guard let samples = try? converter.resampleBuffer(buffer) else { return }
        sampleBuffer.append(contentsOf: samples)
        _ = try? await streamingTranscriber.process(audioBuffer: buffer)
    }

    private static let minimumSamples = 16_000 // 1 second at 16 kHz

    private func transcribeAccumulated() async {
        let samples = sampleBuffer
        sampleBuffer.removeAll(keepingCapacity: true)

        guard samples.count >= Self.minimumSamples else { return }

        do {
            let result = try await batchTranscriber.transcribe(samples)
            resultContinuation?.yield(result)
        } catch {
            Self.logger.error("Batch transcription failed: \(error)")
        }
    }

    public func stop() async {
        processingTask?.cancel()
        eouTask?.cancel()
        await processingTask?.value
        await eouTask?.value
        processingTask = nil
        eouTask = nil

        guard resultContinuation != nil else { return }
        _ = try? await streamingTranscriber.finish()
        await transcribeAccumulated()
        resultContinuation?.finish()
        resultContinuation = nil
    }
}
