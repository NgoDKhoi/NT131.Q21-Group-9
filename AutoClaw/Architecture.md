# AutoClaw — Architecture

> Tài liệu kỹ thuật mô tả kiến trúc hệ thống, data flow, các quyết định thiết kế và lý do đằng sau chúng.

---

## Tổng quan hệ thống

```
┌─────────────────────────────────────────────────────────────┐
│                     CLIENT (Browser)                        │
│                                                             │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐  ┌──────────┐  │
│  │  D-Pad   │  │  Voice   │  │  Gesture  │  │  Auto    │  │
│  │  (HTML)  │  │  (JS)    │  │ (MediaPipe│  │  Drive   │  │
│  └────┬─────┘  └────┬─────┘  └─────┬─────┘  └────┬─────┘  │
│       └─────────────┴───────────────┴──────────────┘        │
│                           │ fetch /control /auto /snapshot  │
└───────────────────────────┼─────────────────────────────────┘
                            │ HTTP (LAN / Ngrok HTTPS)
┌───────────────────────────┼─────────────────────────────────┐
│              RASPBERRY PI 4 — Flask Server                  │
│                           │                                 │
│                    ┌──────▼──────┐                          │
│                    │   app.py    │                          │
│                    │  Flask 3.0  │                          │
│                    └──────┬──────┘                          │
│                           │ PyO3 FFI                        │
│                    ┌──────▼──────┐                          │
│                    │ zeroclaw_   │                          │
│                    │   core      │  ← Rust (Safety Engine)  │
│                    │  (lib.rs)   │                          │
│                    └──────┬──────┘                          │
│                           │ /dev/ttyACM0 (9600 baud)        │
└───────────────────────────┼─────────────────────────────────┘
                            │ USB Serial
┌───────────────────────────┼─────────────────────────────────┐
│                  ARDUINO UNO R3                             │
│                           │                                 │
│              ┌────────────┴──────────────┐                  │
│              │       ZeroClaw.ino        │                  │
│              │   State Machine (C++)     │                  │
│              └──┬──────────┬──────────┬──┘                  │
│                 │          │          │                      │
│           ┌─────▼──┐  ┌───▼───┐  ┌───▼───┐                 │
│           │ L298N  │  │HC-SR04│  │ SG90  │                 │
│           │ Motors │  │ Sonar │  │ Servo │                 │
│           └────────┘  └───────┘  └───────┘                 │
└─────────────────────────────────────────────────────────────┘
```

---

## Layer 1 — Frontend (Browser)

### Mô hình module JS

Bốn file JS load theo thứ tự nghiêm ngặt, giao tiếp qua `window` object:

```
index.html
  │
  ├── api.js          (script)        → định nghĩa window.AppState, cmd(), toggleAuto()
  ├── ui.js           (script)        → định nghĩa appendLogRaw(), polling, keyboard
  ├── voice.js        (script IIFE)   → dùng window.cmd(), window.appendLogRaw()
  └── hand_tracking.js (script module) → dùng window.cmd(), window.toggleAuto()
```

**Lý do dùng `window.AppState` thay vì module import:**  
`hand_tracking.js` là ESM module (bắt buộc vì MediaPipe dùng ESM import). ESM module không thể import từ classic script. Giải pháp: dùng `window` làm shared namespace giữa hai loại script.

### Gesture Detection Pipeline

```
Webcam frame (30fps)
       │
       ▼
MediaPipe GestureRecognizer
  (GPU delegate, VIDEO mode)
       │
       ├── Landmarks [0..20]
       │
       ▼ Priority pipeline:
  P1: isOKSign(lm)          → geometry: dist(tip4, tip8) < 0.05
  P2: isVictorySign(lm)     → geometry: index+middle extended, ring+pinky folded
  P3: Thumb_Up + direction  → geometry: dx(tip4, mcp5)
  P4: ML model labels       → ILoveYou, Closed_Fist, Pointing_Up
       │
       ▼
  updateGestureUI(name)
       │
       ├── Per-gesture cooldown:
       │     Movement gestures : 500ms
       │     ILoveYou          : 2000ms  (tránh spam toggleAuto)
       │     Victory/Snapshot  : 3000ms  (tránh spam /snapshot)
       │
       └── dispatch: cmd() hoặc triggerSnapshot() hoặc toggleAuto()
```

**Lý do dùng geometry thay vì chỉ dùng ML label:**  
- Model hay nhầm `Open_Palm` ↔ `OK_Sign` (đều có ngón tay mở)
- Model hay nhầm `Victory` ↔ `Open_Palm` khi ngón cái duỗi
- Geometry check nhanh hơn (O(1) so sánh float) và deterministic

