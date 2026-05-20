# AutoClaw 🤖

> **Autonomous Edge AI Robot** — Điều khiển đa phương thức: tay, giọng nói, cử chỉ tay và tự lái.  
> Đồ án môn học · Khoa Mạng Máy Tính · Đại học Công nghệ Thông tin UIT

---

## Giới thiệu

AutoClaw là robot tự hành được xây dựng trên nền tảng **Raspberry Pi 4** và **Arduino Uno R3**, với lõi an toàn viết bằng **Rust** (qua PyO3). Hệ thống hỗ trợ 4 phương thức điều khiển đồng thời:

- 🕹️ **Thủ công** — D-pad trên web dashboard
- 🎙️ **Giọng nói** — Web Speech API tiếng Việt
- ✋ **Cử chỉ tay** — MediaPipe Hand Gesture (xử lý hoàn toàn trên client)
- 🤖 **Tự lái** — State machine né vật cản với HC-SR04

---

## Tính năng nổi bật

| Tính năng | Mô tả |
|-----------|-------|
| **Safety Reflex** | Rust core tự động dừng xe khi phát hiện vật cản < 15 cm, bất kể lệnh nào được gửi |
| **Voice Control** | Nhận diện 9 lệnh tiếng Việt, ưu tiên camera servo trước di chuyển để tránh nhầm lẫn |
| **Hand Gesture** | 7 cử chỉ, geometry-based (không phụ thuộc 100% vào ML model), cooldown riêng từng cử chỉ |
| **Snapshot Camera** | Chụp ảnh từ Pi Camera khi giơ cử chỉ ✌ — không stream liên tục, tiết kiệm bandwidth |
| **Auto Drive** | Non-blocking state machine (millis-based), nhận lệnh dừng ngay lập tức |
| **Remote Access** | Ngrok HTTPS tunnel — hỗ trợ Web Speech API từ xa |

---

## Cấu trúc dự án

```
AutoClaw/
├── app.py                  # Flask backend (Python 3.13)
├── requirements.txt        # Python dependencies
├── .env                    # Config & secrets
├── ZeroClaw.ino            # Arduino firmware (C++)
│
├── core/                   # Rust Safety Engine
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs          # PyO3 · ZeroClaw struct · Background reader thread
│
├── templates/
│   └── index.html          # Dashboard UI (Jinja2)
│
└── static/
    ├── css/
    │   └── style.css       # Cyberpunk theme
    └── js/
        ├── api.js          # cmd(), toggleAuto(), AppState
        ├── ui.js           # Polling, keyboard shortcuts, DOM updates
        ├── voice.js        # Web Speech API (vi-VN)
        └── hand_tracking.js # MediaPipe ESM module
```

---

## Hardware

| Linh kiện | Vai trò |
|-----------|---------|
| Raspberry Pi 4 (8GB) | Web server, camera, AI processing |
| Arduino Uno R3 | Motor control, sensor reading |
| L298N Motor Driver | Điều khiển 4 động cơ DC |
| HC-SR04 | Cảm biến siêu âm đo khoảng cách |
| SG90 Servo | Xoay camera trái/phải/giữa |
| Pi Camera | Chụp snapshot khi nhận cử chỉ ✌ |

### Sơ đồ chân Arduino

```
Motor Left  : ENA=5  IN1=6  IN2=7
Motor Right : ENB=10 IN3=8  IN4=9
Ultrasonic  : TRIG=12  ECHO=13
Servo       : PIN=11
Serial      : TX=1  RX=0  (9600 baud)
```

---

## Cài đặt

### Yêu cầu

