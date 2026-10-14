// ============================================================
//  api.js — Commands & Shared State
// ============================================================

// ── Shared State trên window — truy cập từ mọi file ──────────
window.AppState = { autoMode: 'off' };

window.CMD_LABELS = {
  F:'FORWARD', B:'BACKWARD', L:'LEFT', R:'RIGHT',
  S:'STOP', '1':'CAM-LEFT', '2':'CAM-RIGHT', '3':'CAM-CENTER'
};

// ── Cache DOM elements dùng nhiều lần ────────────────────────
const _dpadWrap  = () => document.getElementById('dpad-wrap');

// ── Gửi lệnh điều khiển ──────────────────────────────────────
function cmd(c) {
  if (window.AppState.autoMode !== 'off' && 'FBLRS'.includes(c)) return;
  fetch('/control/' + c).catch(() => window.setOffline?.());
  window.flashBtn?.('btn-' + c);
  window.updateLastCmd?.(c);
  window.appendLog?.(c);
}

// ── Set Auto Drive Mode ──────────────────────────────────────
function setAutoMode(mode) {
  if (!['off', 'cpp', 'ai'].includes(mode)) return;
  
  window.AppState.autoMode = mode;
  fetch('/auto/' + mode).catch(() => window.setOffline?.());

  // Cập nhật class active cho nút bấm
  ['off', 'cpp', 'ai'].forEach(m => {
    const btn = document.getElementById('btn-auto-' + m);
    if (btn) btn.classList.toggle('active', m === mode);
  });

  // Làm mờ D-pad nếu ở chế độ tự động
  const isAuto = (mode !== 'off');
  _dpadWrap()?.classList.toggle('dimmed', isAuto);

  // Ghi log trạng thái mới
  let logText = 'MANUAL MODE';
  let logColor = 'var(--green)';
  if (mode === 'cpp') {
    logText = 'AUTO DRIVE: NATIVE (CPP)';
    logColor = 'var(--green)';
  } else if (mode === 'ai') {
    logText = 'AUTO DRIVE: AI AGENT (RUST)';
    logColor = 'var(--amber)';
  }

  window.appendLogRaw?.(logText, logColor);
  window.updateLastCmd?.(mode === 'off' ? 'MANUAL' : 'AUTO_' + mode.toUpperCase());
}

function toggleAuto() {
  const current = window.AppState.autoMode;
  const next = (current === 'off') ? 'ai' : 'off';
  setAutoMode(next);
}

window.cmd         = cmd;
window.setAutoMode = setAutoMode;
window.toggleAuto  = toggleAuto;