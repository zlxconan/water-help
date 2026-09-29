use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 单日历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayRecord {
    pub day: String, // yyyy-MM-dd
    pub intake_ml: u32,
    pub drink_count: u32,
    pub goal_ml: u32,
}

/// 全部业务状态。持久化为 JSON（应用数据目录下的 state.json），
/// 按日期键记录饮水量，跨天自动清零；history 保留最近 90 天。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub intake_ml: u32,
    pub drink_count: u32,
    pub goal_ml: u32,
    pub interval_min: u32,
    /// Unix 秒
    pub last_drink: Option<i64>,
    pub paused_until: Option<i64>,
    pub next_reminder: i64,
    /// yyyy-MM-dd（本地时区），用于跨天重置
    pub day: String,
    /// 最近若干天的历史（含今天，按日期升序）
    #[serde(default)]
    pub history: Vec<DayRecord>,
}

#[derive(Debug)]
pub enum Tick {
    Nothing,
    /// 到点该发提醒了
    Due {
        since_minutes: u32,
        goal_reached: bool,
    },
}

impl Default for Model {
    fn default() -> Self {
        let now = chrono::Local::now();
        let now_ts = now.timestamp();
        Self {
            intake_ml: 0,
            drink_count: 0,
            goal_ml: 2000,
            interval_min: 45,
            last_drink: None,
            paused_until: None,
            next_reminder: now_ts + 45 * 60,
            day: now.format("%Y-%m-%d").to_string(),
            history: Vec::new(),
        }
    }
}

impl Model {
    pub fn load(dir: &Path) -> Self {
        let now = chrono::Local::now().timestamp();
        let file = dir.join("state.json");
        match fs::read_to_string(&file) {
            Err(_) => {
                // 首次启动：倒计时从现在开始
                let mut m = Self::default();
                m.next_reminder = now + m.interval_secs();
                m
            }
            Ok(text) => match serde_json::from_str::<Model>(&text) {
                // 丢弃过期的倒计时，避免一启动就立刻弹提醒
                Ok(mut m) => {
                    if m.next_reminder <= now {
                        m.next_reminder = now + m.interval_secs();
                    }
                    if let Some(p) = m.paused_until {
                        if p <= now {
                            m.paused_until = None;
                        }
                    }
                    m
                }
                Err(e) => {
                    eprintln!("waterhelp: state.json 解析失败({e})，使用默认值");
                    let mut m = Self::default();
                    m.next_reminder = now + m.interval_secs();
                    m
                }
            },
        }
    }

    pub fn save(&mut self, dir: &Path) {
        self.upsert_today_history();
        let file = dir.join("state.json");
        if let Ok(text) = serde_json::to_string_pretty(self) {
            if let Err(e) = fs::write(&file, text) {
                eprintln!("waterhelp: 保存状态失败 {e}");
            }
        }
    }

    /// 把今天的记录并入历史（upsert），保留最近 90 天
    fn upsert_today_history(&mut self) {
        let rec = DayRecord {
            day: self.day.clone(),
            intake_ml: self.intake_ml,
            drink_count: self.drink_count,
            goal_ml: self.goal_ml,
        };
        match self.history.iter_mut().find(|r| r.day == self.day) {
            Some(slot) => *slot = rec,
            None => {
                self.history.push(rec);
                self.history.sort_by(|a, b| a.day.cmp(&b.day));
            }
        }
        if self.history.len() > 90 {
            let drop = self.history.len() - 90;
            self.history.drain(0..drop);
        }
    }

    pub fn progress(&self) -> f32 {
        if self.goal_ml > 0 {
            (self.intake_ml as f32 / self.goal_ml as f32).min(1.0)
        } else {
            0.0
        }
    }

    pub fn goal_reached(&self) -> bool {
        self.intake_ml >= self.goal_ml
    }

    pub fn is_paused(&self, now: i64) -> bool {
        self.paused_until.map_or(false, |p| p > now)
    }

    /// 提醒间隔（秒）。debug 构建支持 WATERHELP_INTERVAL_SECS 环境变量覆盖，便于开发调试。
    fn interval_secs(&self) -> i64 {
        #[cfg(debug_assertions)]
        if let Ok(s) = std::env::var("WATERHELP_INTERVAL_SECS") {
            if let Ok(s) = s.parse::<i64>() {
                if s > 0 {
                    return s;
                }
            }
        }
        self.interval_min as i64 * 60
    }

    pub fn minutes_since_last_drink(&self, now: i64) -> u32 {
        match self.last_drink {
            Some(t) => (((now - t) / 60) as u32).max(1),
            None => self.interval_min.max(1),
        }
    }

    /// 每秒心跳：跨天重置 / 暂停到点恢复 / 到点提醒
    pub fn tick(&mut self, now: i64) -> Tick {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        if today != self.day {
            self.day = today;
            self.intake_ml = 0;
            self.drink_count = 0;
            self.last_drink = None;
        }

        if let Some(p) = self.paused_until {
            if p <= now {
                self.paused_until = None;
                self.next_reminder = now + self.interval_secs();
            }
        }

        if !self.is_paused(now) && self.next_reminder <= now {
            let since = self.minutes_since_last_drink(now);
            let reached = self.goal_reached();
            self.next_reminder = now + self.interval_secs();
            return Tick::Due {
                since_minutes: since,
                goal_reached: reached,
            };
        }
        Tick::Nothing
    }

    pub fn add_ml(&mut self, now: i64, ml: u32) {
        self.intake_ml += ml;
        self.drink_count += 1;
        self.last_drink = Some(now);
        self.next_reminder = now + self.interval_secs();
    }

    pub fn snooze(&mut self, now: i64, minutes: i64) {
        self.next_reminder = now + minutes * 60;
    }

    pub fn toggle_pause(&mut self, now: i64) {
        if self.is_paused(now) {
            self.paused_until = None;
            self.next_reminder = now + self.interval_secs();
        } else {
            self.paused_until = Some(now + 3600);
        }
    }

    pub fn set_interval(&mut self, now: i64, minutes: u32) {
        self.interval_min = minutes;
        if !self.is_paused(now) {
            self.next_reminder = now + self.interval_secs();
        }
    }

    pub fn set_goal(&mut self, goal_ml: u32) {
        self.goal_ml = goal_ml.clamp(1000, 5000);
    }
}
