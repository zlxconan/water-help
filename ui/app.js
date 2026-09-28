// 喝水助手面板逻辑：通过 Tauri invoke 与 Rust 后端通信
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const $ = (id) => document.getElementById(id);
const RING_C = 345.6; // 2πr, r=55

let state = null;

function pad(n) { return String(n).padStart(2, "0"); }

function fmtTime(tsSec) {
  const d = new Date(tsSec * 1000);
  return `${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

function render() {
  if (!state) return;
  const m = state.model;
  const progress = m.goal_ml > 0 ? Math.min(1, m.intake_ml / m.goal_ml) : 0;

  // 圆环与数字
  $("ring-fg").style.strokeDashoffset = String(RING_C * (1 - progress));
  $("pct").textContent = `${Math.round(progress * 100)}%`;
  $("amount").textContent = `${m.intake_ml} / ${m.goal_ml} ml`;

  // 下次提醒
  const paused = m.paused_until && m.paused_until > state.now;
  const next = $("next-reminder");
  if (paused) {
    next.textContent = `已暂停 · ${fmtTime(m.paused_until)} 恢复`;
  } else {
    const mins = Math.max(0, Math.ceil((m.next_reminder - state.now) / 60));
    next.textContent = `${fmtTime(m.next_reminder)} · ${mins > 0 ? mins + " 分钟后" : "马上"}`;
  }

  // 暂停按钮
  $("pause-btn").textContent = paused ? "▶ 已暂停，点击恢复" : "⏸ 暂停 1 小时";

  // 间隔分段
  document.querySelectorAll("#seg-interval button").forEach((b) => {
    b.classList.toggle("on", Number(b.dataset.min) === m.interval_min);
  });

  // 目标
  $("goal-num").textContent = `${m.goal_ml} ml`;

  // 打卡次数 / 自启
  $("drink-count").textContent = `今日已打卡 ${m.drink_count} 次`;
  $("autostart").setAttribute("aria-checked", String(!!state.autostart));
}

async function refresh() {
  try {
    state = await invoke("get_state");
    render();
  } catch (e) {
    console.error("get_state failed", e);
  }
}

// ---- 交互 ----
document.querySelectorAll(".quick").forEach((b) =>
  b.addEventListener("click", () => invoke("add_ml", { ml: Number(b.dataset.ml) }))
);

$("pause-btn").addEventListener("click", () => invoke("toggle_pause"));

document.querySelectorAll("#seg-interval button").forEach((b) =>
  b.addEventListener("click", () => invoke("set_interval", { minutes: Number(b.dataset.min) }))
);

$("goal-minus").addEventListener("click", () => invoke("set_goal", { goal: state.model.goal_ml - 100 }));
$("goal-plus").addEventListener("click", () => invoke("set_goal", { goal: state.model.goal_ml + 100 }));

$("autostart").addEventListener("click", () => {
  invoke("set_autostart", { enabled: !state.autostart }).then(refresh).catch((e) => console.error(e));
});

$("quit-btn").addEventListener("click", () => invoke("quit"));

// ---- 启动 ----
window.addEventListener("DOMContentLoaded", async () => {
  await refresh();
  // 按内容实际高度校准窗口尺寸（Rust 侧 set_size）
  requestAnimationFrame(() => {
    invoke("fit_panel", { height: document.documentElement.scrollHeight }).catch(() => {});
  });
  setInterval(refresh, 1000);
  listen("state-changed", refresh).catch(() => {});
});
