// ============================================================
//  api.js — Commands & Shared State
// ============================================================

// ── Shared State trên window — truy cập từ mọi file ──────────
window.AppState = { isAuto: false };

window.CMD_LABELS = {
  F:'FORWARD', B:'BACKWARD', L:'LEFT', R:'RIGHT',
  S:'STOP', '1':'CAM-LEFT', '2':'CAM-RIGHT', '3':'CAM-CENTER'
};

// ── Cache DOM elements dùng nhiều lần ────────────────────────
const _btnAuto  = () => document.getElementById('btn-auto');
const _autoLabel = () => document.getElementById('auto-label');
const _dpadWrap  = () => document.getElementById('dpad-wrap');

// ── Gửi lệnh điều khiển ──────────────────────────────────────
function cmd(c) {
  if (window.AppState.isAuto && 'FBLRS'.includes(c)) return;
  fetch('/control/' + c).catch(() => window.setOffline?.());
  window.flashBtn?.('btn-' + c);
  window.updateLastCmd?.(c);
  window.appendLog?.(c);
}

// ── Toggle Auto Drive ─────────────────────────────────────────
function toggleAuto() {
  window.AppState.isAuto = !window.AppState.isAuto;
  fetch('/auto/' + (window.AppState.isAuto ? 'on' : 'off'))
    .catch(() => window.setOffline?.());

  const isAuto = window.AppState.isAuto;
  _btnAuto()?.classList.toggle('active', isAuto);
  _dpadWrap()?.classList.toggle('dimmed', isAuto);

  const lbl = _autoLabel();
  if (lbl) lbl.textContent = isAuto ? 'TẮT TỰ LÁI' : 'BẬT TỰ LÁI';

  window.appendLogRaw?.(
    isAuto ? 'AUTO DRIVE ON' : 'MANUAL MODE',
    isAuto ? 'var(--amber)' : 'var(--green)'
  );
  window.updateLastCmd?.(isAuto ? 'AUTO' : 'MANUAL');
}

window.cmd        = cmd;
window.toggleAuto = toggleAuto;