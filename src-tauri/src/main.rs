// 跨平台喝水提醒：托盘/菜单栏水滴图标 + 定时系统通知 + 面板打卡。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod droplet;
mod notify;
mod state;

use std::sync::Mutex;
use std::time::Duration;

use serde::Serialize;
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition,
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
};

/// 托盘图标的屏幕位置（点击时记录，用于把面板弹到图标旁边）
#[derive(Clone, Copy)]
struct TrayRect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}
static TRAY_RECT: Mutex<Option<TrayRect>> = Mutex::new(None);
/// 上次托盘左键点击时刻（防抖：macOS 可能同时派发按下/抬起两个事件）
static LAST_TRAY_CLICK: Mutex<Option<std::time::Instant>> = Mutex::new(None);
/// 面板最近一次显示时刻（刚显示的瞬间忽略失焦事件，避免被立即隐藏）
static PANEL_SHOWN_AT: Mutex<Option<std::time::Instant>> = Mutex::new(None);

struct Shared {
    model: Mutex<state::Model>,
    data_dir: std::path::PathBuf,
}

#[derive(Serialize)]
struct StateView {
    #[serde(flatten)]
    model: state::Model,
    now: i64,
    autostart: bool,
}

fn now_ts() -> i64 {
    chrono::Local::now().timestamp()
}

/// 修改模型后的统一收尾：持久化 + 刷新托盘图标 + 通知前端
fn after_change(app: &AppHandle, progress_changed: bool) {
    let shared = app.state::<Shared>();
    let mut model = shared.model.lock().unwrap();
    model.save(&shared.data_dir);
    let progress = model.progress();
    drop(model);

    if progress_changed {
        refresh_tray_icon(app, progress);
    }
    let _ = app.emit("state-changed", ());
}

fn refresh_tray_icon(app: &AppHandle, progress: f32) {
    let appearance = droplet::detect_appearance();
    if let Some(png) = droplet::render_png(progress, appearance) {
        if let Ok(img) = tauri::image::Image::from_bytes(&png) {
            if let Some(tray) = app.tray_by_id("main") {
                let _ = tray.set_icon(Some(img));
            }
        }
    }
}

/// 打开主窗口（Dock/启动台/托盘菜单入口），点关闭按钮只隐藏不销毁
fn show_main(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.unminimize();
        let _ = win.show();
        let _ = win.set_focus();
    }
}

fn toggle_panel(app: &AppHandle) {
    let Some(panel) = app.get_webview_window("panel") else {
        return;
    };
    if panel.is_visible().unwrap_or(false) {
        let _ = panel.hide();
        return;
    }
    if let Some(rect) = *TRAY_RECT.lock().unwrap() {
        let size = panel.outer_size().unwrap_or_default();
        let center_x = rect.x + rect.w / 2.0;
        let x = (center_x - size.width as f64 / 2.0).max(8.0);
        // 菜单栏在屏幕顶部(macOS)、任务栏在底部(Windows)的启发式定位
        let y = if rect.y < 200.0 {
            rect.y + rect.h + 8.0
        } else {
            (rect.y - size.height as f64 - 8.0).max(8.0)
        };
        let _ = panel.set_position(PhysicalPosition::new(x, y));
    }
    let _ = panel.show();
    let _ = panel.set_focus();
    *PANEL_SHOWN_AT.lock().unwrap() = Some(std::time::Instant::now());
}

/// 后台每秒心跳：跨天重置 / 暂停到点恢复 / 到点发通知
fn start_ticker(app: AppHandle) {
    std::thread::spawn(move || {
        let mut ticks: u64 = 0;
        loop {
            std::thread::sleep(Duration::from_secs(1));
            ticks += 1;

            let shared = app.state::<Shared>();
            let mut model = shared.model.lock().unwrap();
            let now = now_ts();
            match model.tick(now) {
                state::Tick::Due {
                    since_minutes,
                    goal_reached,
                } => {
                    let intake = model.intake_ml;
                    model.save(&shared.data_dir);
                    drop(model);
                    let handle = app.clone();
                    notify::remind(
                        intake,
                        notify::Reminder {
                            since_minutes,
                            goal_reached,
                        },
                        move |action| {
                            let shared = handle.state::<Shared>();
                            let mut model = shared.model.lock().unwrap();
                            let now = now_ts();
                            match action {
                                "drank" => model.add_ml(now, 250),
                                "snooze" => model.snooze(now, 10),
                                _ => {}
                            }
                            drop(model);
                            after_change(&handle, true);
                        },
                    );
                }
                state::Tick::Nothing => {
                    // 每分钟校准一次托盘图标（进度不变时用于跟随深浅色模式切换）
                    if ticks % 60 == 0 {
                        let progress = model.progress();
                        drop(model);
                        refresh_tray_icon(&app, progress);
                    }
                }
            }
        }
    });
}

