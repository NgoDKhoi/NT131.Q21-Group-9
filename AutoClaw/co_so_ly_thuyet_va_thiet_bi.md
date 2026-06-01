# BÁO CÁO ĐỀ TÀI: XE ROBOT TỰ HÀNH AUTOCLAW
## PHẦN 2: CƠ SỞ LÝ THUYẾT VÀ THIẾT BỊ PHẦN CỨNG

---

### I. CƠ SỞ LÝ THUYẾT

#### 1. Kiến trúc AI Agent tự trị (Autonomous AI Agent)
*   **Khái niệm tác tử AI (AI Agent)**: Là một thực thể có khả năng nhận cảm (perceive) môi trường xung quanh thông qua các cảm biến (sensors), xử lý thông tin để đưa ra quyết định lý giải (reasoning) và thực thi hành động (act) thông qua các cơ cấu chấp hành (actuators) nhằm đạt được một mục tiêu cụ thể.
*   **Vòng lặp Agent (Agent Loop) trong AutoClaw**:
    1.  **Perceive (Nhận cảm)**: Thu thập dữ liệu khoảng cách từ cảm biến siêu âm HC-SR04 và chụp ảnh snapshot thực tế từ Camera.
    2.  **Reason (Suy luận)**: Gửi dữ liệu đa phương thức (hình ảnh + khoảng cách vật lý) về mô hình ngôn ngữ lớn (Gemini 2.0 Flash Vision). Mô hình sẽ phân tích ngữ cảnh hình ảnh để nhận diện vật thể (ví dụ: con người, hộp cát, tường chắn) bằng tiếng Việt và đề xuất hướng bẻ lái tối ưu kèm theo lý giải logic (`reason`).
    3.  **Act (Hành động)**: Chuyển đổi đề xuất của AI thành các ký hiệu lệnh điều khiển nhị phân (`F`, `B`, `L`, `R`, `S`) và truyền qua giao thức Serial để Arduino điều khiển động cơ bẻ lái vật lý.
*   **Lõi bất đồng bộ không chặn (Asynchronous Rust Runtime)**: Sử dụng thư viện `Tokio` trong Rust để tạo một vòng lặp Agent chạy ngầm song song với luồng chính của máy chủ Web. Việc thực thi không đồng bộ bảo đảm tác vụ gọi API Gemini (mất từ 1.5s - 2s) không làm nghẽn quá trình nhận lệnh thủ công hoặc xử lý an toàn khẩn cấp.

#### 2. Phản xạ an toàn thời gian thực (Safety Reflex)
*   **Nguyên lý phản xạ có điều kiện khẩn cấp**: Trong thiết kế hệ thống tự hành nhúng, độ trễ truyền thông mạng (API Call, HTTP Request) là một mối nguy hiểm lớn. Nếu chỉ dựa hoàn toàn vào AI để dừng xe, độ trễ vài giây có thể khiến xe va chạm vật lý trước khi AI phản hồi.
*   **Cơ chế Safety Reflex**: Thiết lập một lõi an toàn trung gian bằng ngôn ngữ **Rust**. Lõi này liên tục đọc giá trị khoảng cách phản hồi từ cảm biến siêu âm được lưu vào bộ đệm cache (`latest_distance`). Ngay khi nhận được lệnh di chuyển tiến (`F`) nhưng khoảng cách thực tế $< 15\text{ cm}$, lõi Rust sẽ lập tức thực hiện cơ chế chặn (intercept) lệnh, ghi đè lệnh thành dừng (`S`) và gửi trực tiếp xuống vi điều khiển Arduino. Quá trình này diễn ra ở cấp độ micro giây, đảm bảo an toàn tuyệt đối cho phần cứng.

#### 3. Thị giác máy tính và Nhận diện cử chỉ (MediaPipe Hand Landmarker)
*   **Mạng nơ-ron tích chập (CNN) trên trình duyệt**: MediaPipe sử dụng mô hình học máy gọn nhẹ được tối ưu hóa để chạy trực tiếp trên GPU máy khách (Client-side) thông qua trình duyệt Web WebGL/WebAssembly.
*   **Mô hình Hand Landmarker**: Nhận diện bàn tay và trích xuất tọa độ của 21 điểm mốc xương bàn tay (keypoints) dạng 3D ($X, Y, Z$).
*   **Thuật toán phân tích cử chỉ hình học (Heuristics Filter)**:
    *   **Cử chỉ Victory (✌)**: Ngón trỏ và ngón giữa duỗi thẳng (khoảng cách đầu ngón tới lòng bàn tay lớn), ngón đeo nhẫn và ngón út co lại. Kích hoạt trigger chụp ảnh gửi Gemini phân tích.
    *   **Cử chỉ OK (👌)**: Ngón cái và ngón trỏ chạm đầu vào nhau (khoảng cách rất nhỏ), các ngón khác duỗi thẳng. Kích hoạt lệnh Tiến (`F`).
    *   **Cử chỉ Nắm đấm (✊)**: Cả 5 ngón tay đồng loạt co lại về phía lòng bàn tay. Kích hoạt lệnh Dừng (`S`).