- Python 3.13+
- Rust + Cargo ([rustup.rs](https://rustup.rs))
- Arduino IDE hoặc CLI

### 1. Clone và cài Python dependencies

```bash
git clone https://github.com/your-repo/autoclaw.git
cd autoclaw
pip install -r requirements.txt
```

### 2. Build Rust Core (bắt buộc)

```bash
cd core
maturin develop          # development
# hoặc
maturin develop --release  # production (nhanh hơn ~3x)
cd ..
```

> ⚠️ Nếu chưa có `maturin`: `pip install maturin`

### 3. Cấu hình `.env`

```bash
cp .env.example .env
# Chỉnh sửa:
# SERIAL_PORT=/dev/ttyACM0
# FLASK_PORT=5000
# NGROK_AUTHTOKEN=your_token_here
```

### 4. Nạp firmware lên Arduino

Mở `ZeroClaw.ino` bằng Arduino IDE và upload lên Arduino Uno R3.

> ⚠️ Rút dây TX/RX (pin 0, 1) trước khi upload, cắm lại sau khi xong.

### 5. Chạy server

```bash
python app.py
```

Mở trình duyệt: `http://<IP_Raspberry_Pi>:5000`

---

## Remote Access (Ngrok)

Web Speech API yêu cầu HTTPS. Dùng Ngrok để tạo tunnel:

```bash
ngrok http 5000
# hoặc với static domain:
ngrok http --domain=your-domain.ngrok-free.app 5000
```

Hoặc nếu dùng HTTP local, bật trong Chrome:

```
chrome://flags → "Insecure origins treated as secure"
→ Thêm: http://<IP_Pi>:5000
```

---

## Điều khiển

### Phím tắt bàn phím

| Phím | Lệnh |
|------|------|
| `W` / `↑` | Tiến |
| `S` / `↓` | Lùi |
| `A` / `←` | Rẽ trái |
| `D` / `→` | Rẽ phải |
| `X` / `Space` | Dừng |
| `P` | Bật/tắt tự lái |
| `V` | Bật/tắt nhận diện giọng nói |
| `1` `2` `3` | Servo: Trái / Phải / Giữa |

### Cử chỉ tay (MediaPipe)

| Cử chỉ | Lệnh |
|--------|------|
| 👌 OK Sign | Tiến |
| ✊ Nắm đấm | Dừng |
| ☝ Chỉ lên | Lùi |
| ✌ Chữ V | Chụp ảnh Pi Camera |
| 👈 Ngón cái trái | Rẽ trái |
| 👉 Ngón cái phải | Rẽ phải |
| 🤟 I Love You | Bật/tắt tự lái |

### Lệnh giọng nói (vi-VN)

```
"tiến" / "đi thẳng" / "tới"     → Tiến
"lùi" / "xuống"                  → Lùi
"rẽ trái" / "quay trái"          → Rẽ trái
"rẽ phải" / "quay phải"          → Rẽ phải
"dừng" / "đứng lại" / "stop"     → Dừng
"nhìn trái" / "nhìn sang trái"   → Servo trái
"nhìn phải" / "nhìn sang phải"   → Servo phải
"nhìn thẳng" / "nhìn giữa"       → Servo giữa
"tự lái" / "tự động"             → Toggle Auto Drive
```

---

## API Endpoints

| Method | Endpoint | Mô tả |
|--------|----------|-------|
| `GET` | `/` | Dashboard UI |
| `GET` | `/control/<cmd>` | Gửi lệnh (`F B L R S 1 2 3`) |
| `GET` | `/auto/<state>` | Bật/tắt auto (`on` / `off`) |
| `GET` | `/status` | Distance (cm) + auto mode state |
| `GET` | `/snapshot` | Chụp 1 frame từ Pi Camera |

---

## Troubleshooting

**Arduino không kết nối được:**
```bash
ls /dev/ttyACM*      # Tìm đúng port
sudo usermod -aG dialout $USER  # Cấp quyền
```

**Rust Core không build được:**
```bash
rustup update
pip install maturin --upgrade
cd core && maturin develop
```

**Voice API lỗi "not-allowed":**  
→ Dùng HTTPS (Ngrok) hoặc bật `chrome://flags` như hướng dẫn ở trên.

**Khoảng cách hiển thị `---`:**  
→ Kiểm tra dây TRIG (pin 12) và ECHO (pin 13) của HC-SR04.


---

## License

MIT License — Xem [LICENSE](LICENSE) để biết thêm.