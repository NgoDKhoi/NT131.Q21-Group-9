# AutoClaw 🤖

> **Autonomous Edge AI Robot** — Xe tự hành thông minh tích hợp điều khiển đa phương thức: tay (D-pad), giọng nói (tiếng Việt), cử chỉ bàn tay (AI MediaPipe), thuật toán tự động tránh vật cản và giám sát/điều khiển từ xa qua **Telegram Bot**.
>
> 🏫 Đồ án môn học · Khoa Mạng Máy Tính & Truyền Thông · Đại học Công nghệ Thông tin UIT

---

## 📖 Giới thiệu dự án

AutoClaw là một dự án nghiên cứu và phát triển xe robot tự hành ứng dụng trí tuệ nhân tạo biên (Edge AI) xây dựng trên nền tảng bộ kit xe thông minh **ELEGOO Smart Robot Car Kit** kết hợp mạch điều khiển trung tâm **Raspberry Pi 4** và **Arduino Uno R3**. Hệ thống hỗ trợ 4 phương thức tương tác độc lập và đồng thời:

*   🕹️ **Thủ công** — Điều khiển qua nút bấm D-pad trên web dashboard.
*   🎙️ **Giọng nói** — Điều khiển rẽ hướng, di chuyển và xoay góc camera bằng khẩu lệnh tiếng Việt (sử dụng Web Speech API).
*   ✋ **Cử chỉ tay** — Nhận diện cử chỉ bàn tay bằng mô hình AI MediaPipe (xử lý trực tiếp trên GPU Client qua trình duyệt).
*   🤖 **Tự lái** — Thuật toán tránh vật cản chủ động sử dụng cảm biến siêu âm HC-SR04 kết hợp động cơ Servo SG90 để quét 180 độ.

---

## ✨ Tính năng nổi bật

*   🛡️ **Safety Reflex (Phanh khẩn cấp)**: Lõi an toàn bằng **Rust** tự động kiểm tra khoảng cách và chặn các lệnh Tiến (`F`), ép dừng xe ngay lập tức khi phát hiện vật cản $< 15\text{ cm}$ để tránh va chạm.
*   🤖 **Lõi Tự Trị AutoClaw Agent**: Chạy bằng vòng lặp Tokio không đồng bộ (Rust). Khi kích hoạt chế độ lái tự động AI (`/auto/ai`), server Flask chuyển quyền quyết định hoàn toàn cho Rust Agent để gọi các công cụ AI (Tools): đo khoảng cách, chụp ảnh snapshot và đề xuất hướng lái tối ưu tránh vật cản.
*   📸 **Snapshot Camera**: Tiết kiệm 99% băng thông của Raspberry Pi bằng cách chụp và gửi ảnh camera tĩnh khi phát hiện cử chỉ Victory (✌), thay vì truyền phát (stream) video MJPEG liên tục gây nghẽn mạng.
*   ⏱️ **Non-blocking State Machine**: Hệ thống lái tự động trên Arduino sử dụng biến mốc thời gian `millis()` thay cho hàm chặn `delay()`, giúp xe luôn sẵn sàng nhận lệnh ngắt thủ công từ người dùng ngay tức khắc.
*   ⚡ **Bộ lọc chống nhiễu cảm biến**: Thuật toán trên Arduino tự động lọc nhiễu tín hiệu siêu âm, xe chỉ phản ứng dừng khi có vật cản xuất hiện trong ít nhất 2 chu kỳ đo liên tiếp (~100ms).

---

## 🛠️ Phần cứng & Sơ đồ đấu nối Arduino

### Linh kiện sử dụng
1.  **Raspberry Pi 4 (8GB)**: Đóng vai trò máy chủ Flask Web Server, xử lý camera Snapshot và điều phối lệnh.
2.  **Arduino Uno R3**: Vi xử lý cấp thấp chịu trách nhiệm đọc cảm biến, quét servo và điều khiển trực tiếp động cơ.
3.  **Module L298N**: Mạch cầu H để điều khiển tốc độ và hướng của 4 động cơ DC.
4.  **Cảm biến siêu âm HC-SR04**: Đo khoảng cách tới chướng ngại vật trước mặt.
5.  **Động cơ Servo SG90**: Xoay cảm biến siêu âm và camera sang Trái/Phải/Giữa.
6.  **Raspberry Pi Camera Module (hoặc USB Webcam)**: Ghi hình môi trường thực tế trước xe để gửi dữ liệu về cho Gemini Vision AI phân tích.

