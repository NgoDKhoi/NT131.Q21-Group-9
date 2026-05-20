// ============================================================
//  hand_tracking.js — MediaPipe Hand Gesture Control v3
//  Victory ✌ → Snapshot Pi Camera (thay vì cmd('3'))
//  Geometry-based detection cho độ chính xác cao hơn ML model
// ============================================================
import {
  GestureRecognizer,
  FilesetResolver,
  DrawingUtils
} from "https://cdn.jsdelivr.net/npm/@mediapipe/tasks-vision@0.10.14/vision_bundle.mjs";

// ── DOM refs ──────────────────────────────────────────────────
const btnHand     = document.getElementById('btn-hand');
const btnLabel    = document.getElementById('hand-btn-label');
const loadingEl   = document.getElementById('hand-loading');
const previewWrap = document.getElementById('hand-preview');
const videoEl     = document.getElementById('hand-video');
const canvasEl    = document.getElementById('hand-canvas');
const gestureEl   = document.getElementById('gesture-display');
const camImg      = document.getElementById('camera-img');       // Pi Camera display
const camCard     = document.getElementById('camera-card');      // flash effect target
const ctx         = canvasEl.getContext('2d');

// ── State ─────────────────────────────────────────────────────
let recognizer  = null;
let drawUtils   = null;    // Khởi tạo 1 lần sau init — không new mỗi frame
let isTracking  = false;
let animFrameId = null;
let lastGesture = '';
let lastCmdTime = 0;

const CMD_INTERVAL     = 500;   // ms — cooldown lệnh di chuyển
const TOGGLE_INTERVAL  = 2000;  // ms — cooldown ILoveYou (tránh spam toggleAuto)
const SNAPSHOT_INTERVAL = 3000; // ms — cooldown Victory (tránh spam /snapshot)
let   lastSnapshotTime  = 0;

// ════════════════════════════════════════════════════════════
//  GESTURE MAP
//  Victory ✌ → triggerSnapshot() (không còn cmd('3'))
//  Servo center → phím '3' trên keyboard (ui.js)
// ════════════════════════════════════════════════════════════
const GESTURE_MAP = {
  'OK_Sign'     : { fn: () => window.cmd('F'),        label: '👌 TIẾN LÊN',        interval: CMD_INTERVAL     },
  'Closed_Fist' : { fn: () => window.cmd('S'),        label: '✊ DỪNG LẠI',        interval: CMD_INTERVAL     },
  'Pointing_Up' : { fn: () => window.cmd('B'),        label: '☝ LÙI LẠI',         interval: CMD_INTERVAL     },
  'Victory'     : { fn: () => triggerSnapshot(),      label: '✌ CHỤP ẢNH',        interval: SNAPSHOT_INTERVAL },
  'ILoveYou'    : { fn: () => window.toggleAuto(),   label: '🤟 BẬT/TẮT TỰ LÁI',  interval: TOGGLE_INTERVAL  },
  'Thumb_Left'  : { fn: () => window.cmd('L'),        label: '👈 RẼ TRÁI',         interval: CMD_INTERVAL     },
  'Thumb_Right' : { fn: () => window.cmd('R'),        label: '👉 RẼ PHẢI',         interval: CMD_INTERVAL     },
};

// ════════════════════════════════════════════════════════════
//  CUSTOM GEOMETRY — Hand Landmark Analysis
//
//  MediaPipe landmark indices:
//  Wrist: 0
//  Thumb:  CMC=1  MCP=2  IP=3   TIP=4
//  Index:  MCP=5  PIP=6  DIP=7  TIP=8
//  Middle: MCP=9  PIP=10 DIP=11 TIP=12
//  Ring:   MCP=13 PIP=14 DIP=15 TIP=16
//  Pinky:  MCP=17 PIP=18 DIP=19 TIP=20
//
//  Tọa độ: chuẩn hóa [0..1], y tăng xuống dưới
//  → Finger pointing UP: tip.y < pip.y < mcp.y
//  → Finger FOLDED:      tip.y > pip.y
// ════════════════════════════════════════════════════════════

/** Khoảng cách Euclid 2D giữa 2 landmark */
function dist(a, b) {
  return Math.hypot(a.x - b.x, a.y - b.y);
}

/**
 * isFingerExtended(tip, pip, mcp)
 * Ngón tay duỗi thẳng: tip cao hơn pip, pip cao hơn mcp.
 * Dùng threshold 0.02 để chịu nhiễu nhỏ.
 */
function isFingerExtended(tip, pip, mcp) {
  return tip.y < pip.y - 0.02 && pip.y < mcp.y + 0.01;
}

/**
 * isFingerFolded(tip, pip)
 * Ngón tay gập: tip thấp hơn pip (cuộn vào lòng bàn tay).
 */
function isFingerFolded(tip, pip) {
  return tip.y > pip.y + 0.02;
}

