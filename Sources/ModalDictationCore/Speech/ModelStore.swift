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

    private func load<T>(
        _ slot: WritableKeyPath<ModelStore, ModelSlot<T>>,
        download: () async throws -> T
    ) async throws -> T {
        if let existing = self[keyPath: slot].value { return existing }

        self[keyPath: slot].state = .loading
        do {
            let value = try await download()
            self[keyPath: slot] = ModelSlot(value: value, state: .ready)
            return value
        } catch {
            self[keyPath: slot].state = .failed(error)
            throw error
        }
    }

    @discardableResult
    public func loadTDT() async throws -> AsrModels {
        try await load(\.tdt) {
            try await AsrModels.downloadAndLoad(progressHandler: progressHandler)
        }
    }

    @discardableResult
    public func loadEOU() async throws -> StreamingEouAsrManager {
        try await load(\.eou) {
            let manager = StreamingEouAsrManager(chunkSize: chunkSize)
            try await manager.loadModelsFromHuggingFace(progressHandler: progressHandler)
            return manager
        }
    }

    @discardableResult
    public func loadCTC() async throws -> CtcModels {
        try await load(\.ctc) {
            try await CtcModels.downloadAndLoad()
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
