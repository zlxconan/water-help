import Foundation
import UserNotifications

/// 系统通知：注册分类（带两个动作按钮）、请求授权、发送提醒。
enum Notifier {
    static let categoryID = "DRINK_REMINDER"
    static let actionDrank = "ACTION_DRANK_250"
    static let actionSnooze = "ACTION_SNOOZE"

    static func registerAndRequestAuthorization() {
        let center = UNUserNotificationCenter.current()
        center.delegate = NotificationDelegate.shared

        let drank = UNNotificationAction(identifier: actionDrank, title: "喝了 +250ml")
        let snooze = UNNotificationAction(identifier: actionSnooze, title: "稍后提醒")
        let category = UNNotificationCategory(
            identifier: categoryID,
            actions: [drank, snooze],
            intentIdentifiers: [],
            options: []
        )
        center.setNotificationCategories([category])

        center.requestAuthorization(options: [.alert, .sound]) { granted, error in
            if let error {
                NSLog("WaterHelp: 通知授权失败 \(error.localizedDescription)")
            } else if !granted {
                NSLog("WaterHelp: 用户未允许通知")
            }
        }
    }

    static func remind(sinceMinutes: Int, goalReached: Bool, intakeMl: Int) {
        let content = UNMutableNotificationContent()
        content.title = goalReached ? "今日目标已达成 🎉" : "该喝水啦 💧"
        content.body = goalReached
            ? "已喝 \(intakeMl)ml，保持状态，再来一杯吧～"
            : "已经 \(sinceMinutes) 分钟没喝水了，起来接杯温水吧，建议 250ml。"
        content.sound = .default
        content.categoryIdentifier = categoryID

        let request = UNNotificationRequest(
            identifier: "water-reminder-\(UUID().uuidString)",
            content: content,
            trigger: nil // 立即送达，时机由 WaterModel 的定时器掌控
        )
        UNUserNotificationCenter.current().add(request) { error in
            if let error {
                NSLog("WaterHelp: 发送通知失败 \(error.localizedDescription)")
            }
        }
    }
}

/// 处理通知按钮回调。
final class NotificationDelegate: NSObject, UNUserNotificationCenterDelegate {
    static let shared = NotificationDelegate()

    /// 应用在前台时也要弹横幅
    func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        willPresent notification: UNNotification,
        withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions) -> Void
    ) {
        completionHandler([.banner, .list, .sound])
    }

    func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        didReceive response: UNNotificationResponse,
        withCompletionHandler completionHandler: @escaping () -> Void
    ) {
        DispatchQueue.main.async {
            switch response.actionIdentifier {
            case Notifier.actionDrank:
                WaterModel.shared.addMl(250)
            case Notifier.actionSnooze:
                WaterModel.shared.snooze()
            default:
                break // 点通知本体：不做额外处理
            }
        }
        completionHandler()
    }
}
