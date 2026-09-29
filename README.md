# 💧 WaterHelp 喝水助手

**[中文](./README.md) | [English](./README.en.md)**

一个跨平台（macOS / Windows）的喝水提醒小工具：托盘/菜单栏常驻水滴图标，蓝色填充实时显示今日饮水进度；到点弹出系统通知，点一下「喝了 +250ml」即可打卡；打开主窗口可以查看历史记录和统计图表。

基于 **Tauri 2**（Rust + 原生 Web 前端），一套代码支持双平台，无 Node 构建链，成品只有十几 MB。

![macOS](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-blue) ![License](https://img.shields.io/badge/license-MIT-green) ![Tauri](https://img.shields.io/badge/Tauri-2.0-orange)

![主窗口截图](docs/main-window.png)

## ✨ 功能

- **水滴进度图标**：托盘/菜单栏图标的水滴内，蓝色填充高度 = 今日已喝 ÷ 目标，不点开也能一眼看到；深浅色模式自动切换描边颜色
- **系统通知一键打卡**（macOS）：提醒通知自带「喝了 +250ml」和「稍后提醒」按钮，点了就记录并重置倒计时，不打断工作
- **快捷面板**：点击托盘图标展开——进度圆环、+100/+250/+500ml 快捷打卡、下次提醒倒计时、间隔 30/45/60 分钟、每日目标 1000–5000ml、暂停 1 小时
- **主窗口**：正常应用窗口（Dock/启动台可见），今日概览、最近 14 天柱状图、历史记录列表（达标状态一目了然）；点关闭只是隐藏，随时从 Dock 秒开
- **每日零点自动重置**，历史记录按天存档（保留 90 天）
- **开机自启**（可选）：登录后静默常驻托盘，不弹窗口
- **数据只存本机**（JSON 文件），不上传任何信息

## 📦 安装

到 [Releases](https://github.com/zlxconan/water-help/releases) 下载最新版：

| 文件 | 平台 | 安装方式 |
|---|---|---|
| `WaterHelp-macOS-universal.zip` | macOS 10.15+（Intel + Apple Silicon 通用） | 解压后拖入「应用程序」 |
| `WaterHelp-Windows-x64.zip` | Windows 10/11 x64 | 解压即用（免安装，依赖系统自带 WebView2） |

> macOS 首次运行如提示无法验证开发者：右键 → 打开，或在 系统设置 → 隐私与安全性 里允许。
> macOS 通知提醒需要在 系统设置 → 通知 → 喝水助手 中打开「允许通知」。

## 🖥 界面一览

| 位置 | 说明 |
|---|---|
| 托盘/菜单栏图标 | 水滴填充 = 今日进度；左键点开快捷面板，右键打开菜单 |
| 快捷面板 | 打卡、改间隔/目标、暂停、开机自启、退出（失焦自动收起） |
| 主窗口 | 今日圆环 + 快捷打卡 + 14 天柱状图 + 历史列表 |
| 系统通知 | 定时提醒，macOS 上可直接点按钮打卡 |

UI 设计稿见 [docs/design.png](docs/design.png)。

## 🔨 从源码构建

```bash
# macOS（通用二进制 x86_64 + arm64）
bash scripts/build-macos.sh          # 产出 dist/WaterHelp.app

# Windows
cd src-tauri && cargo build --release    # 产出 target/release/waterhelp.exe

# 或直接用 GitHub Actions 云端构建：push 后 Actions 自动产出双平台安装包
```

前端是纯静态 HTML/CSS/JS（`ui/` 目录），构建时内嵌进二进制，**无需 Node.js**。

## 🗂 项目结构

```
ui/                 前端（纯 HTML/CSS/JS，无构建步骤）
  index.html        托盘快捷面板
  main.html         主窗口（历史统计）
src-tauri/
  src/main.rs       托盘图标、窗口管理、定时引擎、Tauri 命令
  src/state.rs      业务状态：打卡/暂停/跨天重置/历史存档/JSON 持久化
  src/droplet.rs    tiny-skia 运行时渲染水滴进度图标
  src/notify.rs     平台通知（macOS NSUserNotification / Windows Toast）
scripts/            macOS 打包脚本（本地与 CI 共用）
.github/workflows/  CI：云端构建 macOS 通用版 + Windows x64
docs/               截图与设计稿
legacy/macos-swift/ 早期的纯 Swift 原生版本（仅 macOS，保留作参考）
```

## ⚙️ 数据位置

- macOS：`~/Library/Application Support/com.waterhelp.desktop/state.json`
- Windows：`%APPDATA%\com.waterhelp.desktop\state.json`

清除所有数据：删除该文件即可。

## 🧪 开发调试

```bash
cd src-tauri
cargo run                                   # 调试运行
WATERHELP_INTERVAL_SECS=8 cargo run         # 用 8 秒间隔快速验证提醒链路（仅 debug 构建）
```

## 🗺 Roadmap

- [ ] Windows 通知支持操作按钮（Toast XML / AppUserModelID 注册）
- [ ] Windows 深浅色任务栏图标自适应
- [ ] 今日/本周统计图表增强（连续达标天数等）
- [ ] 多语言界面

## 🤝 贡献

欢迎 Issue 和 PR！提 PR 前请确保 `cargo build` 通过；UI 改动直接编辑 `ui/` 下的 HTML/CSS/JS 即可，无需任何构建工具。

## 📄 License

[MIT](./LICENSE) © WaterHelp Contributors
