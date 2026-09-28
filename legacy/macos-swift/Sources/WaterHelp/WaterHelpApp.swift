import SwiftUI

@main
struct WaterHelpApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var appDelegate
    @ObservedObject private var model = WaterModel.shared

    var body: some Scene {
        MenuBarExtra {
            PanelView()
        } label: {
            DropletIcon(progress: model.progress)
        }
        .menuBarExtraStyle(.window)
    }
}

final class AppDelegate: NSObject, NSApplicationDelegate {
    func applicationDidFinishLaunching(_ notification: Notification) {
        // 纯菜单栏应用：不占 Dock 图标
        NSApp.setActivationPolicy(.accessory)
        Notifier.registerAndRequestAuthorization()
    }
}
