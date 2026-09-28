// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "WaterHelp",
    platforms: [.macOS(.v13)],
    targets: [
        .executableTarget(
            name: "WaterHelp",
            path: "Sources/WaterHelp"
        )
    ]
)