### Voice Control Priority

```
Transcript → toLowerCase()
       │
       ├── Camera servo keywords (check TRƯỚC)
       │     "nhìn trái/phải/thẳng" → cmd('1'/'2'/'3')
       │
       └── Movement keywords (check SAU)
             "tiến/lùi/trái/phải/dừng" → cmd('F'/'B'/'L'/'R'/'S')
```

**Lý do camera check trước:**  
"Nhìn trái" chứa từ "trái" → nếu check movement trước sẽ bị nhầm thành `cmd('L')`.

---

## Layer 2 — Backend (Flask + Rust)

### Serial Ownership Model

```
TRƯỚC (buggy):
  Python (pyserial) ──┐
                      ├── /dev/ttyACM0  → "Device busy" error
  Rust (serialport) ──┘

SAU (correct):
  Python (Flask) → PyO3 → Rust → /dev/ttyACM0
                                  (Rust là owner duy nhất)
```

### Rust ZeroClaw — Thread Model

```
ZeroClaw::new()
       │
       ├── port_writer: Arc<Mutex<Box<dyn SerialPort>>>
       │     └── dùng bởi: send_command() (Flask thread)
       │
       ├── latest_distance: Arc<Mutex<f32>>    (init: 999.0)
       │     └── viết bởi: reader_thread
       │     └── đọc bởi: send_command(), get_distance()
       │
       ├── latest_mode: Arc<Mutex<String>>     (init: "MANUAL")
       │     └── viết bởi: reader_thread
       │     └── đọc bởi: is_auto_mode()
       │
       └── thread::spawn(reader_thread)
             │
             └── BufReader::read_line() loop
                   │
                   ├── "MODE:AUTO"  → latest_mode = "AUTO"
                   ├── "MODE:MANUAL"→ latest_mode = "MANUAL"
                   ├── "25.3"       → latest_distance = 25.3 (nếu 2.0..=400.0)
                   └── "READY", rác → bỏ qua
```

**Lý do dùng `BufReader::read_line()` thay vì `port.read()`:**  
`port.read()` đọc bất kỳ bytes nào có trong buffer — có thể nhận nửa dòng `"23."` hoặc `"MODE:AU"`. `read_line()` đợi đến `\n` nên luôn nhận dữ liệu hoàn chỉnh.

### Safety Reflex Flow

```
Flask receives GET /control/F
       │
       └── robot.send_command("F")   [PyO3 call]
                  │
                  ├── Read cache: dist = *latest_distance.lock()
                  │
                  ├── dist < 15.0?
                  │     YES → final_cmd = "S"
                  │           log: "🚨 Safety Alert: Vật cản Xcm"
                  │     NO  → final_cmd = "F"
                  │
                  └── port_writer.lock().write_all(final_cmd)
```

**Lý do Safety nằm trong Rust, không phải Python:**  
- Rust xử lý trước khi bytes rời khỏi process → không có race condition
- Nếu Safety nằm trong Python: Flask có thể trả về response 200 trước khi check distance
- Memory safety: `Arc<Mutex<T>>` đảm bảo không có data race giữa reader thread và Flask threads

### Snapshot vs MJPEG

```
MJPEG (cũ):
  Pi Camera → generate_camera_stream() → yield frame/30fps
  → Browser nhận ~30 requests/giây
  → Tốn bandwidth, CPU Pi liên tục

Snapshot (mới):
  Victory ✌ detected → fetch('/snapshot')
  → camera.read() × 1 → JPEG response
  → Browser nhận 1 request/lần chụp (cooldown 3s)
  → Tiết kiệm ~99% bandwidth camera
```

---

## Layer 3 — Firmware (Arduino)

### Auto Drive State Machine

```
                    ┌─────────────────┐
                    │  AUTO_FORWARD   │◄────────────┐
                    │  (tiến / chậm)  │             │
                    └────────┬────────┘             │
                    dist < 20cm?                    │
                    YES ↓                           │
                    ┌────────▼────────┐             │
                    │  AUTO_STOPPING  │             │
                    │    200ms        │             │
                    └────────┬────────┘             │
                             ↓                      │
                    ┌────────▼────────┐             │
                    │  AUTO_BACKUP    │             │
                    │    500ms (lùi)  │             │
                    └────────┬────────┘             │
                             ↓                      │
                    ┌────────▼────────┐             │
                    │  AUTO_TURNING   │             │
                    │    600ms (rẽ)   │             │
                    └────────┬────────┘             │
                             ↓                      │
                    ┌────────▼────────┐             │
                    │  AUTO_RESUME    │─────────────┘
                    │    100ms pause  │
                    └─────────────────┘
```

