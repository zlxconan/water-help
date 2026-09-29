// 喝水助手主窗口：今日概览 + 最近 14 天图表 + 历史记录
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const $ = (id) => document.getElementById(id);
const RING_C = 364.4; // 2πr, r=58
const WEEK = ["日", "一", "二", "三", "四", "五", "六"];

let state = null;

function pad(n) { return String(n).padStart(2, "0"); }

function parseDay(day) {
  const [y, m, d] = day.split("-").map(Number);
  return new Date(y, m - 1, d);
}

function fmtDay(day) {
  const d = parseDay(day);
  return `${mday(d)} 周${WEEK[d.getDay()]}`;
}
function mday(d) { return `${d.getMonth() + 1}/${d.getDate()}`; }

function render() {
  if (!state) return;
  const m = state.model;
  const progress = m.goal_ml > 0 ? Math.min(1, m.intake_ml / m.goal_ml) : 0;

  // 头部日期
  const today = parseDay(m.day);
  $("today-label").textContent = `${today.getFullYear()}年${today.getMonth() + 1}月${today.getDate()}日 周${WEEK[today.getDay()]}`;

  // 圆环
  $("ring-fg").style.strokeDashoffset = String(RING_C * (1 - progress));
  $("pct").textContent = `${Math.round(progress * 100)}%`;
  $("amount").textContent = `${m.intake_ml} / ${m.goal_ml} ml`;
  $("count").textContent = String(m.drink_count);
  $("remain").textContent = String(Math.max(0, m.goal_ml - m.intake_ml));

  renderChart(m);
  renderList(m);
}

function renderChart(m) {
  const byDay = new Map(m.history.map((r) => [r.day, r]));
  const days = [];
  const cursor = parseDay(m.day);
  for (let i = 13; i >= 0; i--) {
    const d = new Date(cursor);
    d.setDate(d.getDate() - i);
    const key = `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
    days.push({ key, label: mday(d), rec: byDay.get(key) });
  }
  $("chart").innerHTML = days
    .map(({ key, label, rec }) => {
      const intake = rec ? rec.intake_ml : 0;
      const goal = rec ? rec.goal_ml : m.goal_ml;
      const pct = goal > 0 ? Math.min(1, intake / goal) : 0;
      const isToday = key === m.day;
      const cls = [`bar-col${isToday ? " today" : ""}${intake === 0 ? " zero" : ""}`].join(" ");
      return `<div class="${cls}" title="${key} · ${intake}ml / ${goal}ml">
        <div class="bar" style="height:${Math.max(3, pct * 100)}%"></div>
        <div class="bar-label">${label}</div>
      </div>`;
    })
    .join("");
}

function renderList(m) {
  const rows = [...m.history].reverse(); // 新的在前
  if (rows.length === 0) {
    $("history-list").innerHTML = `<div class="empty">还没有记录，喝第一杯水吧 💧</div>`;
    return;
  }
  const head = `<div class="hrow head"><span class="date">日期</span><span class="ml">摄入 / 目标</span><span class="pct">达成</span><span class="ok">状态</span></div>`;
  const body = rows
    .map((r) => {
      const pct = r.goal_ml > 0 ? Math.round((r.intake_ml / r.goal_ml) * 100) : 0;
      const done = r.intake_ml >= r.goal_ml && r.goal_ml > 0;
      const isToday = r.day === m.day;
      return `<div class="hrow">
        <span class="date">${isToday ? "今天" : fmtDay(r.day)}</span>
        <span class="ml">${r.intake_ml} / ${r.goal_ml} ml · ${r.drink_count} 次</span>
        <span class="pct">${pct}%</span>
        <span class="ok${done ? " done" : ""}">${done ? "✓ 达标" : "—"}</span>
      </div>`;
    })
    .join("");
  $("history-list").innerHTML = head + body;
}

async function refresh() {
  try {
    state = await invoke("get_state");
    render();
  } catch (e) {
    console.error("get_state failed", e);
  }
}

document.querySelectorAll(".quick").forEach((b) =>
  b.addEventListener("click", () => invoke("add_ml", { ml: Number(b.dataset.ml) }))
);

window.addEventListener("DOMContentLoaded", async () => {
  await refresh();
  setInterval(refresh, 2000);
  listen("state-changed", refresh).catch(() => {});
});