#### 4. Giao tiếp nhị phân Serial UART và Đồng bộ Trạng thái
*   **Giao thức Serial UART**: Phương thức truyền thông nối tiếp dị bộ sử dụng 2 đường dây TX (truyền) và RX (nhận) hoạt động ở tốc độ truyền tín hiệu (Baudrate) thiết lập là **9600 bps**.
*   **Cơ chế đồng bộ không chặn (Non-blocking I/O)**:
    *   **Phía Arduino**: Sử dụng hàm `millis()` để kiểm tra chu kỳ gửi khoảng cách (mỗi 200ms) và chu kỳ báo cáo chế độ (mỗi 1000ms), thay thế hoàn toàn cho hàm chặn `delay()`. Nhờ đó, luồng xử lý Serial Commands (`handleSerialCommands`) có thể đón nhận lệnh điều khiển ngay lập tức tại bất kỳ thời điểm nào trong vòng lặp `loop()`.
    *   **Phía Raspberry Pi (Rust Core)**: Mở một luồng phụ ngầm (Background Reader Thread) chuyên trách việc lắng nghe dữ liệu từ Arduino để phân tách và cập nhật liên tục vào bộ đệm cache dùng chung được bảo vệ bằng cơ chế khóa loại trừ tương hỗ `Mutex` (`Arc<Mutex<f32>>` và `Arc<Mutex<String>>`).

---

### II. THIẾT BỊ VÀ LINH KIỆN PHẦN CỨNG

| STT | Thiết bị / Linh kiện | Hình ảnh minh họa | Thông số kỹ thuật chính | Vai trò trong hệ thống AutoClaw |
| :---: | :--- | :---: | :--- | :--- |
| **1** | **Raspberry Pi 4 Model B (8GB)** | *(Hỗ trợ cổng USB, LAN, Wi-Fi)* | • CPU: Broadcom BCM2711 quad-core Cortex-A72 64-bit 1.5GHz<br>• RAM: 8GB LPDDR4<br>• OS: Debian Linux (64-bit) | Đóng vai trò bộ não trung tâm cao cấp (Edge Server). Chạy Flask Web Server, thực thi lõi an toàn Rust Core (Agent Loop), kết nối Internet và lưu trữ các khóa cấu hình API. |
| **2** | **Arduino Uno R3** | *(Mạch vi điều khiển mã nguồn mở)* | • Vi điều khiển: ATmega328P<br>• Điện áp hoạt động: 5V<br>• Tốc độ thạch anh: 16 MHz<br>• Bộ nhớ Flash: 32 KB | Đóng vai trò bộ điều khiển cấp thấp (Low-level Controller). Nhận lệnh di chuyển qua cổng USB Serial để phát tín hiệu xung PWM điều khiển động cơ và điều phối góc xoay Servo. |
| **3** | **Cảm biến siêu âm HC-SR04** | *(Thiết bị đo cự ly bằng sóng siêu âm)* | • Điện áp hoạt động: 5V DC<br>• Tần số hoạt động: 40 kHz<br>• Khoảng cách đo: 2cm - 400cm<br>• Góc đo phản hồi: < 15 độ | Đo khoảng cách vật lý từ đầu xe đến chướng ngại vật trước mặt. Cung cấp dữ liệu thời gian thực giúp kích hoạt cơ chế phanh khẩn cấp `Safety Reflex`. |
| **4** | **Động cơ Servo SG90** | *(Động cơ bước góc xoay nhỏ)* | • Điện áp hoạt động: 4.8V - 6V<br>• Lực kéo: 1.6 kg/cm<br>• Tốc độ: 0.12s / 60 độ<br>• Góc quay tối đa: 180 độ | Xoay cảm biến siêu âm HC-SR04 và Camera sang các hướng quét **30° (Trái) - 90° (Giữa) - 150° (Phải)** để bao quát vật cản chéo khi xe chuẩn bị rẽ. |
| **5** | **Mạch cầu H kép L298N** | *(Mạch công suất điều khiển động cơ)* | • Điện áp động cơ: 5V - 35V DC<br>• Dòng điện đỉnh cực đại: 2A/mỗi cầu<br>• Công suất tối đa: 25W | Nhận tín hiệu điều hướng (`IN1` đến `IN4`) và tín hiệu điều tốc PWM (`ENA`, `ENB`) từ Arduino để điều phối dòng điện cấp cho 4 động cơ DC trên bánh xe. |
| **6** | **Camera Module / USB Webcam** | *(Mắt thần ghi nhận hình ảnh)* | • Độ phân giải: 720p / 1080p<br>• Kết nối: Giao tiếp USB hoặc cáp MIPI CSI | Chụp ảnh snapshot môi trường thực tế phía trước xe dưới dạng mã hóa Base64 gửi lên Gemini Vision API thực hiện phân tích nhận diện vật cản. |
| **7** | **Động cơ DC giảm tốc & Bánh xe** | *(Cơ cấu di chuyển vật lý)* | • Điện áp hoạt động: 3V - 6V DC<br>• Tỷ số truyền: 1:48<br>• Lốp cao su bám đường tốt | Thực hiện chuyển động cơ học (Tiến, lùi, rẽ, dừng) của khung gầm xe robot. |

