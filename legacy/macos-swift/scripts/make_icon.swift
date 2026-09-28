// 生成应用图标：蓝色渐变圆角方块 + 白色水滴（对应设计稿）。
// 用法: swift scripts/make_icon.swift <输出.iconset目录>
import Foundation
import CoreGraphics
import ImageIO
import UniformTypeIdentifiers

let outputDir = CommandLine.arguments.count > 1
    ? CommandLine.arguments[1]
    : "dist/AppIcon.iconset"

try? FileManager.default.createDirectory(
    atPath: outputDir, withIntermediateDirectories: true
)

/// 水滴轮廓（按设计稿的 16x18 坐标手工贝塞尔）
func dropletPath() -> CGPath {
    let p = CGMutablePath()
    let t = CGPoint(x: 8, y: 1)          // 顶部尖端
    let l = CGPoint(x: 2.4, y: 12.1)     // 圆的最左
    let b = CGPoint(x: 8, y: 17.7)       // 圆的最底
    let r = CGPoint(x: 13.6, y: 12.1)    // 圆的最右
    let k: CGFloat = 5.6 * 0.5523        // 四分之一圆的贝塞尔近似系数

    p.move(to: t)
    p.addCurve(to: l, control1: CGPoint(x: 8, y: 1), control2: CGPoint(x: 2.4, y: 8.2))
    p.addCurve(to: b, control1: CGPoint(x: 2.4, y: 12.1 + k), control2: CGPoint(x: 8 - k, y: 17.7))
    p.addCurve(to: r, control1: CGPoint(x: 8 + k, y: 17.7), control2: CGPoint(x: 13.6, y: 12.1 + k))
    p.addCurve(to: t, control1: CGPoint(x: 13.6, y: 8.2), control2: CGPoint(x: 8, y: 1))
    p.closeSubpath()
    return p
}

func renderIcon(size: CGFloat) -> CGImage? {
    guard let ctx = CGContext(
        data: nil, width: Int(size), height: Int(size),
        bitsPerComponent: 8, bytesPerRow: 0,
        space: CGColorSpace(name: CGColorSpace.sRGB)!,
        bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
    ) else { return nil }

    // 统一切换为 y 向下的屏幕坐标系，方便按设计稿坐标绘制
    ctx.translateBy(x: 0, y: size)
    ctx.scaleBy(x: 1, y: -1)

    let rect = CGRect(x: 0, y: 0, width: size, height: size)
    let radius = size * 0.2237 // macOS 图标圆角比例

    // 背景圆角方块 + 对角渐变 #4FC3F7 → #1565D8
    let bgPath = CGPath(roundedRect: rect, cornerWidth: radius, cornerHeight: radius, transform: nil)
    ctx.addPath(bgPath)
    ctx.clip()
    let colors = [
        CGColor(srgbRed: 0.31, green: 0.76, blue: 0.97, alpha: 1),
        CGColor(srgbRed: 0.08, green: 0.40, blue: 0.85, alpha: 1),
    ] as CFArray
    if let gradient = CGGradient(colorsSpace: CGColorSpace(name: CGColorSpace.sRGB)!, colors: colors, locations: [0, 1]) {
        ctx.drawLinearGradient(gradient, start: .zero, end: CGPoint(x: size, y: size), options: [])
    }
    // 顶部轻微高光，增加质感
    if let highlight = CGGradient(colorsSpace: CGColorSpace(name: CGColorSpace.sRGB)!, colors: [
        CGColor(srgbRed: 1, green: 1, blue: 1, alpha: 0.30),
        CGColor(srgbRed: 1, green: 1, blue: 1, alpha: 0.0),
    ] as CFArray, locations: [0, 0.5]) {
        ctx.drawLinearGradient(highlight, start: .zero, end: CGPoint(x: 0, y: size * 0.55), options: [])
    }

    // 水滴：缩放到图标的 46% 宽，水平居中、垂直略偏上
    let dropWidth = size * 0.46
    let scale = dropWidth / 16.0
    let dropHeight = 18.0 * scale
    ctx.saveGState()
    ctx.translateBy(x: (size - dropWidth) / 2, y: (size - dropHeight) / 2 - size * 0.015)
    ctx.scaleBy(x: scale, y: scale)
    ctx.setShadow(offset: CGSize(width: 0, height: -1.2), blur: 2.4,
                  color: CGColor(srgbRed: 0, green: 0.1, blue: 0.3, alpha: 0.35))
    ctx.setFillColor(CGColor(srgbRed: 1, green: 1, blue: 1, alpha: 0.97))
    ctx.addPath(dropletPath())
    ctx.fillPath()
    ctx.restoreGState()

    return ctx.makeImage()
}

func savePNG(_ image: CGImage, named name: String) {
    let url = URL(fileURLWithPath: outputDir).appendingPathComponent(name)
    guard let dest = CGImageDestinationCreateWithURL(
        url as CFURL, UTType.png.identifier as CFString, 1, nil
    ) else { fatalError("无法创建 \(name)") }
    CGImageDestinationAddImage(dest, image, nil)
    guard CGImageDestinationFinalize(dest) else { fatalError("写入 \(name) 失败") }
}

let sizes: [(name: String, px: CGFloat)] = [
    ("icon_16x16.png", 16), ("icon_16x16@2x.png", 32),
    ("icon_32x32.png", 32), ("icon_32x32@2x.png", 64),
    ("icon_128x128.png", 128), ("icon_128x128@2x.png", 256),
    ("icon_256x256.png", 256), ("icon_256x256@2x.png", 512),
    ("icon_512x512.png", 512), ("icon_512x512@2x.png", 1024),
]

for (name, px) in sizes {
    guard let img = renderIcon(size: px) else { fatalError("渲染 \(name) 失败") }
    savePNG(img, named: name)
}
print("图标已生成到 \(outputDir)")
