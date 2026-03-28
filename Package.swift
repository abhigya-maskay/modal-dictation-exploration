// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "ModalDictation",
    platforms: [
        .macOS(.v14),
    ],
    dependencies: [
        .package(url: "https://github.com/FluidInference/FluidAudio", from: "0.12.0"),
        .package(url: "https://github.com/LebJe/TOMLKit", from: "0.6.0"),
    ],
    targets: [
        .target(
            name: "ModalDictationCore",
            dependencies: [
                .product(name: "FluidAudio", package: "FluidAudio"),
                .product(name: "TOMLKit", package: "TOMLKit"),
            ],
            path: "Sources/ModalDictationCore",
            resources: [
                .process("Resources/default-config.toml"),
            ]
        ),
        .executableTarget(
            name: "ModalDictation",
            dependencies: [
                "ModalDictationCore",
            ],
            path: "Sources/ModalDictation"
        ),
        .testTarget(
            name: "ModalDictationTests",
            dependencies: ["ModalDictationCore"],
            path: "Tests/ModalDictationTests"
        ),
    ]
)
