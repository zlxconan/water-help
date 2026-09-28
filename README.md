# 💧 WaterHelp 喝水助手

一个跨平台（macOS / Windows）的喝水提醒小工具：托盘/菜单栏常驻水滴图标，蓝色填充实时显示今日饮水进度；到点弹出系统通知，点一下「喝了 +250ml」即可打卡。

基于 **Tauri 2**（Rust + 原生 Web 前端），一套代码双平台，无框架依赖、无构建链，成品只有几 MB。

![License](https://img.shields.io/badge/license-MIT-green)

## ✨ 功能

- **水滴进度图标**：托盘/菜单栏图标的水滴内，蓝色填充高度 = 今日已喝 ÷ 目标，不点开也能一眼看到，深浅色模式自动适配描边
- **系统通知一键打卡**（macOS）：通知自带「喝了 +250ml」主按钮和「稍后提醒」按钮，点了就记录并重置倒计时，不打断工作
- **面板**：点击图标展开——进度圆环、+100/+250/+500ml 快捷打卡、下次提醒时间、间隔 30/45/60 分钟、每日目标 1000–5000ml
- **暂停**：一键暂停 1 小时，到点自动恢复
- **每日零点自动重置**，数据只存本机（JSON 文件），不上传任何信息
- **开机自启**（可选）

## 📦 安装

### 从 Release 下载（推荐）

到 [Releases](../../releases) 下载：

| 文件 | 平台 | 安装 |
|---|---|---|
| `WaterHelp-macOS-universal.zip` | macOS 13+（Intel + Apple Silicon 通用） | 解压后拖入「应用程序」 |
| `WaterHelp-Windows-x64.zip` | Windows 10/11 x64 | 解压即用（免安装，依赖系统自带 WebView2） |

### 从源码构建

```bash
# macOS（通用二进制）
bash scripts/build-macos.sh       # 产出 dist/WaterHelp.app

# Windows
cd src-tauri && cargo build --release   # 产出 target/release/waterhelp.exe

# 或用 GitHub Actions 云端构建：推送代码后 Actions 页面自动产出双平台安装包
```

前端是纯静态 HTML/CSS/JS（`ui/` 目录），已内嵌进二进制，无需 Node.js。

## 🗂 项目结构

```
ui/                 面板前端（纯 HTML/CSS/JS，无构建步骤）
src-tauri/
  src/main.rs       托盘图标、面板窗口、定时引擎、Tauri 命令
  src/state.rs      业务状态：打卡/暂停/跨天重置/JSON 持久化
  src/droplet.rs    tiny-skia 运行时渲染水滴进度图标
  src/notify.rs     平台通知（macOS NSUserNotification / Windows Toast）
scripts/            macOS 打包脚本（本地与 CI 共用）
.github/workflows/  CI：云端构建 macOS 通用版 + Windows 版
design/             UI 设计稿
legacy/macos-swift/ 早期的纯 Swift 原生版本（仅 macOS，保留作参考）
```

## ⚙️ 数据位置

- macOS: `~/Library/Application Support/com.waterhelp.desktop/state.json`
- Windows: `%APPDATA%\com.waterhelp.desktop\state.json`

清除所有数据：删除该文件即可。

## 🧪 开发

```bash
cd src-tauri
cargo run                                   # 调试运行
WATERHELP_INTERVAL_SECS=8 cargo run         # 用 8 秒间隔快速验证提醒链路（仅 debug 构建）
```

## 🗺 Roadmap

- [ ] Windows 通知支持操作按钮（Toast XML / AppUserModelID 注册）
- [ ] Windows 深浅色任务栏图标自适应
- [ ] 今日/本周统计图表
- [ ] 多语言（英文）

## 📄 License

[MIT](./LICENSE)