Motor Trái   : ENA = chân 5  | IN1 = chân 7  | IN2 = chân 8
Motor Phải   : ENB = chân 6  | IN3 = chân 9  | IN4 = chân 11
Cảm biến âm  : TRIG = chân A5 | ECHO = chân A4
Động cơ Servo: PIN = chân 10
Giao tiếp    : Cổng USB Serial (9600 baud, chân TX=1, RX=0)
```

---

## 📂 Cấu trúc thư mục

```
AutoClaw/
├── app.py                  # Flask Web Server (Python)
├── requirements.txt        # Các thư viện Python cần cài đặt
├── .env                    # Lưu trữ cấu hình cổng COM/Serial, Camera Index, Port Web, Key bảo mật, Token API và Telegram Bot.
├── AutoClaw.cpp            # Firmware Arduino Uno (C++)
├── autoclaw_mock.py        # Giả lập xe AutoClaw (Mock Car) khi chạy Localhost
│
├── core/                   # Lõi an toàn Rust Safety Engine & AutoClaw Agent
│   ├── Cargo.toml          # Khai báo thư viện phụ thuộc của Rust (bao gồm các thư viện pyo3, serialport, tokio...)
│   └── src/
│       ├── lib.rs          # PyO3 Bindings, reader thread đọc Serial, APIs start_agent/stop_agent
│       ├── gemini.rs       # Module Vision AI gọi API Gemini 2.0
│       └── agent.rs        # Loop agent tuần tra tự trị (Tokio loop, chứa ControlCarTool, GetDistanceTool, CaptureSnapshotTool)
│
├── templates/
│   └── index.html          # Giao diện điều khiển Web Dashboard (HTML)
│
└── static/
    ├── css/
    │   └── style.css       # CSS thiết kế giao diện Cyberpunk neon retro
    └── js/
        ├── api.js          # Khai báo namespace AppState và hàm gửi API lệnh
        ├── ui.js           # Polling cập nhật UI, phím tắt keyboard, hiển thị log
        ├── voice.js        # Nhận dạng giọng nói Web Speech API tiếng Việt
        └── hand_tracking.js# AI MediaPipe nhận diện cử chỉ bàn tay qua webcam