#[tauri::command]
fn get_state(app: AppHandle) -> StateView {
    use tauri_plugin_autostart::ManagerExt;
    let shared = app.state::<Shared>();
    let model = shared.model.lock().unwrap().clone();
    StateView {
        model,
        now: now_ts(),
        autostart: app.autolaunch().is_enabled().unwrap_or(false),
    }
}

#[tauri::command]
fn add_ml(app: AppHandle, ml: u32) {
    let shared = app.state::<Shared>();
    let mut model = shared.model.lock().unwrap();
    model.add_ml(now_ts(), ml);
    drop(model);
    after_change(&app, true);
}

#[tauri::command]
fn set_interval(app: AppHandle, minutes: u32) {
    let shared = app.state::<Shared>();
    let mut model = shared.model.lock().unwrap();
    model.set_interval(now_ts(), minutes);
    drop(model);
    after_change(&app, false);
}

#[tauri::command]
fn set_goal(app: AppHandle, goal: u32) {
    let shared = app.state::<Shared>();
    let mut model = shared.model.lock().unwrap();
    model.set_goal(goal);
    drop(model);
    after_change(&app, true);
}

#[tauri::command]
fn toggle_pause(app: AppHandle) {
    let shared = app.state::<Shared>();
    let mut model = shared.model.lock().unwrap();
    model.toggle_pause(now_ts());
    drop(model);
    after_change(&app, false);
}

#[tauri::command]
fn snooze(app: AppHandle, minutes: i64) {
    let shared = app.state::<Shared>();
    let mut model = shared.model.lock().unwrap();
    model.snooze(now_ts(), minutes);
    drop(model);
    after_change(&app, false);
}

#[tauri::command]
fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    let autolaunch = app.autolaunch();
    if enabled {
        autolaunch.enable().map_err(|e| e.to_string())
    } else {
        autolaunch.disable().map_err(|e| e.to_string())
    }
}

/// 前端加载后按内容实际高度调整窗口尺寸
#[tauri::command]
fn fit_panel(app: AppHandle, height: f64) {
    if let Some(panel) = app.get_webview_window("panel") {
        let _ = panel.set_size(tauri::LogicalSize::new(324.0, height));
    }
}

