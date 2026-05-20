// ============================================================
//  voice.js — Voice Control (Web Speech API, vi-VN)
//  Keyboard shortcut V được xử lý bởi ui.js — không duplicate
//  ở đây nữa.
// ============================================================
(function () {
  const SR = window.SpeechRecognition || window.webkitSpeechRecognition;

  // Cache DOM
  const btnMic       = document.getElementById('btn-mic');
  const statusEl     = document.getElementById('voice-status');
  const transcriptEl = document.getElementById('voice-transcript');

  if (!SR) {
    if (btnMic) { btnMic.textContent = '✕'; btnMic.disabled = true; }
    if (transcriptEl) {
      transcriptEl.textContent = 'Trình duyệt không hỗ trợ Web Speech API. Dùng Chrome/Edge.';
      transcriptEl.classList.add('error');
    }
    return;
  }

  let recognition = null;
  let isListening = false;

  // ── Bảng lệnh ─────────────────────────────────────────────
  // Camera servo TRƯỚC di chuyển → tránh "nhìn trái" nhầm lệnh 'L'
  const VOICE_MAP = [
    { kw: ['nhìn trái', 'nhìn sang trái'],          fn: () => window.cmd('1') },
    { kw: ['nhìn phải', 'nhìn sang phải'],          fn: () => window.cmd('2') },
    { kw: ['nhìn thẳng', 'nhìn giữa'],              fn: () => window.cmd('3') },
    { kw: ['tiến', 'đi thẳng', 'tới'],              fn: () => window.cmd('F') },
    { kw: ['lùi', 'xuống'],                          fn: () => window.cmd('B') },
    { kw: ['quay trái', 'rẽ trái', 'sang trái'],    fn: () => window.cmd('L') },
    { kw: ['quay phải', 'rẽ phải', 'sang phải'],    fn: () => window.cmd('R') },
    { kw: ['dừng', 'đứng lại', 'stop', 'dừng lại'], fn: () => window.cmd('S') },
    { kw: ['tự lái', 'tự động'],                    fn: () => window.toggleAuto() },
  ];

  // ── Xử lý transcript ──────────────────────────────────────
  function processTranscript(text) {
    const t = text.toLowerCase().trim();
    if (transcriptEl) {
      transcriptEl.classList.remove('error');
      transcriptEl.style.color = '';
      transcriptEl.textContent = '❝ ' + t + ' ❞';
    }

    for (const rule of VOICE_MAP) {
      if (rule.kw.some(k => t.includes(k))) {
        rule.fn();
        window.appendLogRaw?.('🎙 ' + t, 'var(--blue)');
        return;
      }
    }

    // Không khớp
    if (transcriptEl) transcriptEl.style.color = 'var(--amber)';
    setTimeout(() => { if (transcriptEl) transcriptEl.style.color = ''; }, 1500);
    window.appendLogRaw?.('🎙 ? ' + t, 'var(--amber)');
  }

  // ── Build recognition object ───────────────────────────────
  // Tạo mới mỗi lần để tránh bug state của Web Speech API
  function buildRecognition() {
    const r = new SR();
    r.lang            = 'vi-VN';
    r.continuous      = false;
    r.interimResults  = false;
    r.maxAlternatives = 1;

    r.onstart = () => {
      isListening = true;
      btnMic?.classList.add('listening');
      statusEl?.classList.add('active');
      if (statusEl)     statusEl.textContent     = 'ĐANG NGHE...';
      if (transcriptEl) transcriptEl.textContent = '—';
    };

    r.onresult = e => processTranscript(e.results[0][0].transcript);

    r.onerror = e => {
      const ERRORS = {
        'not-allowed'  : '⚠ Micro bị chặn! HTTP → chrome://flags → "Insecure origins treated as secure" → thêm địa chỉ Pi.',
        'no-speech'    : 'Không nghe thấy gì. Thử lại.',
        'network'      : 'Lỗi mạng khi nhận dạng.',
        'audio-capture': 'Không tìm thấy Microphone.',
      };
      if (transcriptEl) {
        transcriptEl.textContent = ERRORS[e.error] || ('Lỗi: ' + e.error);
        transcriptEl.classList.add('error');
      }
      stopListening();
    };

    r.onend = stopListening;
    return r;
  }

  function stopListening() {
    isListening = false;
    btnMic?.classList.remove('listening');
    statusEl?.classList.remove('active');
    if (statusEl) statusEl.textContent = 'NHẤN ĐỂ NÓI';
  }

  // ── Toggle (gọi từ ui.js keydown + onclick HTML) ──────────
  window.toggleVoice = function () {
    if (isListening) {
      recognition?.stop();
      stopListening();
      return;
    }
    recognition = buildRecognition();
    try {
      recognition.start();
    } catch (err) {
      if (transcriptEl) {
        transcriptEl.textContent = 'Lỗi khởi động micro: ' + err.message;
        transcriptEl.classList.add('error');
      }
    }
  };
  // Không thêm keydown listener ở đây — ui.js đã xử lý phím V
})();