```

---

## 🚀 Hướng dẫn cài đặt và vận hành

### Yêu cầu hệ thống
*   **Hệ điều hành**: Raspberry Pi OS (Debian 12 Bookworm / Trixie hoặc mới hơn) trên Pi 4 / Windows 10/11 trên PC.
*   **Python**: Phiên bản 3.10 đến 3.13.
*   **Rust & Cargo**: Để biên dịch lõi an toàn `autoclaw_core`.
*   **Arduino IDE**: Để nạp chương trình lên Arduino Uno R3.
*   **Thư viện hệ thống (Linux/Pi)**: `libssl-dev`, `pkg-config`, `libudev-dev`, `libopenblas-dev` (xem chi tiết ở Bước 5).

---

### 🍓 Hướng dẫn 1: Triển khai thực tế trên Raspberry Pi 4 (Ưu tiên)

Triển khai thực tế trên xe robot thông qua sự kết hợp giữa **Raspberry Pi 4** (làm máy chủ web và AI) và **Arduino Uno R3** (điều khiển phần cứng). Quy trình lắp đặt và cấu hình cụ thể như sau:

---

#### 🔌 GIAI ĐOẠN A: CHUẨN BỊ PHẦN CỨNG & NẠP CODE ARDUINO

##### Bước 1: Lắp ráp bộ kit xe ELEGOO
* Lắp ráp động cơ, khung gầm, mạch cầu H L298N, cảm biến siêu âm HC-SR04 và servo SG90 theo tài liệu hướng dẫn của bộ kit xe thông minh ELEGOO.
* Kết nối các dây điều khiển động cơ và cảm biến vào các chân cắm trên Arduino Uno R3 theo đúng **Sơ đồ chân cắm** ở phần [Phần cứng & Sơ đồ đấu nối Arduino](#️-phần-cứng--sơ-đồ-đấu-nối-arduino) phía trên.

##### Bước 2: Nạp firmware cho Arduino Uno R3 (trên máy tính cá nhân)
1. Kết nối mạch Arduino Uno R3 vào máy tính cá nhân (PC/Laptop) của bạn bằng cáp USB.
2. Mở file [AutoClaw.cpp](AutoClaw.cpp) bằng công cụ **Arduino IDE**.
3. **Quan trọng**: Rút dây kết nối RX/TX (chân 0, 1) trên mạch Arduino ra trước khi nạp để tránh lỗi xung đột cổng Serial.
4. Trên Arduino IDE, chọn đúng loại Board mạch là **Arduino Uno** và cổng COM kết nối tương ứng.
5. Tiến hành biên dịch và nạp chương trình (nút **Upload**). Sau khi nạp thành công, cắm lại dây kết nối RX/TX vào các chân 0, 1.

##### Bước 3: Kết nối vật lý Arduino Uno với Raspberry Pi 4
1. Rút cáp kết nối USB của Arduino Uno ra khỏi máy tính cá nhân.
2. Cắm cáp USB này từ Arduino Uno vào một trong các cổng USB của mạch **Raspberry Pi 4**.
3. Gắn camera (Raspberry Pi Camera Module hoặc USB Webcam) vào Pi 4.
4. Cấp nguồn để khởi động xe robot và mạch Raspberry Pi 4.

---

#### 💻 GIAI ĐOẠN B: TRIỂN KHAI PHẦN MỀM TRÊN RASPBERRY PI 4

##### Bước 4: Kết nối SSH vào Raspberry Pi 4
1. Đảm bảo Raspberry Pi 4 và máy tính cá nhân của bạn đang kết nối chung một mạng Wi-Fi (hoặc mạng nội bộ LAN).
2. Tìm địa chỉ IP của Raspberry Pi 4 trong mạng (có thể quét qua phần mềm quét IP hoặc xem trên router).
3. Mở terminal trên máy tính cá nhân và kết nối SSH sang Pi 4:
   ```bash
   ssh pi@<IP_RASPBERRY_PI>
   ```
   *(Thay đổi `pi` thành username của Pi nếu bạn đặt tên khác).*

##### Bước 5: Cài đặt các thư viện hệ thống cần thiết qua APT
Sau khi đã kết nối SSH thành công vào Pi 4, chạy lệnh cài đặt các package phát triển cần thiết cho OpenCV, OpenSSL (yêu cầu bởi Rust crate `reqwest`), libudev (yêu cầu bởi crate `serialport`) và các dependencies khác:
```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev libudev-dev libopenblas-dev libjpeg-dev libtiff-dev libpng-dev
```
> **Lưu ý:** `libssl-dev` + `pkg-config` là **bắt buộc** cho crate `reqwest` → `openssl-sys`. `libudev-dev` là **bắt buộc** cho crate `serialport` → `libudev-sys`. Nếu thiếu bất kỳ package nào, `maturin develop` sẽ báo lỗi build.

##### Bước 6: Tải mã nguồn và thiết lập Môi trường ảo Python (venv)
Để tránh lỗi chặn cài thư viện toàn cục (`externally-managed-environment`) trên Debian, khởi tạo môi trường ảo Python trong thư mục dự án:
```bash
git clone https://github.com/NgoDKhoi/NT131.Q21-Group-9.git
cd NT131.Q21-Group-9/AutoClaw

# Khởi tạo và kích hoạt môi trường ảo
python3 -m venv venv
source venv/bin/activate
```

##### Bước 7: Cài đặt các dependencies Python
Nâng cấp công cụ pip và cài đặt toàn bộ thư viện:
```bash
pip install --upgrade pip
pip install -r requirements.txt
```

##### Bước 8: Cài đặt Rust Compiler và biên dịch Rust Core
1. Cài đặt toolchain Rust trên Pi 4:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
   *Chọn tùy chọn `1` (mặc định) và đợi quá trình cài đặt hoàn tất.*
2. Cập nhật biến môi trường cho terminal hiện tại:
   ```bash
   source $HOME/.cargo/env
   ```
3. **Kiểm tra lại** đã cài đủ thư viện hệ thống (Bước 5), đặc biệt `libssl-dev` và `pkg-config`:
   ```bash
   pkg-config --libs openssl   # Phải trả về "-lssl -lcrypto", nếu báo lỗi thì quay lại Bước 5
   ```
4. Biên dịch lõi an toàn `autoclaw_core` bằng Maturin:
   ```bash
   cd core
   maturin develop --release
   cd ..
   ```

##### Bước 9: Cấu hình cổng kết nối và môi trường trên Pi
Sao chép file cấu hình mẫu `.env.example` thành `.env` và điền thông số thực tế của bạn:
```bash
cp .env.example .env
```
Sử dụng trình soạn thảo văn bản (ví dụ `nano .env`) để điền `GEMINI_API_KEY`, `TELEGRAM_BOT_TOKEN` và `TELEGRAM_CHAT_ID` để kích hoạt đầy đủ tính năng AI và Telegram Bot.

##### Bước 10: Khởi chạy và vận hành
Kích hoạt máy chủ điều khiển trên Pi:
```bash
python app.py
```
* Server sẽ lắng nghe tại cổng `5000`. Bạn mở trình duyệt trên điện thoại hoặc máy tính trong mạng LAN và truy cập: `http://<IP_RASPBERRY_PI>:5000`.
* *Lưu ý: Để sử dụng các tính năng Camera tay và Nhận diện giọng nói từ xa (yêu cầu kết nối HTTPS bảo mật), hãy xem thêm mục **Cấu hình Ngrok để truy cập qua HTTPS** ở phía dưới.*