#[tauri::command]
fn quit(app: AppHandle) {
    app.exit(0);
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]), // 登录自启时静默启动，不弹主窗口
        ))
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // macOS：用 UNUserNotificationCenter 请求通知授权（弹一次系统授权框），
            // 授权后 mac-notification-sys 发的带按钮通知才能展示横幅。
            #[cfg(target_os = "macos")]
            {
                use tauri_plugin_notification::NotificationExt;
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    if let Err(e) = handle.notification().request_permission() {
                        eprintln!("waterhelp: 请求通知授权失败 {e:?}");
                    }
                });
            }

            // 数据目录（macOS: ~/Library/Application Support/com.waterhelp.desktop）
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let model = state::Model::load(&data_dir);
            let progress = model.progress();
            app.manage(Shared {
                model: Mutex::new(model),
                data_dir,
            });

            // 面板窗口（tauri.conf.json 里定义）：平台毛玻璃特效 + 失焦自动隐藏
            let panel = app
                .get_webview_window("panel")
                .expect("panel window defined in tauri.conf.json");
            #[cfg(target_os = "macos")]
            panel
                .set_effects(tauri::utils::config::WindowEffectsConfig {
                    effects: vec![tauri::utils::WindowEffect::Sidebar],
                    state: Some(tauri::utils::WindowEffectState::Active),
                    ..Default::default()
                })
                .ok();
            #[cfg(target_os = "windows")]
            panel
                .set_effects(tauri::utils::config::WindowEffectsConfig {
                    effects: vec![
                        tauri::utils::WindowEffect::Acrylic,
                        tauri::utils::WindowEffect::Mica,
                    ],
                    ..Default::default()
                })
                .ok();
            {
                // 面板失焦自动收起（刚显示的 400ms 内忽略，避免显示瞬间被隐藏）
                let app = app.handle().clone();
                panel.on_window_event(move |event| {
                    if let tauri::WindowEvent::Focused(false) = event {
                        let just_shown = PANEL_SHOWN_AT
                            .lock()
                            .unwrap()
                            .map_or(false, |t| t.elapsed() < Duration::from_millis(400));
                        if just_shown {
                            return;
                        }
                        if let Some(p) = app.get_webview_window("panel") {
                            let _ = p.hide();
                        }
                    }
                });
            }

            // 主窗口：点关闭只隐藏不销毁（下次从 Dock/启动台秒开）
            if let Some(main_win) = app.get_webview_window("main") {
                let app = app.handle().clone();
                main_win.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.hide();
                        }
                    }
                });
            }

            // 托盘右键菜单
            let open_main = MenuItem::with_id(app, "open_main", "打开喝水助手", true, None::<&str>)?;
            let open_item = MenuItem::with_id(app, "open", "快捷面板", true, None::<&str>)?;
            let pause_item =
                MenuItem::with_id(app, "pause", "暂停/恢复 1 小时", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出喝水助手", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_main, &open_item, &pause_item, &quit_item])?;

            let tray_icon = droplet::render_png(progress, droplet::detect_appearance())
                .and_then(|png| tauri::image::Image::from_bytes(&png).ok())
                .expect("render tray icon");

            let app_handle = app.handle().clone();
            TrayIconBuilder::with_id("main")
                .icon(tray_icon)
                .tooltip("喝水助手 WaterHelp")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "open_main" => show_main(app),
                    "open" => toggle_panel(app),
                    "pause" => {
                        let shared = app.state::<Shared>();
                        let mut model = shared.model.lock().unwrap();
                        model.toggle_pause(now_ts());
                        drop(model);
                        after_change(app, false);
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(move |_, event| {
                    // 左键单击切换面板。不区分按下/抬起（平台行为不一），
                    // 用 250ms 防抖避免一次点击触发两次切换。
                    let should_toggle = match &event {
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            ..
                        } => true,
                        // 双击兜底（个别版本单击事件不派发）
                        TrayIconEvent::DoubleClick {
                            button: MouseButton::Left,
                            ..
                        } => true,
                        _ => false,
                    };
                    if should_toggle {
                        let now = std::time::Instant::now();
                        let mut last = LAST_TRAY_CLICK.lock().unwrap();
                        if last.map_or(false, |t| {
                            now.duration_since(t) < Duration::from_millis(250)
                        }) {
                            return;
                        }
                        *last = Some(now);
                        drop(last);

                        if let TrayIconEvent::Click { rect, .. } = &event {
                            // rect 的 position/size 是 Physical/Logical 枚举，取出数值
                            let (x, y) = match rect.position {
                                tauri::Position::Physical(p) => (p.x as f64, p.y as f64),
                                tauri::Position::Logical(p) => (p.x, p.y),
                            };
                            let (w, h) = match rect.size {
                                tauri::Size::Physical(s) => (s.width as f64, s.height as f64),
                                tauri::Size::Logical(s) => (s.width, s.height),
                            };
                            *TRAY_RECT.lock().unwrap() = Some(TrayRect { x, y, w, h });
                        }
                        toggle_panel(&app_handle);
                    }
                })
                .build(app)?;

            // 用户手动启动（启动台/Dock/Finder）时打开主窗口；
            // 登录自启带 --hidden 参数，静默常驻托盘。
            if !std::env::args().any(|a| a == "--hidden") {
                show_main(app.handle());
            }
            start_ticker(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            add_ml,
            set_interval,
            set_goal,
            toggle_pause,
            snooze,
            set_autostart,
            fit_panel,
            quit
        ])
        .build(tauri::generate_context!())
        .expect("error while building waterhelp")
        .run(|app, event| {
            // macOS：点击 Dock 图标重新打开主窗口（Reopen 事件仅 macOS 存在）
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen { .. } = event {
                show_main(app);
            }
            #[cfg(not(target_os = "macos"))]
            {
                let _ = (app, event);
            }
        });
}
