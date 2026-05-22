// ============================================================
//  ui.js — UI Updates, Status Polling, Keyboard Shortcuts
// ============================================================

// ── Cache DOM elements (tránh getElementById mỗi lần gọi) ───
const _el = id => document.getElementById(id);
const elDist     = _el('dist');
const elLastCmd  = _el('last-cmd');
const elConnDot  = _el('conn-dot');
const elConnText = _el('conn-text');
const elLogBox   = _el('log-box');
const elCamImg   = _el('camera-img');

// ── Visual feedback ───────────────────────────────────────────
function flashBtn(id) {
  const el = _el(id);
  if (!el) return;
  el.classList.add('pressed');
  setTimeout(() => el.classList.remove('pressed'), 200);
}

function updateLastCmd(c) {
  if (!elLastCmd) return;
  elLastCmd.textContent = window.CMD_LABELS?.[c] || c;
  elLastCmd.style.color = c === 'S'    ? 'var(--red)'
                        : c === 'AUTO' ? 'var(--amber)'
                        : 'var(--green)';
}

function appendLog(c) {
  appendLogRaw(
    window.CMD_LABELS?.[c] || c,
    c === 'S' ? 'var(--red)' : 'var(--green)'
  );
}

function appendLogRaw(text, color) {
  if (!elLogBox) return;
  const now  = new Date().toLocaleTimeString('vi-VN', { hour12: false });
  const line = document.createElement('div');
  line.className = 'log-line';
  line.innerHTML = `${now} → <span style="color:${color}">${text}</span>`;
  elLogBox.appendChild(line);
  elLogBox.scrollTop = elLogBox.scrollHeight;
  // Giữ tối đa 50 dòng
  while (elLogBox.children.length > 50) elLogBox.removeChild(elLogBox.firstChild);
}

// ── Connection helpers ────────────────────────────────────────
function setOnline() {
  elConnDot?.classList.remove('offline');
  if (elConnText) elConnText.textContent = 'LIVE';
}
function setOffline() {
  elConnDot?.classList.add('offline');
  if (elConnText) elConnText.textContent = 'LOST';
}

// ── Status polling ────────────────────────────────────────────
// AbortController: hủy request cũ nếu server chậm hơn interval
let _pollController = null;

async function pollStatus() {
  // Bỏ qua nếu tab đang ẩn (tiết kiệm pin, giảm tải Pi)
  if (document.hidden) return;

  // Hủy request trước nếu chưa xong
  _pollController?.abort();
  _pollController = new AbortController();

  try {
    const r = await fetch('/status', { signal: _pollController.signal });
    const d = await r.json();
    setOnline();

    // Khoảng cách
    const val = parseFloat(d.distance);
    if (elDist) {
      elDist.textContent = isNaN(val) ? '---' : val;
      elDist.className = 'stat-value';
      if (!isNaN(val)) {
        if      (val < 15) elDist.classList.add('danger');
        else if (val < 30) elDist.classList.add('warn');
      }
    }

    // Đồng bộ auto state từ server khi reload trang
    const serverAutoMode = d.auto || 'off';
    if (serverAutoMode !== window.AppState?.autoMode) {
      window.AppState.autoMode = serverAutoMode;
      ['off', 'cpp', 'ai'].forEach(m => {
        const btn = _el('btn-auto-' + m);
        if (btn) btn.classList.toggle('active', m === serverAutoMode);
      });
      const dpad = _el('dpad-wrap');
      dpad?.classList.toggle('dimmed', serverAutoMode !== 'off');
    }

    // Hiển thị log AI agent thời gian thực
    if (d.ai_log && serverAutoMode === 'ai') {
      const elAiDesc = _el('ai-desc');
      if (elAiDesc) elAiDesc.textContent = d.ai_log;
    }

  } catch (e) {
    if (e.name !== 'AbortError') setOffline();
  }
}

setInterval(pollStatus, 500);

// Pause/resume polling theo tab visibility
document.addEventListener('visibilitychange', () => {
  if (!document.hidden) pollStatus(); // Resume ngay khi tab active lại
});

// ── Camera Snapshot ───────────────────────────────────────────
// Việc chụp ảnh từ camera Pi hiện tại dùng phương thức Snapshot-on-demand
// được xử lý hoàn toàn trong hand_tracking.js thông qua endpoint /snapshot.

// ── Keyboard shortcuts ────────────────────────────────────────
const KEY_MAP = {
  w:'F', W:'F', ArrowUp:'F',
  s:'B', S:'B', ArrowDown:'B',
  a:'L', A:'L', ArrowLeft:'L',
  d:'R', D:'R', ArrowRight:'R',
  x:'S', X:'S', ' ':'S',
  '1':'1', '2':'2', '3':'3',
  p:'AUTO', P:'AUTO',
  v:'VOICE', V:'VOICE',
  g:'AI', G:'AI',
};
document.addEventListener('keydown', e => {
  if (e.repeat) return;
  // Bỏ qua nếu đang gõ trong input field
  if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;
  const c = KEY_MAP[e.key];
  if (!c) return;
  e.preventDefault();
  if      (c === 'AUTO')  window.toggleAuto?.();
  else if (c === 'VOICE') window.toggleVoice?.();
  else if (c === 'AI')    window.triggerAiAnalyze?.();
  else                    window.cmd?.(c);
});

let _suggestTimeout = null;

function showAiSuggestion(cmd, desc) {
  // Xóa highlight cũ ở tất cả các phím điều hướng
  const buttons = ['btn-F', 'btn-B', 'btn-L', 'btn-R', 'btn-S'];
  buttons.forEach(id => {
    _el(id)?.classList.remove('highlight-suggest');
  });

  // Highlight phím đề xuất mới
  const targetId = 'btn-' + cmd;
  const btn = _el(targetId);
  if (btn) {
    btn.classList.add('highlight-suggest');

    // Reset timeout cũ
    if (_suggestTimeout) clearTimeout(_suggestTimeout);

    // Tự tắt sau 5 giây
    _suggestTimeout = setTimeout(() => {
      btn.classList.remove('highlight-suggest');
    }, 5000);
  }
}

// ── Expose ra window ──────────────────────────────────────────
Object.assign(window, {
  appendLogRaw, appendLog,
  flashBtn, updateLastCmd,
  setOnline, setOffline,
  showAiSuggestion,
});