/**
 * isOKSign(lm)
 * Ngón cái (4) chạm ngón trỏ (8): dist < 0.05.
 * Override mọi nhãn ML — check ở Priority 1.
 */
function isOKSign(lm) {
  return dist(lm[4], lm[8]) < 0.05;
}

/**
 * isVictorySign(lm)
 * Geometry chính xác cho cử chỉ ✌:
 *   ✅ Index (8) extended
 *   ✅ Middle (12) extended
 *   ❌ Ring (16) folded
 *   ❌ Pinky (20) folded
 *   ❌ Thumb (4) không được chạm index (loại trừ OK Sign)
 *
 * Dùng geometry thay vì ML label "Victory" vì model đôi khi
 * nhầm Victory ↔ Open_Palm khi ngón cái duỗi ra.
 */
function isVictorySign(lm) {
  const indexExtended  = isFingerExtended(lm[8],  lm[6],  lm[5]);
  const middleExtended = isFingerExtended(lm[12], lm[10], lm[9]);
  const ringFolded     = isFingerFolded(lm[16], lm[14]);
  const pinkyFolded    = isFingerFolded(lm[20], lm[18]);
  const notOKSign      = dist(lm[4], lm[8]) >= 0.05; // Không nhầm với OK Sign

  return indexExtended && middleExtended && ringFolded && pinkyFolded && notOKSign;
}

/**
 * resolveThumbDir(lm)
 * Thumb_Up → Thumb_Left / Thumb_Right
 * dx: thumb tip (4) so với index MCP (5).
 */
function resolveThumbDir(lm) {
  const dx = lm[4].x - lm[5].x;
  if (dx < -0.12) return 'Thumb_Left';
  if (dx >  0.12) return 'Thumb_Right';
  return 'Thumb_Up'; // Không map lệnh
}

// ════════════════════════════════════════════════════════════
//  SNAPSHOT — Chụp ảnh từ Pi Camera khi Victory ✌ detected
// ════════════════════════════════════════════════════════════
async function triggerSnapshot() {
  const now = Date.now();
  if (now - lastSnapshotTime < SNAPSHOT_INTERVAL) return; // Cooldown
  lastSnapshotTime = now;

  // Flash animation trên camera card
  camCard?.classList.add('snapshot-flash');
  setTimeout(() => camCard?.classList.remove('snapshot-flash'), 400);

  try {
    const res = await fetch('/snapshot');
    if (!res.ok) throw new Error(`HTTP ${res.status}`);

    // Nhận JPEG blob → tạo Object URL → cập nhật img
    const blob = await res.blob();
    const url  = URL.createObjectURL(blob);

    if (camImg) {
      // Revoke URL cũ nếu là blob (tránh memory leak)
      if (camImg.src.startsWith('blob:')) URL.revokeObjectURL(camImg.src);
      camImg.src = url;
    }

    // Cập nhật timestamp trên camera card
    const tsEl = document.getElementById('cam-timestamp');
    if (tsEl) tsEl.textContent = new Date().toLocaleTimeString('vi-VN');

    window.appendLogRaw?.('📸 Snapshot @ ' + new Date().toLocaleTimeString('vi-VN'), 'var(--green)');

  } catch (err) {
    window.appendLogRaw?.('📸 Snapshot lỗi: ' + err.message, 'var(--red)');
  }
}

// ════════════════════════════════════════════════════════════
//  MEDIAPIPE INIT
// ════════════════════════════════════════════════════════════
async function initRecognizer() {
  loadingEl?.classList.add('visible');
  try {
    const vision = await FilesetResolver.forVisionTasks(
      "https://cdn.jsdelivr.net/npm/@mediapipe/tasks-vision@0.10.14/wasm"
    );
    recognizer = await GestureRecognizer.createFromOptions(vision, {
      baseOptions: {
        modelAssetPath: "https://storage.googleapis.com/mediapipe-models/gesture_recognizer/gesture_recognizer/float16/1/gesture_recognizer.task",
        delegate: "GPU"
      },
      runningMode               : "VIDEO",
      numHands                  : 1,
      minHandDetectionConfidence: 0.6,
      minHandPresenceConfidence : 0.6,
      minTrackingConfidence     : 0.5,
    });
    // DrawingUtils 1 lần — tái dùng mọi frame
    drawUtils = new DrawingUtils(ctx);
  } catch (err) {
    if (gestureEl) {
      gestureEl.textContent = '⚠ Lỗi tải AI model. Kiểm tra mạng.';
      gestureEl.style.color = 'var(--red)';
    }
    throw err;
  } finally {
    loadingEl?.classList.remove('visible');
  }
}

