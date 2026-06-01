# Hướng dẫn Chuẩn bị Câu hỏi & Trả lời Bảo vệ Đồ án (Presentation Q&A) 🎓
*Dự án xe tự hành thông minh AutoClaw - Khoa Mạng Máy Tính & Truyền Thông (UIT)*

Tài liệu này tổng hợp các câu hỏi chuyên sâu mà Hội đồng phản biện và Giảng viên hướng dẫn có thể đặt ra trong buổi báo cáo đồ án, kèm theo câu trả lời gợi ý chi tiết, bám sát kỹ thuật và mang tính học thuật cao.

---

## 📁 Danh mục các Nhóm Câu hỏi
* [Nhóm 1: Kiến trúc Hệ thống & Tích hợp đa ngôn ngữ (Python, Rust, C++)](#nhóm-1)
* [Nhóm 2: Lõi An toàn bằng Rust (Safety Reflex)](#nhóm-2)
* [Nhóm 3: Vòng lặp Quyết định Tự trị của AI Agent (Gemini API)](#nhóm-3)
* [Nhóm 4: Điều khiển Phần cứng & Xử lý Tín hiệu (Arduino)](#nhóm-4)
* [Nhóm 5: Các phương thức điều khiển Đa phương thức (Voice, Gesture, Telegram)](#nhóm-5)

---

<a name="nhóm-1"></a>
## 🕹️ Nhóm 1: Kiến trúc Hệ thống & Tích hợp đa ngôn ngữ

### Q1: Tại sao nhóm lại sử dụng kết hợp cả 3 ngôn ngữ Python, Rust và C++ trong đồ án này? Sao không viết toàn bộ bằng Python hay C++ để đơn giản hóa?
* **Trả lời:** Sự kết hợp này dựa trên nguyên lý **"Chọn đúng công cụ cho đúng việc"** nhằm tối ưu hóa tài nguyên phần cứng có hạn trên thiết bị Edge (Raspberry Pi 4):
  * **C++ (Arduino):** Thích hợp nhất cho lập trình vi điều khiển cấp thấp nhờ khả năng tương tác trực tiếp với phần cứng, thời gian đáp ứng thời gian thực (real-time) và không có độ trễ của hệ điều hành.
  * **Python (Flask):** Đóng vai trò làm máy chủ Web và tích hợp thư viện ứng dụng (Telegram Bot API, OpenCV, và giao tiếp client-side MediaPipe). Python giúp xây dựng Dashboard và điều phối luồng nhanh chóng.
  * **Rust (Safety Engine & Agent):** Mang lại hiệu năng cực cao tương đương C/C++ nhưng **an toàn tuyệt đối về bộ nhớ (Memory Safety)** nhờ cơ chế *Ownership & Borrow Checker*. Rust xử lý luồng background đa nhiệm (Async/Tokio runtime) cực kỳ nhẹ nhàng mà không gặp hiện tượng nghẽn luồng (GIL) như Python, và đóng vai trò làm chốt chặn an toàn (Safety Reflex) mà Python không thể đảm nhiệm một cách tin cậy.

### Q2: Cơ chế tích hợp và truyền thông giữa Python, Rust và C++ hoạt động như thế nào?
* **Trả lời:**
  * **Python $\leftrightarrow$ Rust:** Sử dụng **PyO3** để biên dịch lõi an toàn của Rust (`autoclaw-core`) thành một thư viện nhị phân dạng `.so` (C-extension module). Từ đó, mã nguồn Python (`app.py`) có thể import và gọi các hàm của Rust trực tiếp như một thư viện Python thông thường (`import autoclaw_core`).
  * **Rust $\leftrightarrow$ C++ (Arduino):** Giao tiếp thông qua cáp **USB Serial** (UART). Rust mở cổng Serial (sử dụng crate `serialport`), chạy một luồng đọc ngầm (background reader thread) để bắt các tín hiệu khoảng cách và chế độ gửi lên từ Arduino, đồng thời gửi ngược các lệnh điều khiển (`F`, `B`, `L`, `R`, `S`) xuống cho vi điều khiển C++ thực thi.

---

<a name="nhóm-2"></a>
## 🛡️ Nhóm 2: Lõi An toàn bằng Rust (Safety Reflex)

### Q3: Cơ chế "Safety Reflex" bảo vệ xe khỏi va chạm hoạt động như thế nào trong Rust?
* **Trả lời:** Lõi an toàn Rust liên tục nhận dữ liệu khoảng cách đo được từ cảm biến siêu âm thông qua luồng đọc Serial ngầm và cache vào cấu trúc bộ nhớ chia sẻ an toàn `Arc<Mutex<f32>>`.
  * Khi người dùng (hoặc AI Agent) gửi lệnh đi Tiến (`F`), hàm `send_command("F")` của Rust sẽ chặn lệnh lại và thực hiện một kiểm tra điều kiện an toàn: **Nếu khoảng cách cache $< 15\text{ cm}$**, Rust sẽ tự động hủy bỏ lệnh `F` và thay thế bằng lệnh Dừng xe `S` rồi mới gửi xuống Arduino.
  * Việc này diễn ra hoàn toàn ở tầng ngôn ngữ biên dịch Rust, hoạt động độc lập và không phụ thuộc vào trạng thái bận hay rảnh của Flask Server (Python), đảm bảo phanh khẩn cấp hoạt động tức thời.

### Q4: Tại sao phải dùng luồng đọc ngầm (Background Reader Thread) trong Rust thay vì để Python đọc cổng Serial?
* **Trả lời:** Giao tiếp Serial trên Linux là dạng chặn luồng (blocking). Nếu Python trực tiếp đọc/ghi cổng Serial, khi Flask Server xử lý đồng thời các request camera hoặc nhận diện giọng nói, luồng chính sẽ bị block, dẫn đến mất mát dữ liệu Serial hoặc phản hồi chậm.
  * Rust giải quyết triệt để bằng cách spawn một thread riêng độc lập ở tầng hệ thống để đọc liên tục từ Arduino và cập nhật vào cache. Python chỉ việc đọc giá trị từ cache (thông qua con trỏ an toàn `Arc<Mutex>`), đảm bảo thời gian phản hồi cực nhanh (dưới 1ms) và non-blocking hoàn toàn.

---

<a name="nhóm-3"></a>
## 🤖 Nhóm 3: Vòng lặp Quyết định Tự trị của AI Agent

### Q5: Chế độ tự lái bằng AI (Gemini Vision) hoạt động ra sao? Nó khác gì với tự lái bằng cảm biến siêu âm (C++)?
* **Trả lời:** 
  * **Tự lái bằng C++ (Arduino):** Là phản xạ cấp thấp (Reflex). Xe chỉ phản ứng một cách máy móc dựa trên khoảng cách số đo được trước mặt (nếu $<20\text{ cm}$ thì quay trái/phải). Nó không biết xung quanh có vật thể gì, môi trường ra sao.
  * **Tự lái bằng AI Agent (Rust + Gemini Vision):** Là hành vi tự trị cấp cao (Cognitive). Khi kích hoạt chế độ AI, Rust Agent chạy ngầm một vòng lặp Tokio không đồng bộ. Nó sẽ:
    1. Gọi công cụ **CaptureSnapshotTool** để chụp ảnh môi trường thực tế trước đầu xe.
    2. Gọi công cụ **GetDistanceTool** lấy số đo khoảng cách cảm biến.
    3. Đóng gói cả hình ảnh (Base64) và số đo cảm biến gửi lên **API Gemini 2.0 Flash**.
    4. Gemini phân tích ngữ cảnh (ví dụ: "Trước mặt là một chiếc hộp các-tông ở cự ly 25cm, bên phải trống") và đề xuất hướng lái tối ưu tránh vật cản bằng tiếng Việt (dưới 15 từ) cùng lệnh điều hướng đi kèm.
    5. Agent tự động thực thi lệnh đó thông qua **ControlCarTool**.

### Q6: Làm sao để đảm bảo xe không bị đâm vào tường khi đang đợi API Gemini phản hồi (thường mất 1-2 giây)?
* **Trả lời:** Đây là điểm mấu chốt của tính năng an toàn nhiều lớp trong đồ án của nhóm:
  * Trong lúc chờ API Gemini phân tích ảnh và trả về lệnh lái, xe vẫn liên tục di chuyển tiến. 
  * Tuy nhiên, nếu khoảng cách đột ngột giảm xuống dưới $15\text{ cm}$, lõi an toàn **Safety Reflex** bằng Rust (ở tầng dưới) sẽ ngay lập tức kích hoạt, chặn đứng lệnh đi tiếp và ép dừng xe mà không cần đợi kết quả phân tích của Gemini. Điều này ngăn chặn việc xe đâm vào vật cản do độ trễ mạng hoặc độ trễ xử lý của mô hình AI.

---

<a name="nhóm-4"></a>
## 🔌 Nhóm 4: Điều khiển Phần cứng & Xử lý Tín hiệu (Arduino)

### Q7: Thuật toán trên Arduino tự động lọc nhiễu tín hiệu siêu âm như thế nào để tránh xe phanh ảo?
* **Trả lời:** Cảm biến siêu âm HC-SR04 thường bị nhiễu do sóng phản xạ góc chéo hoặc nhiễu điện từ động cơ (xuất hiện các phép đo sai lệch cực ngắn $0$ hoặc dưới $15\text{ cm}$).
  * Trong hàm `updateAutoDrive()`, nhóm triển khai một bộ lọc tích lũy trạng thái (State Accumulator) thông qua biến `consecutiveObstacleCount`.
  * Xe chỉ xác nhận có vật cản và dừng lại khi khoảng cách đo được nhỏ hơn ngưỡng an toàn trong **ít nhất 2 chu kỳ đo liên tiếp** (mỗi chu kỳ cách nhau 50ms, tương đương tổng thời gian xác nhận là 100ms). Nếu chỉ có 1 chu kỳ báo khoảng cách nhỏ rồi chu kỳ sau bình thường, bộ lọc sẽ tự động reset về `0` và coi đó là nhiễu xung động.

### Q8: Cơ chế điều khiển Servo tránh vật cản của nhóm được thiết kế như thế nào để không gây xung đột với cảm biến siêu âm?
* **Trả lời:** Thư viện `Servo.h` trên Arduino Uno sử dụng ngắt cứng của Timer 1. Ngắt này xảy ra liên tục để duy trì góc quay của servo, gây nhiễu nghiêm trọng cho hàm `pulseIn()` (đo độ rộng xung phản hồi của siêu âm), khiến cảm biến luôn báo lỗi timeout (`999.0` cm).
  * **Giải pháp phần mềm:** Nhóm thiết kế hàm an toàn `safeServoWrite()`. Hàm này chỉ `attach()` servo vào chân điều khiển khi cần thay đổi góc quay, ghi góc mới (`write()`), chờ một khoảng ngắn cho servo dịch chuyển vật lý, và lập tức `detach()` servo để giải phóng hoàn toàn Timer 1, trả lại môi trường sạch không ngắt cho cảm biến siêu âm đo đạc.

---

<a name="nhóm-5"></a>
## 🌐 Nhóm 5: Các phương thức điều khiển Đa phương thức

### Q9: Làm cách nào nhóm truyền hình ảnh từ xe lên trình duyệt và Telegram Bot mà không làm nghẽn băng thông của Raspberry Pi 4?
* **Trả lời:** Nhóm không sử dụng cơ chế truyền phát video liên tục (Video Streaming MJPEG) vì nó tiêu tốn từ 5 - 10 Mbps băng thông mạng, gây nghẽn vi xử lý và làm trễ các lệnh điều khiển thời gian thực.
  * Thay vào đó, nhóm áp dụng kiến trúc **Snapshot-on-Demand** (Chụp ảnh theo yêu cầu): 
    * Trên web dashboard, camera chỉ chụp 1 khung hình tĩnh khi người dùng yêu cầu phân tích AI (phím `G` hoặc cử chỉ Victory).
    * Trên Telegram Bot, camera chỉ chụp và gửi ảnh khi người dùng gõ lệnh `/snapshot`.
  * Phương pháp này tiết kiệm đến **99%** băng thông mạng LAN và giảm tải xử lý CPU trên Raspberry Pi xuống mức tối thiểu (dưới 5%).

### Q10: Tại sao tính năng Nhận diện cử chỉ bàn tay (MediaPipe) và Giọng nói (Web Speech API) chỉ hoạt động qua kết nối HTTPS bảo mật hoặc localhost?
* **Trả lời:** Đây là chính sách bảo mật bắt buộc của các trình duyệt hiện đại (Chrome, Edge, Safari) nhằm bảo vệ quyền riêng tư của người dùng. Các API truy cập phần cứng nhạy cảm như Camera (getUserMedia) và Microphone (SpeechRecognition) **chỉ được cấp quyền chạy** trong môi trường bảo mật (Secure Contexts), bao gồm địa chỉ `localhost` hoặc các tên miền sử dụng giao thức mã hóa **HTTPS**.
  * Để chạy thực tế trên Raspberry Pi qua mạng Wi-Fi của phòng Lab/LAN, nhóm đã tích hợp công cụ **Ngrok** để tạo một đường truyền bảo mật (tunnel) HTTPS hướng ngoại. Trình duyệt client truy cập qua link ngrok này sẽ được cấp đầy đủ quyền sử dụng Camera và Micro để điều khiển xe từ xa.
