//! 菜单栏/托盘水滴图标：水滴内蓝色填充高度 = 今日进度。
//! 用 tiny-skia 在运行时按进度渲染 PNG，深浅色模式下描边颜色自适应。

use tiny_skia::*;

const SIZE: u32 = 64;

/// 深浅色外观（缓存检测系统设置，避免每秒都起子进程）
#[derive(Clone, Copy, PartialEq)]
pub enum Appearance {
    Dark,
    Light,
}

pub fn detect_appearance() -> Appearance {
    // macOS: `defaults read -g AppleInterfaceStyle` 输出 Dark 则为深色模式；
    // Windows 暂按浅色处理（蓝底描边在两种任务栏上都可读）。
    #[cfg(target_os = "macos")]
    {
        let out = std::process::Command::new("defaults")
            .args(["read", "-g", "AppleInterfaceStyle"])
            .output();
        if let Ok(o) = out {
            let s = String::from_utf8_lossy(&o.stdout);
            if s.contains("Dark") {
                return Appearance::Dark;
            }
        }
        Appearance::Light
    }
    #[cfg(not(target_os = "macos"))]
    Appearance::Light
}

/// 水滴轮廓（按设计稿 16x18 坐标手工贝塞尔）
fn droplet_path() -> Path {
    let k: f32 = 5.6 * 0.5523;
    let mut pb = PathBuilder::new();
    pb.move_to(8.0, 1.0); // 顶部尖端
    pb.cubic_to(8.0, 1.0, 2.4, 8.2, 2.4, 12.1); // 左侧曲线到圆最左
    pb.cubic_to(2.4, 12.1 + k, 8.0 - k, 17.7, 8.0, 17.7); // 左下弧
    pb.cubic_to(8.0 + k, 17.7, 13.6, 12.1 + k, 13.6, 12.1); // 右下弧
    pb.cubic_to(13.6, 8.2, 8.0, 1.0, 8.0, 1.0); // 右侧曲线回尖端
    pb.close();
    pb.finish().unwrap()
}

/// 渲染水滴图标 PNG。`progress` ∈ [0,1]
pub fn render_png(progress: f32, appearance: Appearance) -> Option<Vec<u8>> {
    let progress = progress.clamp(0.0, 1.0);
    let mut pixmap = Pixmap::new(SIZE, SIZE)?;

    // 设计坐标 16x18 → 64px：水滴宽约 48，水平居中，垂直留边
    let scale = 3.0_f32;
    let tx = (SIZE as f32 - 16.0 * scale) / 2.0;
    let ty = (SIZE as f32 - 18.0 * scale) / 2.0;
    let ts = Transform::from_scale(scale, scale).post_translate(tx, ty);
    let path = droplet_path().transform(ts)?;

    // 1) 填充：裁剪到水滴内部，从底部向上按进度填充蓝色渐变
    let mut mask = Mask::new(SIZE, SIZE)?;
    mask.fill_path(&path, FillRule::Winding, true, Transform::identity());
    let bounds = path.bounds();
    let level_y = bounds.top() + bounds.height() * (1.0 - progress);
    let mut fill_pb = PathBuilder::new();
    fill_pb.push_rect(Rect::from_xywh(0.0, level_y, SIZE as f32, SIZE as f32 - level_y)?);
    let fill_path = fill_pb.finish()?;
    let gradient = LinearGradient::new(
        Point::from_xy(0.0, bounds.top()),
        Point::from_xy(0.0, bounds.bottom()),
        vec![
            GradientStop::new(0.0, Color::from_rgba8(0x5e, 0xc2, 0xff, 255)),
            GradientStop::new(1.0, Color::from_rgba8(0x15, 0x73, 0xe6, 255)),
        ],
        SpreadMode::Pad,
        Transform::identity(),
    )?;
    let fill_paint = Paint {
        shader: gradient,
        anti_alias: true,
        ..Paint::default()
    };
    pixmap.fill_path(
        &fill_path,
        &fill_paint,
        FillRule::Winding,
        Transform::identity(),
        Some(&mask),
    );

    // 2) 描边：深色模式白色、浅色模式深灰
    let stroke_color = match appearance {
        Appearance::Dark => Color::from_rgba8(245, 245, 247, 242),
        Appearance::Light => Color::from_rgba8(28, 28, 32, 225),
    };
    let stroke_paint = Paint {
        shader: Shader::SolidColor(stroke_color),
        anti_alias: true,
        ..Paint::default()
    };
    pixmap.stroke_path(
        &path,
        &stroke_paint,
        &Stroke {
            width: scale * 1.2,
            ..Stroke::default()
        },
        Transform::identity(),
        None,
    );

    pixmap.encode_png().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 导出各进度的图标 PNG，供人工/视觉核对渲染效果
    #[test]
    fn render_icons_for_inspection() {
        for (name, p, a) in [
            ("dark_0", 0.0_f32, Appearance::Dark),
            ("dark_50", 0.5, Appearance::Dark),
            ("dark_100", 1.0, Appearance::Dark),
            ("light_0", 0.0, Appearance::Light),
            ("light_65", 0.65, Appearance::Light),
        ] {
            let png = render_png(p, a).expect("render");
            std::fs::write(format!("/tmp/droplet_{name}.png"), png).unwrap();
        }
    }
}
