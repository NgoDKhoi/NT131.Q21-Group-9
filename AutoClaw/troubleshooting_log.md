# Nhật ký Khắc phục Sự cố & Lỗi Kỹ thuật (Troubleshooting Log) 🛠️
*Dự án xe tự hành thông minh AutoClaw - Môn học Khoa Mạng Máy Tính & Truyền Thông (UIT)*

Tài liệu này ghi lại chi tiết các sự cố phần cứng, phần mềm và cấu hình hệ thống phát sinh trong quá trình triển khai dự án, cùng nguyên nhân gốc rễ và giải pháp xử lý triệt để đã áp dụng.

---

## 📁 Mục lục
1. [Sự cố 1: Lỗi biên dịch TOML Parser trên Linux (Raspberry Pi 4)](#sự-cố-1)
2. [Sự cố 2: Thiếu thư viện hệ thống C/C++ khi build Rust Core](#sự-cố-2)
3. [Sự cố 3: Lỗi không tương thích phiên bản Python và PyO3](#sự-cố-3)
4. [Sự cố 4: Lỗi động cơ hoạt động một bên (Tiến/Rẽ phải đứng yên)](#sự-cố-4)
5. [Sự cố 5: Cảm biến siêu âm HC-SR04 luôn trả về khoảng cách tối đa 999.0 cm](#sự-cố-5)

---

<a name="sự-cố-1"></a>
### 1. Sự cố 1: Lỗi biên dịch TOML Parser trên Linux (Raspberry Pi 4)
* **Triệu chứng:** Khi chạy `maturin develop --release` để biên dịch thư viện Rust Core trên Raspberry Pi, trình biên dịch báo lỗi cú pháp TOML:
  ```text
  Caused by: TOML parse error at line 25, column 16
    |
  25 | base64 = "0.21"
    |                ^
  expected newline, `#`
  ```
* **Nguyên nhân:** File `Cargo.toml` được chỉnh sửa trên môi trường Windows và lưu dưới định dạng ngắt dòng **CRLF** (`\r\n`). Khi chuyển sang Raspberry Pi (Linux), trình phân tích cú pháp TOML của Rust không xử lý được ký tự thừa `\r` (carriage return) ở cuối dòng 25 và báo lỗi.
* **Giải pháp:**
  * Chuyển định dạng ngắt dòng của file `Cargo.toml` sang **LF** (`\n`).
  * Tạo file cấu hình quản lý mã nguồn [.gitattributes](file:///c:/Users/khoi1/MyRepo/NT131.Q21-Group-9/.gitattributes) ở thư mục gốc để ép buộc Git luôn chuyển đổi tự động các file source code thành dạng ngắt dòng **LF** khi checkout trên môi trường Linux (Raspberry Pi).

---

<a name="sự-cố-2"></a>
### 2. Sự cố 2: Thiếu thư viện hệ thống C/C++ khi build Rust Core
* **Triệu chứng:** Quá trình biên dịch Rust Core báo thiếu các gói phát triển:
  * Lỗi 1: `Could not find openssl via pkg-config` (yêu cầu bởi crate `reqwest` / `openssl-sys`).
  * Lỗi 2: `The system library libudev required by crate libudev-sys was not found` (yêu cầu bởi crate `serialport`).
* **Nguyên nhân:** Hệ điều hành Raspberry Pi OS mặc định chỉ đi kèm các thư viện runtime thông thường, không tích hợp sẵn các gói header phát triển (`-dev`) và công cụ định vị thư viện `pkg-config`.
* **Giải pháp:** Cài đặt bổ sung các thư viện hệ thống qua APT trước khi biên dịch:
  ```bash
  sudo apt update
  sudo apt install -y pkg-config libssl-dev libudev-dev
  ```

---

<a name="sự-cố-3"></a>
### 3. Sự cố 3: Lỗi không tương thích phiên bản Python và PyO3
* **Triệu chứng:** Biên dịch bị dừng ở bước xử lý `pyo3-ffi` với thông báo lỗi:
  ```text
  error: the configured Python interpreter version (3.13) is newer than PyO3's maximum supported version (3.12)
  ```
* **Nguyên nhân:** Raspberry Pi OS thế hệ mới (Debian Trixie) mặc định đi kèm phiên bản **Python 3.13**. Trong khi đó, file cấu hình `Cargo.toml` ban đầu sử dụng thư viện kết nối **PyO3 v0.21** chỉ hỗ trợ tối đa đến Python 3.12.
* **Giải pháp:** Nâng cấp thư viện PyO3 trong file [Cargo.toml](file:///c:/Users/khoi1/MyRepo/NT131.Q21-Group-9/AutoClaw/core/Cargo.toml) lên **v0.22** (hỗ trợ đầy đủ Python 3.13) và cập nhật lại signature khởi tạo module của PyO3 trong file [lib.rs](file:///c:/Users/khoi1/MyRepo/NT131.Q21-Group-9/AutoClaw/core/src/lib.rs) từ API cũ sang API mới (`Bound<'_, PyModule>`).

---

<a name="sự-cố-4"></a>
### 4. Sự cố 4: Lỗi động cơ hoạt động một bên (Tiến/Rẽ phải đứng yên)
* **Triệu chứng:** Lệnh Lùi (B) và Rẽ Trái (L) hoạt động tốt. Tuy nhiên, lệnh Tiến (F) chỉ có 2 bánh bên phải tiến (bên trái đứng yên) và lệnh Rẽ Phải (R) chỉ có 2 bánh bên phải lùi.
* **Nguyên nhân:** Sơ đồ chân cắm logic động cơ trong code ban đầu không đồng nhất với sơ đồ đấu nối dây vật lý thực tế của kit xe **ELEGOO Smart Robot Car Kit V3.0**. Việc cấu hình nhầm chân cắm và đảo chiều điện áp khiến cầu H L298N kích hoạt sai trạng thái cho động cơ bên trái.
* **Giải pháp:** Đồng bộ hóa toàn bộ cấu hình chân cắm trong firmware [AutoClaw.cpp](file:///c:/Users/khoi1/MyRepo/NT131.Q21-Group-9/AutoClaw/AutoClaw.cpp) theo đúng tài liệu chuẩn của hãng ELEGOO:
  * `ENA` = chân **5** | `ENB` = chân **6**
  * `IN1` = chân **7** | `IN2` = chân **8**
  * `IN3` = chân **9** | `IN4` = chân **11**
  * Chuyển chân điều khiển động cơ Servo SG90 sang chân **10** (để tránh trùng lặp chân 11 của `IN4`).

---

<a name="sự-cố-5"></a>
### 5. Sự cố 5: Cảm biến siêu âm HC-SR04 luôn trả về khoảng cách tối đa 999.0 cm
* **Triệu chứng:** Xe chạy bình thường nhưng cảm biến siêu âm luôn báo khoảng cách `999.0` (tương đương với timeout không nhận được xung Echo) ngay cả khi có vật cản sát phía trước.
* **Nguyên nhân:** 
  1. *Đấu sai chân cắm:* Bo mạch mở rộng mở rộng (Sensor Shield) của kit ELEGOO định tuyến sẵn các đường tín hiệu của cảm biến siêu âm về hai chân analog **A5** (đóng vai trò chân Trigger) và **A4** (đóng vai trò chân Echo), trong khi code cũ định nghĩa chân 12 và 13.
  2. *Nhiễu ngắt của thư viện Servo:* Thư viện `Servo.h` mặc định liên tục kích hoạt ngắt phần cứng trên Timer 1 của Arduino để giữ vị trí góc quay, gây xung đột và làm sai lệch phép đo đạc thời gian của hàm `pulseIn()` dùng cho siêu âm.
* **Giải pháp:**
  * Sửa lại chân cắm trong code [AutoClaw.cpp](file:///c:/Users/khoi1/MyRepo/NT131.Q21-Group-9/AutoClaw/AutoClaw.cpp) thành: `pinTRIG = A5` và `pinECHO = A4`.
  * Viết hàm điều khiển Servo an toàn `safeServoWrite()`: Thực hiện `attach` servo tạm thời khi cần xoay và gọi `detach()` giải phóng Timer ngay sau khi xoay xong để triệt tiêu nhiễu ngắt phần cứng, bảo vệ độ chính xác cho hàm đo khoảng cách siêu âm.
