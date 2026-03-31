import CoreML
import FluidAudio

public enum ModelStoreError: Error {
    case modelsNotLoaded(String)
}

public actor ModelStore {

    public enum State: Sendable {
        case idle
        case loading
        case ready
        case failed(Error)
    }

    private struct ModelSlot<T> {
        var value: T?
        var state: State = .idle
    }

    private var tdt = ModelSlot<AsrModels>()
    private var eou = ModelSlot<StreamingEouAsrManager>()
    private var ctc = ModelSlot<CtcModels>()

    public var tdtState: State { tdt.state }
    public var eouState: State { eou.state }
    public var ctcState: State { ctc.state }

    private let chunkSize: StreamingChunkSize
    private let progressHandler: DownloadUtils.ProgressHandler?

    public init(
        chunkSize: StreamingChunkSize = .ms320,
        progressHandler: DownloadUtils.ProgressHandler? = nil
    ) {
        self.chunkSize = chunkSize
        self.progressHandler = progressHandler
    }

    public func loadAll() async throws {
        async let tdt = loadTDT()
        async let eou = loadEOU()
        async let ctc = loadCTC()
        _ = try await (tdt, eou, ctc)
    }

    @discardableResult
    public func loadTDT() async throws -> AsrModels {
        if let existing = tdt.value { return existing }
        tdt.state = .loading
        do {
            let value = try await AsrModels.downloadAndLoad(progressHandler: progressHandler)
            tdt = ModelSlot(value: value, state: .ready)
            return value
        } catch {
            tdt.state = .failed(error)
            throw error
        }
    }

    @discardableResult
    public func loadEOU() async throws -> StreamingEouAsrManager {
        if let existing = eou.value { return existing }
        eou.state = .loading
        do {
            let manager = StreamingEouAsrManager(chunkSize: chunkSize)
            try await manager.loadModelsFromHuggingFace(progressHandler: progressHandler)
            eou = ModelSlot(value: manager, state: .ready)
            return manager
        } catch {
            eou.state = .failed(error)
            throw error
        }
    }

    @discardableResult
    public func loadCTC() async throws -> CtcModels {
        if let existing = ctc.value { return existing }
        ctc.state = .loading
        do {
            let value = try await CtcModels.downloadAndLoad()
            ctc = ModelSlot(value: value, state: .ready)
            return value
        } catch {
            ctc.state = .failed(error)
            throw error
        }
    }

    public func getTDTModels() throws -> AsrModels {
        guard let models = tdt.value else {
            throw ModelStoreError.modelsNotLoaded("TDT models not loaded — call loadTDT() first")
        }
        return models
    }

    public func getEOUManager() throws -> any StreamingTranscriber {
        guard let manager = eou.value else {
            throw ModelStoreError.modelsNotLoaded("EOU models not loaded — call loadEOU() first")
        }
        return manager
    }

    public func getCTCModels() throws -> CtcModels {
        guard let models = ctc.value else {
            throw ModelStoreError.modelsNotLoaded("CTC models not loaded — call loadCTC() first")
        }
        return models
    }
}

extension ModelStore: ModelProviding {
    public func makeBatchTranscriber(
        commandsConfig: CommandsConfig? = nil
    ) async throws -> any BatchTranscriber {
        let models = try getTDTModels()
        let manager = AsrManager()
        try await manager.initialize(models: models)

        if let commandsConfig {
            let ctc = try getCTCModels()
            let vocabulary = VocabularyBuilder.build(from: commandsConfig)
            try await manager.configureVocabularyBoosting(
                vocabulary: vocabulary,
                ctcModels: ctc
            )
        }

        return manager
    }
}
