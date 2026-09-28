import SwiftUI

/// 菜单栏水滴图标：水滴内的蓝色填充高度 = 今日饮水进度。
struct DropletIcon: View {
    var progress: Double

    private static let fillTop = Color(red: 0.37, green: 0.76, blue: 1.0)    // #5EC2FF
    private static let fillBottom = Color(red: 0.08, green: 0.45, blue: 0.90) // #1573E6

    var body: some View {
        ZStack {
            Image(systemName: "drop.fill")
                .foregroundStyle(
                    LinearGradient(
                        colors: [Self.fillTop, Self.fillBottom],
                        startPoint: .top, endPoint: .bottom
                    )
                )
                .mask(alignment: .bottom) {
                    GeometryReader { geo in
                        Rectangle().frame(height: geo.size.height * progress)
                    }
                }
            Image(systemName: "drop")
                .foregroundStyle(.primary)
        }
        .font(.system(size: 15, weight: .medium))
        .frame(width: 20, height: 20)
        .help("喝水助手")
        .accessibilityLabel("喝水助手，今日进度 \(Int(progress * 100))%")
    }
}
