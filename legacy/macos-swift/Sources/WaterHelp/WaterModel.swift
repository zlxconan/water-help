import Foundation
import ServiceManagement
import UserNotifications

/// 全局状态：今日饮水量、提醒设置、提醒引擎。
/// 数据仅存放在 UserDefaults（本机），按日期键记录，跨天自动清零。
@MainActor
final class WaterModel: ObservableObject {
    static let shared = WaterModel()

    private let defaults = UserDefaults.standard
    private var ticker: Timer?

    // MARK: - 今日数据

    @Published private(set) var intakeMl: Int
    @Published private(set) var drinkCount: Int
    @Published private(set) var lastDrinkDate: Date?
    private var dayKey: String

    // MARK: - 提醒设置

    @Published var intervalMinutes: Int {
        didSet {
            defaults.set(intervalMinutes, forKey: Keys.interval)
            if !isPaused { nextReminder = now().addingTimeInterval(TimeInterval(intervalMinutes) * 60) }
        }
    }
    @Published var dailyGoalMl: Int {
        didSet { defaults.set(dailyGoalMl, forKey: Keys.goal) }
    }
    @Published var launchAtLogin: Bool {
        didSet {
            defaults.set(launchAtLogin, forKey: Keys.login)
            updateLoginItem()
        }
    }

    /// 非空表示提醒已暂停到该时刻
    @Published var pausedUntil: Date?
    /// 下次发通知的时刻
    @Published private(set) var nextReminder: Date

    private enum Keys {
        static let interval = "intervalMinutes"
        static let goal = "dailyGoalMl"
        static let login = "launchAtLogin"
        static let pausedUntil = "pausedUntil"
        static let nextReminder = "nextReminder"
        static func intake(_ day: String) -> String { "intake_\(day)" }
        static func count(_ day: String) -> String { "count_\(day)" }
        static func lastDrink(_ day: String) -> String { "lastDrink_\(day)" }
    }

    // MARK: - 派生数据

    var progress: Double {
        dailyGoalMl > 0 ? min(1, Double(intakeMl) / Double(dailyGoalMl)) : 0
    }
    var goalReached: Bool { intakeMl >= dailyGoalMl }
    var isPaused: Bool {
        if let p = pausedUntil { return p > now() }
        return false
    }
    var minutesUntilNext: Int {
        max(0, Int(ceil(nextReminder.timeIntervalSince(now()) / 60)))
    }
    /// 距离上次喝水过了多少分钟（没记录过就用提醒间隔）
    var minutesSinceLastDrink: Int {
        guard let last = lastDrinkDate else { return intervalMinutes }
        return max(1, Int(now().timeIntervalSince(last) / 60))
    }

    private func now() -> Date { Date() }

    // MARK: - 初始化

    init() {
        let d = UserDefaults.standard
        let initialInterval = d.object(forKey: Keys.interval) as? Int ?? 45
        intervalMinutes = initialInterval
        dailyGoalMl = d.object(forKey: Keys.goal) as? Int ?? 2000
        launchAtLogin = d.bool(forKey: Keys.login)

        let today = Self.dayKey(for: Date())
        dayKey = today
        intakeMl = d.integer(forKey: Keys.intake(today))
        drinkCount = d.integer(forKey: Keys.count(today))
        lastDrinkDate = d.object(forKey: Keys.lastDrink(today)) as? Date

        let savedPause = d.object(forKey: Keys.pausedUntil) as? Date
        pausedUntil = (savedPause ?? .distantPast) > Date() ? savedPause : nil

        let savedNext = d.object(forKey: Keys.nextReminder) as? Date
        nextReminder = (savedNext ?? .distantPast) > Date()
            ? savedNext!
            : Date().addingTimeInterval(TimeInterval(initialInterval) * 60)

        ticker = Timer.scheduledTimer(withTimeInterval: 1, repeats: true) { [weak self] _ in
            Task { @MainActor in self?.tick() }
        }
    }

    // MARK: - 每秒心跳：跨天重置 / 暂停到点恢复 / 到点提醒

    private func tick() {
        let date = now()

        let today = Self.dayKey(for: date)
        if today != dayKey {
            dayKey = today
            intakeMl = 0
            drinkCount = 0
            lastDrinkDate = nil
            saveToday()
        }

        if let pauseEnd = pausedUntil, pauseEnd <= date {
            pausedUntil = nil
            defaults.removeObject(forKey: Keys.pausedUntil)
            nextReminder = date.addingTimeInterval(TimeInterval(intervalMinutes) * 60)
            saveNextReminder()
        }

        if !isPaused, nextReminder <= date {
            Notifier.remind(sinceMinutes: minutesSinceLastDrink, goalReached: goalReached, intakeMl: intakeMl)
            nextReminder = date.addingTimeInterval(TimeInterval(intervalMinutes) * 60)
            saveNextReminder()
        }
    }

    // MARK: - 用户动作

    func addMl(_ ml: Int) {
        intakeMl += ml
        drinkCount += 1
        lastDrinkDate = now()
        saveToday()
        // 喝水打卡后，从现在重新倒数
        nextReminder = now().addingTimeInterval(TimeInterval(intervalMinutes) * 60)
        saveNextReminder()
    }

    /// 通知里的「稍后提醒」：10 分钟后再提醒
    func snooze(minutes: Int = 10) {
        nextReminder = now().addingTimeInterval(TimeInterval(minutes) * 60)
        saveNextReminder()
    }

    func togglePauseOneHour() {
        if isPaused {
            pausedUntil = nil
            defaults.removeObject(forKey: Keys.pausedUntil)
            nextReminder = now().addingTimeInterval(TimeInterval(intervalMinutes) * 60)
        } else {
            pausedUntil = now().addingTimeInterval(3600)
            defaults.set(pausedUntil, forKey: Keys.pausedUntil)
        }
        saveNextReminder()
    }

    func adjustGoal(by delta: Int) {
        dailyGoalMl = min(5000, max(1000, dailyGoalMl + delta))
    }

    // MARK: - 开机自启

    private func updateLoginItem() {
        do {
            let service = SMAppService.mainApp
            if launchAtLogin, service.status != .enabled {
                try service.register()
            } else if !launchAtLogin, service.status == .enabled {
                try service.unregister()
            }
        } catch {
            NSLog("WaterHelp: 登录项设置失败 \(error.localizedDescription)")
        }
    }

    /// 面板打开时校准开关状态（用户可能直接在系统设置里增删过登录项）
    func refreshLoginItemState() {
        let enabled = SMAppService.mainApp.status == .enabled
        if enabled != launchAtLogin {
            launchAtLogin = enabled
        }
    }

    // MARK: - 持久化

    private func saveToday() {
        defaults.set(intakeMl, forKey: Keys.intake(dayKey))
        defaults.set(drinkCount, forKey: Keys.count(dayKey))
        if let last = lastDrinkDate {
            defaults.set(last, forKey: Keys.lastDrink(dayKey))
        } else {
            defaults.removeObject(forKey: Keys.lastDrink(dayKey))
        }
    }

    private func saveNextReminder() {
        defaults.set(nextReminder, forKey: Keys.nextReminder)
    }

    private static func dayKey(for date: Date) -> String {
        let fmt = DateFormatter()
        fmt.dateFormat = "yyyy-MM-dd"
        return fmt.string(from: date)
    }
}