**Lý do dùng millis() thay vì delay():**  
`delay()` block toàn bộ Arduino — `Serial.read()` không chạy trong thời gian đó. Nếu user gửi `'M'` (tắt auto) trong khi xe đang lùi 500ms, lệnh bị bỏ qua hoàn toàn. `millis()`-based state machine kiểm tra Serial mỗi iteration (~1ms).

### Serial Protocol

```
Pi → Arduino (commands):
  'F' = Forward       'B' = Backward
  'L' = Left          'R' = Right
  'S' = Stop
  'A' = Auto ON       'M' = Manual (Auto OFF)
  '1' = Servo Left    '2' = Servo Right    '3' = Servo Center

Arduino → Pi (telemetry, mỗi 200ms):
  "25.3\n"        = khoảng cách (cm)
  "MODE:AUTO\n"   = xác nhận bật auto
  "MODE:MANUAL\n" = xác nhận tắt auto
  "READY\n"       = khởi động xong
```

---

## Các quyết định thiết kế quan trọng

### 1. Tại sao Rust cho Serial, không phải Python?

| Tiêu chí | Python (pyserial) | Rust (serialport) |
|----------|------------------|-------------------|
| Thread safety | Cần GIL | `Arc<Mutex<T>>` native |
| Safety logic | Có thể bị bypass bởi race condition | Atomic, không thể bypass |
| Performance | GC overhead | Zero-cost abstraction |
| Serial owner | Có thể xung đột | Ownership model đảm bảo 1 owner |

### 2. Tại sao không dùng WebSocket?

Flask `threaded=True` + HTTP polling 500ms đủ cho latency điều khiển robot (< 50ms trên LAN). WebSocket tăng độ phức tạp (cần `flask-socketio` + eventlet) trong khi lợi ích không đáng kể cho use case này.

### 3. Tại sao MediaPipe chạy trên client, không phải Pi?

Pi 4 không đủ để chạy MediaPipe real-time (< 10fps). Client browser chạy được 25-30fps nhờ WebGL/GPU delegate. Đây cũng là lý do `mediapipe` bị xóa khỏi `requirements.txt`.

### 4. Tại sao gesture Victory không còn map servo center?

Hai lý do:
1. Victory → snapshot là tính năng unique, có visual feedback rõ ràng (flash + ảnh mới)
2. Servo center vẫn accessible qua phím `3` hoặc nút GIỮA trên UI — không mất tính năng

---

## Performance Characteristics

| Component | Metric | Value |
|-----------|--------|-------|
| Flask status polling | Interval | 500ms |
| Arduino distance send | Interval | 200ms |
| Rust safety check | Latency | < 1ms (cache read) |
| MediaPipe detection | Throughput | ~25-30fps (client GPU) |
| Snapshot cooldown | Min interval | 3s |
| Auto gesture cooldown | Min interval | 2s |
| Serial baud rate | Speed | 9600 baud |

---

## Security Considerations

- **Ngrok HTTPS tunnel**: Required for Web Speech API remote access
- **SECRET_KEY**: Stored in `.env`, không commit lên git
- **NGROK_AUTHTOKEN**: Stored in `.env`, không commit lên git
- **Serial validation**: Flask chỉ chấp nhận commands trong whitelist `{F, B, L, R, S, 1, 2, 3}`
- **No authentication**: Dashboard không có auth — chỉ phù hợp cho LAN/controlled environment

---

## Known Limitations

1. **Single-user**: Không có session management — nhiều client cùng gửi lệnh sẽ conflict
2. **HC-SR04 blind spot**: Cảm biến chỉ nhìn thẳng — vật cản từ bên cạnh không được detect
3. **Voice requires Chrome**: Web Speech API chưa được hỗ trợ trên Firefox/Safari
4. **Snapshot delay**: Pi Camera cần ~3 warm-up frames → ảnh đầu tiên có thể tối hơn
5. **Gesture sensitivity**: Lighting conditions ảnh hưởng đến accuracy của MediaPipe

---

## Dependency Graph

```
app.py
  ├── flask==3.0.0
  ├── flask-cors==4.0.0
  ├── python-dotenv==1.0.1
  ├── opencv-python-headless==4.8.1.78
  ├── maturin==1.4.0
  └── zeroclaw_core  (built từ core/ bằng maturin)
        ├── pyo3==0.21
        └── serialport==4.3.0

hand_tracking.js (ESM CDN)
  └── @mediapipe/tasks-vision@0.10.14
        └── gesture_recognizer.task (model, ~25MB, lazy loaded)

index.html (Google Fonts CDN)
  ├── Orbitron (headings)
  └── Share Tech Mono (body)
```