---

### 💻 Hướng dẫn 2: Triển khai nhanh trên Windows/PC (Localhost - Giả lập)

Để chạy thử nghiệm giao diện web và thuật toán trên máy tính cá nhân (sử dụng camera webcam giả lập và serial giả lập):

1. **Tải mã nguồn**:
   ```bash
   git clone https://github.com/NgoDKhoi/NT131.Q21-Group-9.git
   cd NT131.Q21-Group-9/AutoClaw
   ```
2. **Cài đặt thư viện Python**:
   ```bash
   pip install -r requirements.txt
   ```
3. **Biên dịch Rust Core** (Yêu cầu đã cài đặt Rust từ trước):
   ```bash
   cd core
   maturin develop --release
   cd ..
   ```
   *Mẹo: Nếu chưa có Rust trên Windows, backend sẽ tự động chuyển sang chế độ giả lập (`autoclaw_mock.py`) để kiểm thử UI mà không bị crash.*
4. **Tạo cấu hình môi trường**:
   Sao chép file cấu hình mẫu `.env.example` thành `.env` và điền các thông tin của bạn (đặc biệt là API key Gemini và Token Bot Telegram nếu muốn dùng các tính năng AI & điều khiển từ xa):
   * Trên Windows (PowerShell):
     ```powershell
     copy .env.example .env
     ```
   * Hoặc tạo thủ công file `.env` từ nội dung mẫu sau:
     ```env
     SERIAL_PORT=COM3 # Thay đổi thành cổng COM thực tế của bạn
     CAMERA_INDEX=0
     GEMINI_API_KEY=YOUR_GEMINI_API_KEY_HERE # Điền API Key Gemini của bạn để chạy AI
     TELEGRAM_BOT_TOKEN=YOUR_TELEGRAM_BOT_TOKEN_HERE
     TELEGRAM_CHAT_ID=YOUR_TELEGRAM_CHAT_ID_HERE
     ```
5. **Chạy server**:
   ```bash
   python app.py
   ```
   Truy cập `http://localhost:5000` trên trình duyệt.

---

## 🎮 Hướng dẫn điều khiển

### 1. Phím tắt bàn phím
*   `W` hoặc `↑`: Đi tiến
*   `S` hoặc `↓`: Đi lùi
*   `A` hoặc `←`: Rẽ trái
*   `D` hoặc `→`: Rẽ phải
*   `X` hoặc `Phím Cách (Space)`: Dừng xe khẩn cấp
*   `P`: Bật / Tắt chế độ tự lái (Auto Drive)
*   `V`: Bật / Tắt nhận diện giọng nói
*   `G`: Kích hoạt Trợ lý AI Phân tích (Gemini Vision) mô tả cảnh vật và gợi ý hướng rẽ nhấp nháy trên nút điều hướng
*   `1` / `2` / `3`: Xoay camera (Trái / Phải / Giữa)