// ════════════════════════════════════════════════════════════
//  DETECT LOOP
// ════════════════════════════════════════════════════════════
function detectLoop(timestamp) {
  if (!isTracking || videoEl.readyState < 2) {
    animFrameId = requestAnimationFrame(detectLoop);
    return;
  }

  // Sync canvas size
  if (canvasEl.width !== videoEl.videoWidth) {
    canvasEl.width  = videoEl.videoWidth;
    canvasEl.height = videoEl.videoHeight;
  }

  const result = recognizer.recognizeForVideo(videoEl, timestamp);

  // Vẽ skeleton
  ctx.clearRect(0, 0, canvasEl.width, canvasEl.height);
  for (const lm of result.landmarks) {
    drawUtils.drawConnectors(lm, GestureRecognizer.HAND_CONNECTIONS,
      { color: '#00cfff44', lineWidth: 1.5 });
    drawUtils.drawLandmarks(lm, { color: '#00ff88', lineWidth: 1, radius: 3 });
  }

  // ── 4 tầng ưu tiên gesture detection ────────────────────
  let gestureName = 'None';

  if (result.landmarks.length > 0) {
    const lm  = result.landmarks[0];
    const top = result.gestures?.[0]?.[0];

    // P1: OK Sign (geometry) — override mọi nhãn ML
    if (isOKSign(lm)) {
      gestureName = 'OK_Sign';
    }
    // P2: Victory (geometry) — chính xác hơn ML label
    else if (isVictorySign(lm)) {
      gestureName = 'Victory';
    }
    // P3: Thumb direction (geometry từ ML seed)
    else if (top?.score > 0.70 && top.categoryName === 'Thumb_Up') {
      gestureName = resolveThumbDir(lm);
    }
    // P4: Các gesture ML khác (ILoveYou, Closed_Fist, Pointing_Up)
    else if (top?.score > 0.70 && top.categoryName in GESTURE_MAP) {
      gestureName = top.categoryName;
    }
  }

  updateGestureUI(gestureName);
  animFrameId = requestAnimationFrame(detectLoop);
}

// ════════════════════════════════════════════════════════════
//  UPDATE UI & DISPATCH
// ════════════════════════════════════════════════════════════
function updateGestureUI(name) {
  const map = GESTURE_MAP[name];

  if (!map) {
    if (lastGesture !== '') {
      if (gestureEl) {
        gestureEl.textContent = '— CHỜ CỬ CHỈ —';
        gestureEl.style.color = 'var(--muted)';
      }
      gestureEl?.classList.remove('active-gesture');
      lastGesture = '';
    }
    return;
  }

  if (gestureEl) {
    gestureEl.textContent = map.label;
    gestureEl.style.color = name === 'Victory' ? 'var(--green)' : 'var(--blue)';
  }
  gestureEl?.classList.add('active-gesture');

  const now     = Date.now();
  const changed = name !== lastGesture;
  const ready   = (now - lastCmdTime) >= map.interval;

  if (changed || ready) {
    lastGesture = name;
    lastCmdTime = now;
    map.fn();
    window.appendLogRaw?.('✋ ' + map.label, name === 'Victory' ? 'var(--green)' : 'var(--blue)');
  }
}

// ════════════════════════════════════════════════════════════
//  WEBCAM
// ════════════════════════════════════════════════════════════
async function startCamera() {
  const stream = await navigator.mediaDevices.getUserMedia({
    video: { facingMode: 'user', width: { ideal: 320 }, height: { ideal: 240 } }
  }).catch(err => {
    if (gestureEl) {
      gestureEl.textContent = err.name === 'NotAllowedError'
        ? '⚠ Bị chặn quyền Camera!' : '⚠ ' + err.message;
      gestureEl.style.color = 'var(--red)';
    }
    throw err;
  });

  videoEl.srcObject = stream;
  await new Promise(res => { videoEl.onloadeddata = res; });
  isTracking  = true;
  animFrameId = requestAnimationFrame(detectLoop);
}

function stopCamera() {
  isTracking = false;
  cancelAnimationFrame(animFrameId);
  ctx.clearRect(0, 0, canvasEl.width, canvasEl.height);
  videoEl.srcObject?.getTracks().forEach(t => t.stop());
  videoEl.srcObject = null;
  if (gestureEl) {
    gestureEl.textContent = '— CHƯA NHẬN DIỆN —';
    gestureEl.style.color = 'var(--blue)';
  }
  gestureEl?.classList.remove('active-gesture');
  lastGesture = '';
}

// ── Toggle ────────────────────────────────────────────────────
window.toggleHandTracking = async function () {
  if (isTracking) {
    stopCamera();
    btnHand?.classList.remove('active');
    if (btnLabel) btnLabel.textContent = 'BẬT CAMERA TAY';
    previewWrap?.classList.remove('visible');
    return;
  }

  if (!recognizer) {
    if (btnHand) btnHand.disabled = true;
    await initRecognizer().catch(() => {});
    if (btnHand) btnHand.disabled = false;
    if (!recognizer) return;
  }

  await startCamera().catch(() => {
    previewWrap?.classList.remove('visible');
  });

  if (isTracking) {
    btnHand?.classList.add('active');
    if (btnLabel) btnLabel.textContent = 'TẮT CAMERA TAY';
    previewWrap?.classList.add('visible');
  }
};