---

### III. SƠ ĐỒ NGUYÊN LÝ ĐẤU NỐI (WIRING DIAGRAM)

```mermaid
graph TD
    subgraph RaspberryPi4["Raspberry Pi 4 (Edge Server)"]
        USB_Port["Cổng USB Host"]
        Pi_Cam["Pi Camera / USB Cam"]
    end

    subgraph ArduinoUno["Arduino Uno R3 (Vi điều khiển)"]
        USB_Serial["Cổng USB Serial (ATmega16U2)"]
        Pin_3["Pin D3 (PWM)"]
        Pin_5["Pin D5 (PWM)"]
        Pin_6["Pin D6 (PWM)"]
        Pin_7["Pin D7"]
        Pin_8["Pin D8"]
        Pin_9["Pin D9"]
        Pin_11["Pin D11"]
        Pin_A5["Pin A5 (Analog)"]
        Pin_A4["Pin A4 (Analog)"]
    end

    subgraph HardwareComponents["Cơ cấu chấp hành & Cảm biến"]
        Servo["SG90 Servo"]
        US_Sensor["Cảm biến siêu âm HC-SR04"]
        L298N["Mạch cầu H L298N"]
        Motors["4 Động cơ DC (Bánh xe)"]
    end

    %% Kết nối truyền thông & Camera
    USB_Port <==>|"Cáp USB Type-B (Baud 9600)"| USB_Serial
    Pi_Cam -.->|Mã hóa JPEG/Base64| USB_Port

    %% Kết nối Servo & Siêu âm
    Pin_3 -->|Tín hiệu xung điều khiển| Servo
    Pin_A5 -->|Chân TRIG (Phát sóng)| US_Sensor
    Pin_A4 <--- |Chân ECHO (Thu sóng)| US_Sensor

    %% Điều khiển Động cơ qua L298N
    Pin_5 -->|Tốc độ bánh Trái ENA| L298N
    Pin_6 -->|Tốc độ bánh Phải ENB| L298N
    Pin_7 -->|Hướng IN1| L298N
    Pin_8 -->|Hướng IN2| L298N
    Pin_9 -->|Hướng IN3| L298N
    Pin_11 -->|Hướng IN4| L298N

    L298N ===>|Dòng công suất điều tốc| Motors
```

> [!NOTE]
> **Giải pháp tối ưu hóa nguồn điện**: Do mạch L298N và 4 động cơ DC tiêu thụ dòng điện rất lớn khi khởi động, hệ thống sử dụng nguồn cấp độc lập từ 2 viên pin Li-ion 18650 (7.4V - 8.4V) nối vào cổng nguồn `GND/12V` của L298N. Từ L298N trích nguồn ổn áp 5V cấp ngược lại cho Arduino. Raspberry Pi 4 được cấp nguồn riêng biệt bằng Pin sạc dự phòng 5V-3A nhằm tránh hiện tượng sụt áp đột ngột gây khởi động lại hệ thống (Reset loop).