### 2. Khẩu lệnh giọng nói (Tiếng Việt)
*   *"tiến" / "đi thẳng" / "tới"* $\rightarrow$ Tiến
*   *"lùi" / "xuống"* $\rightarrow$ Lùi
*   *"rẽ trái" / "quay trái"* $\rightarrow$ Rẽ trái
*   *"rẽ phải" / "quay phải"* $\rightarrow$ Rẽ phải
*   *"dừng" / "đứng lại" / "stop"* $\rightarrow$ Dừng xe
*   *"nhìn trái" / "nhìn phải" / "nhìn thẳng"* $\rightarrow$ Xoay camera tương ứng
*   *"tự lái" / "tự động"* $\rightarrow$ Bật/Tắt chế độ tự lái
*   *"ai phân tích" / "ai xem trước mặt" / "quét vật cản"* $\rightarrow$ Gọi trợ lý AI phân tích bằng camera xe và gợi ý hướng đi

### 3. Cử chỉ bàn tay (MediaPipe)
*   👌 **OK Sign**: Đi tiến
*   ✊ **Closed Fist (Nắm đấm)**: Dừng lại
*   ☝ **Pointing Up (Ngón trỏ chỉ lên)**: Đi lùi
*   👈 **Thumb Left (Ngón cái trái)**: Rẽ trái
*   👉 **Thumb Right (Ngón cái phải)**: Rẽ phải
*   🤟 **I Love You**: Bật/Tắt tự lái
*   ✌ **Victory (Chữ V)**: Gọi trợ lý AI chụp ảnh, mô tả vật cản bằng tiếng Việt (dưới 15 từ) và đề xuất hướng di chuyển nhấp nháy trên giao diện
*   *Lưu ý: Không dùng MJPEG stream liên tục mà chỉ gửi 1 ảnh khi có cử chỉ hoặc khẩu lệnh để tiết kiệm tối đa token.*

---

## 🌐 Các API Endpoints của Flask Server

Flask Web Server cung cấp các API RESTful sau để frontend hoặc các ứng dụng bên thứ ba giao tiếp và điều khiển robot:

| Endpoint | Phương thức | Tham số | Mô tả | Định dạng phản hồi |
| :--- | :--- | :--- | :--- | :--- |
| `/` | `GET` | Không | Trả về trang Web Dashboard chính. | HTML |
| `/control/<cmd>` | `GET` | `cmd`: `F` (Tiến), `B` (Lùi), `L` (Trái), `R` (Phải), `S` (Dừng), `1` (Xoay servo trái), `2` (Xoay servo phải), `3` (Xoay servo giữa) | Gửi lệnh di chuyển hoặc điều khiển góc quay servo camera xuống Arduino. *Lưu ý: Lệnh `F` sẽ bị bộ lọc Safety Reflex chặn và chuyển thành `S` nếu khoảng cách vật cản < 15cm.* | `{"status": "success", "command": cmd}` |
| `/auto/<state>` | `GET` | `state`: `off` (Tắt tự lái), `cpp` (Tự lái tránh vật cản C++ trên Arduino), `ai` (AI Agent tự lái sử dụng Gemini Vision) | Thiết lập chế độ lái tự động cho robot. Khi chuyển sang `ai`, Flask sẽ kích hoạt AutoClaw Agent chạy bằng Rust (gọi `robot.start_agent(api_key)`), chạy ngầm một vòng lặp quyết định Tokio. Khi tắt hoặc đổi chế độ, Flask sẽ gọi `robot.stop_agent()`. | `{"status": "success", "mode": state}` |
| `/status` | `GET` | Không | Polling thông tin trạng thái xe (khoảng cách hiện tại, chế độ tự lái và log phân tích của AI). Được gọi bởi frontend mỗi 500ms. | `{"distance": "25.4" (hoặc "---"), "auto": "off"/"cpp"/"ai", "ai_log": "..."}` |
| `/snapshot` | `GET` | Không | Chụp 1 khung hình JPEG tĩnh từ camera Pi trên xe theo yêu cầu. Tự động trả về ảnh giả lập Cyberpunk nếu không kết nối camera. | Ảnh JPEG (mimetype: `image/jpeg`) |
| `/ai_analyze` | `GET` | Không | Chụp ảnh từ camera, mã hóa Base64 và gửi lên Rust Core để phân tích vật cản và lấy gợi ý hướng đi bằng Gemini API. | `{"status": "success", "image": "data:image/jpeg;base64,...", "description": "...", "command": "..."}` |

---

## 🤖 Telegram Bot — Giám sát & Điều khiển từ xa

AutoClaw tích hợp sẵn **Telegram Bot** cho phép bạn giám sát và điều khiển xe robot từ bất cứ đâu qua ứng dụng Telegram trên điện thoại.

### Thiết lập Telegram Bot:

