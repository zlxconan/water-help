//! 各平台原生通知。
//!
//! - macOS: mac-notification-sys（NSUserNotification），支持「喝了 +250ml」
//!   主按钮和「稍后提醒」关闭按钮，用户点击后回调业务逻辑。
//! - Windows: winrt-notification（WinRT Toast），v1 先支持标题+正文+声音，
//!   通知按钮交互在 roadmap 中（需要 windows crate 的 Toast XML）。

/// 当前提醒内容（文案参数快照）
pub struct Reminder {
    pub since_minutes: u32,
    pub goal_reached: bool,
}

fn texts(model_intake: u32, r: &Reminder) -> (String, String) {
    if r.goal_reached {
        (
            "今日目标已达成 🎉".into(),
            format!("已喝 {}ml，保持状态，再来一杯吧～", model_intake),
        )
    } else {
        (
            "该喝水啦 💧".into(),
            format!(
                "已经 {} 分钟没喝水了，起来接杯温水吧，建议 250ml。",
                r.since_minutes
            ),
        )
    }
}

/// 发送提醒。用户在通知上的操作会通过 `on_action` 回调
/// （"drank" = 喝了 250ml，"snooze" = 稍后提醒）。
pub fn remind<F>(model_intake: u32, r: Reminder, on_action: F)
where
    F: FnOnce(&str) + Send + 'static,
{
    let (title, body) = texts(model_intake, &r);

    #[cfg(target_os = "macos")]
    {
        std::thread::spawn(move || {
            use mac_notification_sys::{MainButton, Notification, NotificationResponse};
            // send() 会阻塞等待用户交互，因此放在独立线程
            let response = Notification::new()
                .title(&title)
                .message(&body)
                .default_sound()
                .main_button(MainButton::SingleAction("喝了 +250ml"))
                .close_button("稍后提醒")
                .asynchronous(false)
                .send();
            match response {
                Ok(NotificationResponse::ActionButton(_)) => on_action("drank"),
                Ok(NotificationResponse::CloseButton(_)) => on_action("snooze"),
                _ => {}
            }
        });
    }

    #[cfg(target_os = "windows")]
    {
        let _ = on_action; // Windows v1 无按钮回调
        let toast = winrt_notification::Toast::new(POWERSHELL_AUMID)
            .title(&title)
            .text1(&body)
            .sound(Some(winrt_notification::Sound::Default));
        if let Err(e) = toast.show() {
            eprintln!("waterhelp: Windows 通知发送失败 {e:?}");
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = (model_intake, on_action);
        println!("waterhelp: {} - {}", title, body);
    }
}

/// Windows 上借用 PowerShell 的 AUMID 保证 Toast 能稳定显示（见 winrt-notification 文档）；
/// 正式安装包后续可注册自己的 AppUserModelID。
#[cfg(target_os = "windows")]
const POWERSHELL_AUMID: &str =
    "{1AC14E34-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe";
