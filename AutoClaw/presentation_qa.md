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
* [Nhóm 6: Các lỗi kỹ thuật và sự cố phần cứng/phần mềm thực tế đã vượt qua](#nhóm-6)

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

### Q13: Mô tả luồng hoạt động chi tiết của tính năng điều khiển xe bằng cử chỉ tay (MediaPipe)?
* **Trả lời:** Luồng xử lý cử chỉ tay hoạt động theo mô hình xử lý phân tán **Edge-Client** để tối ưu hiệu năng:
  1. **Thu thập dữ liệu hình ảnh (Client-side):** Trình duyệt của người dùng (Client) sử dụng camera/webcam để bắt các khung hình video thời gian thực từ tay người điều khiển.
  2. **Nhận diện cử chỉ (Mô hình AI trên Client):** Các khung hình được đưa trực tiếp vào thư viện **MediaPipe Tasks Vision** (chạy bằng Javascript WebAssembly, tận dụng card đồ họa GPU của thiết bị client). MediaPipe phát hiện 21 điểm mốc khớp xương bàn tay (Hand Landmarks) và phân tích dạng hình học:
     * **OK Sign (👌):** Trình duyệt phát hiện ngón trỏ và ngón cái chạm nhau tạo vòng tròn $\rightarrow$ Lệnh Tiến (`F`).
     * **Closed Fist (✊):** Cả 5 ngón tay khép chặt vào lòng bàn tay $\rightarrow$ Lệnh Dừng (`S`).
     * **Pointing Up (☝):** Chỉ có ngón trỏ giơ thẳng đứng hướng lên $\rightarrow$ Lệnh Lùi (`B`).
     * **Thumb Left (👈) / Thumb Right (👉):** Ngón cái chỉ sang trái/phải $\rightarrow$ Lệnh Rẽ Trái (`L`) / Rẽ Phải (`R`).
     * **Victory (✌):** Ngón trỏ và ngón giữa tạo hình chữ V $\rightarrow$ Kích hoạt Gemini AI phân tích hình ảnh và ra lệnh tự trị.
  3. **Gửi lệnh điều khiển (API Request):** Sau khi nhận dạng được cử chỉ tương ứng, JavaScript trên frontend tự động gửi một yêu cầu HTTP GET đến Flask API của Pi (ví dụ `/control/F`).
  4. **Thực thi lệnh vật lý (Lõi an toàn & Vi điều khiển):** Flask nhận request, chuyển qua lớp an toàn Rust để kiểm tra khoảng cách vật cản. Nếu an toàn, lệnh sẽ được đẩy qua Serial xuống Arduino để điều khiển các bánh xe di chuyển.

### Q14: Mô tả luồng hoạt động chi tiết của tính năng điều khiển bằng giọng nói (tiếng Việt)?
* **Trả lời:** Luồng điều khiển bằng giọng nói được thiết kế dựa trên dịch vụ nhận dạng tiếng Việt trực tuyến kết hợp tiền xử lý từ khóa trên Client:
  1. **Thu âm & Chuyển âm thoại thành văn bản (STT):** Trình duyệt của người dùng sử dụng API **Web Speech API** (cụ thể là đối tượng `webkitSpeechRecognition`) để truy cập Microphone. Khi người dùng nói, âm thanh được gửi lên máy chủ nhận diện của Google Cloud (qua Web API tích hợp sẵn trong trình duyệt Chrome) để chuyển đổi thành văn bản tiếng Việt thời gian thực với cấu hình `recognition.lang = 'vi-VN'`.
  2. **Tiền xử lý chuỗi và Trích xuất từ khóa (Keyword Extraction):** Đoạn văn bản trả về sẽ được đưa vào hàm xử lý JavaScript trên Dashboard:
     * Toàn bộ chuỗi chữ được chuẩn hóa (chuyển sang chữ thường, loại bỏ khoảng trắng thừa).
     * Sử dụng cấu trúc rẽ nhánh khớp chuỗi (String Matching) để lọc ra các từ khóa điều khiển:
       * *"tiến" / "đi thẳng" / "tới"* $\rightarrow$ Map với lệnh Tiến (`F`).
       * *"lùi" / "xuống"* $\rightarrow$ Map với lệnh Lùi (`B`).
       * *"rẽ trái" / "quay trái" / "sang trái"* $\rightarrow$ Map với lệnh Rẽ Trái (`L`).
       * *"rẽ phải" / "quay phải" / "sang phải"* $\rightarrow$ Map với lệnh Rẽ Phải (`R`).
       * *"dừng" / "đứng lại" / "stop"* $\rightarrow$ Map với lệnh Dừng (`S`).
       * *"nhìn trái" / "nhìn phải" / "nhìn thẳng"* $\rightarrow$ Map tương ứng với góc quay Servo (`1`, `2`, `3`).
       * *"ai phân tích" / "quét vật cản"* $\rightarrow$ Gọi API phân tích hình ảnh AI (`/ai_analyze`).
  3. **Gửi và Thực thi lệnh:** JavaScript frontend gọi API tương ứng đến Flask Server `/control/<cmd>` hoặc `/ai_analyze` giống như luồng cử chỉ tay để điều khiển xe vật lý.

---


<a name="nhóm-6"></a>
## 🛠️ Nhóm 6: Lỗi kỹ thuật và Sự cố thực tế đã vượt qua

### Q11: Trong quá trình triển khai thực tế, nhóm đã gặp những sự cố phần cứng nào khó khăn nhất và giải quyết ra sao?
* **Trả lời:** Có hai sự cố phần cứng tiêu biểu mà nhóm đã giải quyết triệt để:
  1. **Lỗi sụt áp nguồn & Nhiễu ngắt của Servo:** Ban đầu, khi xe đang chạy thẳng ở chế độ tự động và gặp vật cản, cảm biến siêu âm liên tục báo khoảng cách giả định `999.0` cm khiến xe đâm vào vật cản. 
     * *Nguyên nhân:* Thư viện `Servo.h` trên Arduino Uno liên tục chạy ngắt phần cứng trên Timer 1 để giữ góc cho động cơ servo SG90, gây nhiễu cho hàm đo xung thời gian `pulseIn()` của cảm biến siêu âm. 
     * *Giải pháp:* Nhóm đã viết hàm điều khiển an toàn `safeServoWrite()`. Hàm này chỉ kết nối (`attach`) servo khi cần thay đổi góc quay, và ngay sau đó gọi ngắt kết nối (`detach`) để trả lại môi trường không có ngắt cho cảm biến siêu âm đo đạc chính xác.
  2. **Lỗi GPIO phần cứng của Arduino:** Có thời điểm động cơ bên trái của xe không thể tiến hoặc xoay phải được dù lệnh lùi vẫn tốt.
     * *Nguyên nhân:* Chân GPIO số 6 trên bo mạch Arduino Uno bị hỏng vật lý phần cứng ở trạng thái xuất điện áp mức cao (`HIGH`).
     * *Giải pháp:* Nhóm đã cấu hình lại firmware chuyển chân tín hiệu `IN1` điều khiển động cơ bên trái từ chân số **6** sang chân số **4** để dự phòng cổng ra, giúp xe hoạt động bình thường mà không cần thay bo Arduino mới.
  3. **Lỗi cắm sai chân cảm biến trên Shield mở rộng:** 
     * *Nguyên nhân:* Mạch mở rộng (Sensor Shield V5) của xe tự động định tuyến chân TRIG và ECHO của siêu âm về cổng **A5** và **A4**, trong khi mã nguồn cũ mặc định cấu hình chân 12 và 13.
     * *Giải pháp:* Nhóm đã cập nhật lại cấu hình chân của siêu âm trong firmware khớp hoàn toàn với thiết kế mạch in của ELEGOO: `pinTRIG = A5`, `pinECHO = A4`, và chân tín hiệu Servo về cổng số **3**.

### Q12: Về mặt phần mềm và biên dịch hệ thống, nhóm đã giải quyết xung đột gì khi deploy trên Raspberry Pi 4?
* **Trả lời:** 
  1. **Lỗi không tương thích Python 3.13 trên Raspberry Pi OS mới:** Lõi Rust ban đầu sử dụng PyO3 v0.21 không tương thích với Python 3.13 (gây lỗi biên dịch `maturin develop`). Nhóm đã nâng cấp PyO3 lên phiên bản **v0.22** và cấu hình lại API `#[pymodule]` để hỗ trợ trơn tru Python 3.13.
  2. **Lỗi định dạng ngắt dòng (CRLF vs LF):** File `Cargo.toml` viết trên Windows khi chuyển lên Linux gặp lỗi cú pháp TOML do ký tự ngắt dòng ẩn `\r` (CR). Nhóm đã chuẩn hóa toàn bộ mã nguồn sang ngắt dòng **LF** và bổ sung cấu hình **`.gitattributes`** để ngăn ngừa lỗi này lặp lại trong tương lai.
  3. **Thiếu thư viện hệ thống khi build reqwest:** Hệ thống thiếu gói OpenSSL và udev headers (`libssl-dev`, `libudev-dev`, `pkg-config`). Nhóm đã cài đặt bổ sung các package này từ kho APT của Debian để quá trình biên dịch liên kết thư viện tĩnh thành công.