1. **Tạo Bot**: Mở Telegram, tìm [@BotFather](https://t.me/BotFather), gửi lệnh `/newbot` và làm theo hướng dẫn. Sao chép **Bot Token** được cung cấp.
2. **Lấy Chat ID**: Gửi tin nhắn bất kỳ cho bot, sau đó truy cập `https://api.telegram.org/bot<TOKEN>/getUpdates` để lấy `chat.id` của bạn.
3. **Cấu hình `.env`**:
   ```env
   TELEGRAM_BOT_TOKEN=<Bot Token từ BotFather>
   TELEGRAM_CHAT_ID=<Chat ID của bạn>
   ```
4. **Khởi động lại server** (`python app.py`). Bot sẽ tự động chạy ngầm.

### Các lệnh Telegram Bot:

| Lệnh | Mô tả | Yêu cầu Chat ID |
| :--- | :--- | :---: |
| `/start` hoặc `/help` | Hiển thị hướng dẫn sử dụng | Không |
| `/status` | Xem khoảng cách cảm biến và chế độ lái hiện tại | Không |
| `/snapshot` | Chụp ảnh thời gian thực từ camera xe | Không |
| `/control <F/B/L/R/S>` | Di chuyển xe thủ công (qua Safety Reflex) | **Có** |
| `/auto <off/cpp/ai>` | Chuyển chế độ lái tự động | **Có** |

> [!NOTE]
> **Bảo mật**: Nếu đã cấu hình `TELEGRAM_CHAT_ID`, chỉ tài khoản Telegram có Chat ID trùng khớp mới được phép sử dụng lệnh điều khiển (`/control`, `/auto`). Các lệnh xem trạng thái (`/status`, `/snapshot`) vẫn hoạt động công khai nếu không cấu hình Chat ID.

> [!WARNING]
> Nếu `TELEGRAM_BOT_TOKEN` chưa được cấu hình hoặc giữ giá trị mặc định placeholder, bot sẽ tự động bỏ qua mà không làm ảnh hưởng tới Flask server.

---

## 🔒 Cấu hình Ngrok để truy cập từ xa qua HTTPS

> [!IMPORTANT]
> Trình duyệt Chrome và Edge áp dụng chính sách bảo mật nghiêm ngặt đối với **Web Speech API** (Nhận diện giọng nói) và **Camera Webcam** (Nhận diện cử chỉ tay). Các API này **chỉ hoạt động** trên `localhost` hoặc kết nối bảo mật **HTTPS**.
> Khi triển khai dự án trên Raspberry Pi trong mạng LAN, việc truy cập bằng IP của Pi (ví dụ `http://192.168.1.50:5000`) từ điện thoại hoặc máy tính khác sẽ **không sử dụng được camera tay và giọng nói**. Bạn cần cấu hình đường truyền bảo mật HTTPS bằng **Ngrok**.

### Hướng dẫn thiết lập Ngrok Tunnel:

1. **Đăng ký tài khoản**: Truy cập [ngrok.com](https://ngrok.com/), đăng ký một tài khoản miễn phí và lấy mã thông báo xác thực (`Authtoken`).
2. **Cấu hình Authtoken**:
   - Nếu sử dụng ngrok CLI cài đặt trên hệ điều hành, chạy lệnh:
     ```bash
     ngrok config add-authtoken <MÃ_TOKEN_CỦA_BẠN>
     ```
   - Hoặc bạn có thể thêm cấu hình này vào file `.env` để quản lý tập trung:
     ```env
     NGROK_AUTHTOKEN=<MÃ_TOKEN_CỦA_BẠN>
     ```
3. **Khởi chạy đường truyền HTTPS**:
   Trong khi Flask Server đang chạy (ở port `5000`), mở một terminal mới và chạy lệnh:
   ```bash
   ngrok http 5000
   ```
4. **Truy cập dự án**:
   Ngrok sẽ cung cấp một đường dẫn công khai có dạng `https://xxxx-xxxx.ngrok-free.app`. Hãy sao chép liên kết này, mở trên trình duyệt điện thoại hoặc máy tính client của bạn. 
   - Hệ thống lúc này chạy qua HTTPS bảo mật nên trình duyệt sẽ cho phép yêu cầu quyền Micro và Camera.
   - Bạn có thể điều khiển xe robot từ xa qua internet mà không gặp bất cứ rào cản bảo mật nào!