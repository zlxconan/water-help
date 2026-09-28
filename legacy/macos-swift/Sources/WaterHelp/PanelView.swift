import SwiftUI
import AppKit

/// 点击菜单栏图标弹出的主面板（对应设计稿）。
struct PanelView: View {
    @ObservedObject private var model = WaterModel.shared

    private static let timeFormatter: DateFormatter = {
        let fmt = DateFormatter()
        fmt.dateFormat = "HH:mm"
        return fmt
    }()

    var body: some View {
        VStack(spacing: 0) {
            header

            ProgressRingView(progress: model.progress, intakeMl: model.intakeMl, goalMl: model.dailyGoalMl)
                .padding(.top, 12)
                .padding(.bottom, 14)

            quickAddRow

            Divider()
                .padding(.vertical, 10)

            settingRows

            Divider()
                .padding(.vertical, 10)

            HStack {
                Text("登录时自动启动")
                Spacer()
                Toggle("", isOn: $model.launchAtLogin)
                    .labelsHidden()
                    .toggleStyle(.switch)
                    .controlSize(.mini)
            }

            footer
        }
        .padding(16)
        .frame(width: 320)
        .onAppear {
            // 打开面板时同步一次登录项状态（用户可能在系统设置里改过）
            model.refreshLoginItemState()
        }
    }

    // MARK: - 头部

    private var header: some View {
        HStack {
            Label("喝水助手", systemImage: "drop.fill")
                .font(.system(size: 14, weight: .semibold))
                .foregroundStyle(.primary)
            Spacer()
            Button {
                model.togglePauseOneHour()
            } label: {
                Text(model.isPaused ? "▶ 已暂停，点击恢复" : "⏸ 暂停 1 小时")
                    .font(.system(size: 12))
            }
            .buttonStyle(.bordered)
            .controlSize(.small)
        }
    }

    // MARK: - 进度圆环

    private struct ProgressRingView: View {
        let progress: Double
        let intakeMl: Int
        let goalMl: Int

        private let ringGradient = AngularGradient(
            colors: [Color(red: 0.37, green: 0.76, blue: 1.0), Color(red: 0.08, green: 0.45, blue: 0.90)],
            center: .center
        )

        var body: some View {
            ZStack {
                Circle()
                    .stroke(Color.primary.opacity(0.10), style: StrokeStyle(lineWidth: 10))
                Circle()
                    .trim(from: 0, to: progress)
                    .stroke(ringGradient, style: StrokeStyle(lineWidth: 10, lineCap: .round))
                    .rotationEffect(.degrees(-90))
                    .animation(.easeInOut(duration: 0.3), value: progress)
                VStack(spacing: 2) {
                    Text("\(Int(progress * 100))%")
                        .font(.system(size: 26, weight: .bold))
                        .monospacedDigit()
                    Text("\(intakeMl) / \(goalMl) ml")
                        .font(.system(size: 11))
                        .foregroundStyle(.secondary)
                        .monospacedDigit()
                }
            }
            .frame(width: 124, height: 124)
        }
    }

    // MARK: - 一键打卡

    private var quickAddRow: some View {
        HStack(spacing: 8) {
            ForEach([100, 250, 500], id: \.self) { ml in
                Button {
                    model.addMl(ml)
                } label: {
                    Text("+\(ml)ml")
                        .font(.system(size: 13, weight: .semibold))
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.bordered)
            }
        }
    }

    // MARK: - 设置行

    private var settingRows: some View {
        VStack(spacing: 10) {
            HStack {
                Text("下次提醒")
                    .foregroundStyle(.primary.opacity(0.8))
                Spacer()
                nextReminderText
            }

            HStack {
                Text("提醒间隔")
                    .foregroundStyle(.primary.opacity(0.8))
                Spacer()
                Picker("", selection: $model.intervalMinutes) {
                    Text("30分").tag(30)
                    Text("45分").tag(45)
                    Text("60分").tag(60)
                }
                .pickerStyle(.segmented)
                .labelsHidden()
                .frame(width: 160)
            }

            HStack {
                Text("每日目标")
                    .foregroundStyle(.primary.opacity(0.8))
                Spacer()
                HStack(spacing: 10) {
                    Button {
                        model.adjustGoal(by: -100)
                    } label: {
                        Image(systemName: "minus.circle")
                    }
                    .buttonStyle(.borderless)
                    Text("\(model.dailyGoalMl) ml")
                        .monospacedDigit()
                        .frame(width: 62)
                    Button {
                        model.adjustGoal(by: 100)
                    } label: {
                        Image(systemName: "plus.circle")
                    }
                    .buttonStyle(.borderless)
                }
            }
        }
        .font(.system(size: 13))
    }

    private var nextReminderText: some View {
        Group {
            if model.isPaused, let until = model.pausedUntil {
                Text("已暂停 · \(Self.timeFormatter.string(from: until)) 恢复")
            } else {
                let time = Self.timeFormatter.string(from: model.nextReminder)
                let mins = model.minutesUntilNext
                Text(mins > 0 ? "\(time) · \(mins) 分钟后" : "\(time) · 马上")
            }
        }
        .foregroundStyle(.secondary)
        .monospacedDigit()
    }

    // MARK: - 底部

    private var footer: some View {
        HStack {
            Text("今日已打卡 \(model.drinkCount) 次")
                .font(.system(size: 12))
                .foregroundStyle(.tertiary)
            Spacer()
            Button("退出喝水助手") {
                NSApp.terminate(nil)
            }
            .buttonStyle(.plain)
            .font(.system(size: 12))
            .foregroundStyle(.secondary)
        }
        .padding(.top, 10)
    }
}
