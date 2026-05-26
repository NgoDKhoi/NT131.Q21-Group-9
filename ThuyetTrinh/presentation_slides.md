# BÁO CÁO ĐỒ ÁN: XE TỰ HÀNH AUTOCLAW TÍCH HỢP EDGE AI & ĐIỀU KHIỂN ĐA PHƯƠNG THỨC

Tài liệu này chứa nội dung chi tiết cho **15 Slide** báo cáo đồ án phục vụ buổi thuyết trình. Mỗi slide bao gồm: tiêu đề, các ý chính hiển thị trực quan và lời thoại/nội dung chi tiết cho người thuyết trình.

---

## 🖥️ Slide 1: Trang tiêu đề (Title Slide)
* **Tiêu đề**: XE TỰ HÀNH THÔNG MINH AUTOCLAW TÍCH HỢP EDGE AI & ĐIỀU KHIỂN ĐA PHƯƠNG THỨC
* **Nội dung chính**:
  * Đề tài môn học / Đồ án tốt nghiệp.
  * Giảng viên hướng dẫn: [Tên giảng viên]
  * Nhóm thực hiện: Group 9 (Lớp NT131.Q21)
  * Thành viên: [Tên các thành viên]
* **Hình ảnh gợi ý**: Ảnh chụp thực tế xe AutoClaw với đèn LED/camera hoặc hình minh họa xe robot có cánh tay kẹp và camera quét.
* **Lời thoại người trình bày**: 
  > "Kính thưa Hội đồng và các thầy cô, nhóm chúng em xin phép được báo cáo về đồ án: 'Xe tự hành thông minh AutoClaw tích hợp Edge AI và điều khiển đa phương thức'. Đây là một hệ thống robot tự hành hướng tới việc kết hợp giữa điều khiển nhúng thời gian thực và trí tuệ nhân tạo biên (Edge AI) thông qua mô hình ngôn ngữ lớn."

---

## 🖥️ Slide 2: Đặt vấn đề & Mục tiêu đề tài
* **Nội dung chính**:
  * **Thực trạng**: Robot tự hành truyền thống thường bị giới hạn bởi các tập lệnh cứng nhắc, khó thích ứng linh hoạt với môi trường phức tạp thay đổi liên tục.
  * **Cơ hội**: Sự phát triển mạnh mẽ của Generative AI và mô hình thị giác lớn (Vision LLM) mở ra khả năng tự ra quyết định thông minh.
  * **Mục tiêu**:
    1. Thiết kế phần cứng xe tự hành ổn định, phản xạ nhanh.
    2. Xây dựng giao diện điều khiển đa phương thức (Web, Giọng nói, Cử chỉ bàn tay).
    3. Tích hợp AI để xe có khả năng tự trị và ra quyết định thông qua phân tích hình ảnh thực tế.
* **Lời thoại người trình bày**:
  > "Các robot tránh vật cản thông thường chỉ hoạt động dựa trên cảm biến siêu âm với logic rẽ trái/phải cố định. Khi gặp các chướng ngại vật phức tạp, chúng dễ bị kẹt. Mục tiêu của nhóm chúng em là đưa trí tuệ nhân tạo (Edge AI) làm bộ não quyết định hướng đi và kết hợp đa dạng các phương thức điều khiển tự nhiên cho người dùng."

---

## 🖥️ Slide 3: Tổng quan kiến trúc hệ thống
* **Nội dung chính**:
  * **Sơ đồ khối hệ thống (High-Level Architecture)**:
    ```mermaid
    graph TD
        User([Người dùng]) -->|Giọng nói/Cử chỉ/D-pad| WebClient[Frontend Web UI]
        User -->|Lệnh Telegram| TelegramBot[Telegram Bot Daemon]
        
        WebClient -->|API Requests| FlaskBackend[Flask Web Server]
        TelegramBot -->|API Requests| FlaskBackend
        
        subgraph Raspberry Pi / Máy tính chủ
            FlaskBackend -->|PyO3 Bridge| RustCore[Rust Safety Core / ZeroClaw Agent]
            RustCore -->|Gemini API| Gemini[Gemini 2.0 Flash Vision]
        end
        
        RustCore -->|Giao tiếp Serial USB| ArduinoUno[Arduino Uno Firmware]
        ArduinoUno -->|PWM/Tín hiệu| Motors[Động cơ & Servo SG90]
        ArduinoUno -->|Đo khoảng cách| HCSR04[Cảm biến siêu âm HC-SR04]
    ```
* **Lời thoại người trình bày**:
  > "Hệ thống của chúng em được chia làm 3 lớp rõ ràng: Lớp Thiết bị ngoại vi (nhận tín hiệu điều khiển trực tiếp từ người dùng), Lớp xử lý trung tâm (chạy Flask backend kết hợp cùng lõi an toàn viết bằng Rust để giao tiếp AI và xử lý dữ liệu nặng), và Lớp nhúng (Arduino Uno nhận lệnh di chuyển trực tiếp và xử lý cơ cấu chấp hành)."

---

## 🖥️ Slide 4: Nền tảng phần cứng & Cơ cấu chấp hành
* **Nội dung chính**:
  * **Khung gầm**: Xe 4 bánh, động cơ DC giảm tốc điều khiển bằng mạch cầu H L298N.
  * **Bộ xử lý nhúng**: Arduino Uno R3 (điều khiển động cơ, servo, đọc cảm biến thời gian thực).
  * **Bộ xử lý trung tâm**: Raspberry Pi hoặc Máy tính cá nhân đóng vai trò làm Web Server và chạy AI Agent.
  * **Cảm biến**:
    * Cảm biến siêu âm HC-SR04 đo khoảng cách vật cản phía trước.
    * Camera/Webcam gắn trên Servo SG90 có khả năng quét góc 180 độ.
* **Lời thoại người trình bày**:
  > "Về phần cứng, chúng em tối ưu chi phí bằng cách sử dụng Arduino Uno để xử lý nhúng ở tần số cao. Bộ xử lý trung tâm kết nối qua cáp USB Serial để gửi nhận tập lệnh. Servo SG90 dùng để quay góc camera độc lập với hướng di chuyển của xe để AI có thể chủ động quét môi trường."

---

## 🖥️ Slide 5: Cơ sở lý thuyết & Thuật toán nhúng (C++)
* **Nội dung chính**:
  * **Lập trình non-blocking (millis-based)**: Thay thế hoàn toàn hàm `delay()` bằng máy trạng thái dựa trên thời gian thực tế `millis()`.
  * **Bộ lọc chống nhiễu siêu âm**: Cơ chế tích lũy 2 chu kỳ đo liên tiếp (~100ms) dưới ngưỡng an toàn để loại bỏ các điểm nhiễu phản xạ âm (sonar drop).
  * **Thuật toán tự động tránh vật cản truyền thống (C++ State Machine)**:
    * *Quét (Scan)*: Phát hiện chướng ngại vật -> Dừng -> Quay servo sang Trái -> Quay sang Phải -> Đo khoảng cách.
    * *Quyết định (Decide)*: So sánh khoảng cách bên trái và bên phải để rẽ hướng thoáng hơn.
* **Lời thoại người trình bày**:
  > "Ở mức nhúng, toàn bộ firmware Arduino được viết tối ưu bằng ngôn ngữ C++ dưới dạng máy trạng thái phi tuần tự (non-blocking). Việc loại bỏ hàm delay giúp robot có thể phản hồi lệnh Serial từ máy tính chủ ngay lập tức, ngay cả khi đang trong chu trình tự động quét khoảng cách tránh vật cản."

---

## 🖥️ Slide 6: Công nghệ cốt lõi: Edge AI & Gemini Vision
* **Nội dung chính**:
  * **Mô hình**: Gemini 2.0 Flash API (tận dụng tốc độ phản hồi cực nhanh < 1.5 giây và khả năng hiểu hình ảnh chất lượng cao).
  * **ZeroClaw Agent Framework**: Thiết lập một AI Agent chuyên biệt chạy trên nền tảng Rust (Tokio runtime).
  * **Prompt Engineering cho Robot**:
    * AI hoạt động như một bộ não lái xe.
    * Trả về định dạng JSON nghiêm ngặt chứa: `description` (Mô tả vật cản bằng tiếng Việt dưới 15 từ) và `command` (Hướng đề xuất di chuyển: F, B, L, R, S).
* **Lời thoại người trình bày**:
  > "Khi chuyển sang chế độ tự hành AI, camera sẽ chụp ảnh môi trường phía trước. Ảnh kết hợp với thông tin khoảng cách cảm biến sẽ được gửi tới Gemini 2.0 Flash. AI đóng vai trò như một người tài xế thông minh, phân tích vật cản (ví dụ: 'đây là chiếc ghế nhựa, hãy rẽ trái') và đưa ra quyết định di chuyển tối ưu thay vì chỉ so sánh các con số khoảng cách cơ học."

---

## 🖥️ Slide 7: Công nghệ cốt lõi: Cầu nối Rust-Python (PyO3)
* **Nội dung chính**:
  * **Lý do sử dụng Rust**: Bảo đảm tính an toàn bộ nhớ, tránh tình trạng race-condition khi nhiều thread cùng ghi cổng Serial và tăng hiệu năng xử lý tác vụ đồng thời.
  * **Kiến trúc liên kết**:
    * Thư viện `zeroclaw_core` được viết bằng Rust và biên dịch thành module Python thông qua **PyO3** và **Maturin**.
    * Python Flask chỉ cần gọi các hàm từ Rust biên dịch sẵn mà không cần quản lý kết nối Serial thô.
  * **Cơ chế dự phòng (Fallback)**: Nếu chạy thử nghiệm trên máy không có môi trường biên dịch Rust, backend tự động chuyển sang file mock `zeroclaw_core.py` để chạy thử nghiệm giao diện mà không bị lỗi.
* **Lời thoại người trình bày**:
  > "Một thách thức lớn trong lập trình điều khiển xe qua Web là xung đột tài nguyên cổng Serial khi nhiều yêu cầu web gửi đến cùng lúc. Nhóm đã giải quyết triệt để bằng cách viết toàn bộ lõi quản lý kết nối và đọc luồng dữ liệu nhấp nháy bằng ngôn ngữ Rust, giao tiếp độc quyền thông qua cầu nối PyO3 sang Python Flask."

---

## 🖥️ Slide 8: Tính năng nổi bật: Dashboard Cyberpunk thời gian thực
* **Nội dung chính**:
  * **Thiết kế**: Phong cách viễn tưởng Cyberpunk neon retro, tích hợp hiệu ứng scanline và CRT glow độc đáo.
  * **Tính năng trên giao diện**:
    * Hiển thị luồng video/snapshot camera trực quan.
    * D-pad điều khiển hướng đi phản hồi trực quan.
    * Bản ghi nhật ký hệ thống (System Logs) và trạng thái kết nối phần cứng.
    * Nút chuyển đổi nhanh 3 chế độ: Thủ công (Manual), Tự động C++ (Auto C++), và Tự động AI (Auto AI).
* **Lời thoại người trình bày**:
  > "Giao diện Dashboard điều khiển được nhóm thiết kế theo phong cách Cyberpunk rất hiện đại. Mọi hoạt động của xe bao gồm khoảng cách chướng ngại vật thực tế, hình ảnh camera thu về, và các trạng thái kết nối đều được đồng bộ hóa thời gian thực lên giao diện web này."

---

## 🖥️ Slide 9: Tính năng nổi bật: Điều khiển bằng Giọng nói (Voice Control)
* **Nội dung chính**:
  * **Công nghệ**: Web Speech API tích hợp trực tiếp trên trình duyệt client-side.
  * **Xử lý ngôn ngữ tự nhiên (NLP) tiếng Việt**:
    * Hỗ trợ nhận diện các khẩu lệnh tiếng Việt tự nhiên: *"đi thẳng"*, *"chạy tới"*, *"quay lại"*, *"rẽ trái"*, *"sang phải"*, *"dừng xe"*, *"quay camera"*.
    * Cơ chế chuẩn hóa văn bản (Regex mapping) để chuyển đổi khẩu lệnh thành ký tự điều khiển tương ứng gửi đến phần cứng.
  * **Khắc phục giới hạn**: Tích hợp đường truyền HTTPS (qua Ngrok) để trình duyệt cấp quyền truy cập Microphone an toàn.
* **Lời thoại người trình bày**:
  > "Tính năng điều khiển giọng nói giúp người dùng rảnh tay. Chúng em tận dụng Web Speech API ngay trên trình duyệt để nhận diện giọng nói tiếng Việt mà không cần tốn tài nguyên xử lý âm thanh ở server. Người dùng chỉ cần nói 'rẽ trái' hay 'dừng lại', xe sẽ phản hồi gần như lập tức."

---

## 🖥️ Slide 10: Tính năng nổi bật: Điều khiển bằng Cử chỉ (Hand Gesture)
* **Nội dung chính**:
  * **Công nghệ**: MediaPipe Tasks Vision (chạy trực tiếp trên trình duyệt bằng Web Assembly và GPU tăng tốc).
  * **Mô hình**: Nhận diện cử chỉ bàn tay (Gesture Recognizer) 21 điểm xương tay.
  * **Bản đồ cử chỉ điều khiển**:
    * ✊ (Closed_Fist) -> **LÙI (B)**
    * ✋ (Open_Palm) -> **DỪNG (S)**
    * 👆 (Pointing_Up) -> **TIẾN (F)**
    * 👈 (Thumb_Left/Point_Left) -> **RẼ TRÁI (L)**
    * 👉 (Thumb_Right/Point_Right) -> **RẼ PHẢI (R)**
    * ✌ (Victory) -> **KÍCH HOẠT CHỤP ẢNH AI (Snapshot)**
* **Lời thoại người trình bày**:
  > "Bên cạnh giọng nói, chúng em tích hợp mô hình MediaPipe nhận diện cử chỉ tay thời gian thực qua camera máy tính/điện thoại. Chỉ cần đưa ngón tay trỏ hướng lên, xe sẽ chạy tới; xòe bàn tay ra, xe lập tức dừng lại; và giơ ký hiệu Victory để kích hoạt AI chụp ảnh phân tích cảnh quan."

---

## 🖥️ Slide 11: Tính năng nổi bật: Chế độ tự hành AI & Safe-Reflex
* **Nội dung chính**:
  * **Quy trình hoạt động Chế độ AI**:
    * Xe tự động di chuyển tiến lên phía trước.
    * Camera chụp ảnh liên tục truyền tải dữ liệu.
    * AI phân tích đề xuất hướng rẽ nếu có vật cản.
  * **Safe-Reflex (Phản xạ an toàn phần cứng)**:
    * *Vấn đề*: AI mất ~1.5 giây để xử lý hình ảnh, trong thời gian đó xe có thể tông vào tường nếu di chuyển nhanh.
    * *Giải pháp*: Một vòng lặp giám sát khoảng cách siêu âm chạy ở mức ưu tiên cao nhất ở Rust Safety Core. Nếu khoảng cách đột ngột < 15cm, Rust Core sẽ tự động override (ghi đè) lệnh di chuyển thành DỪNG (S) ngay lập tức mà không đợi phản hồi của AI.
* **Lời thoại người trình bày**:
  > "Một thách thức cực lớn của xe tự hành AI là độ trễ mạng khi gọi API. Để ngăn chặn va chạm trong lúc chờ AI phân tích, nhóm đã lập trình một cơ chế phản xạ an toàn gọi là Safe-Reflex ở lõi Rust. Cảm biến siêu âm liên tục giám sát khoảng cách thực tế, nếu nhỏ hơn 15cm, xe sẽ tự động phanh khẩn cấp để bảo vệ phần cứng."

---

## 🖥️ Slide 12: Tính năng nổi bật: Giám sát & Điều khiển qua Telegram Bot
* **Nội dung chính**:
  * **Vai trò**: Cho phép người dùng theo dõi và can thiệp điều khiển xe từ xa thông qua ứng dụng trò chuyện Telegram (không bị giới hạn khoảng cách địa lý).
  * **Hệ thống lệnh**:
    * `/start`, `/help`: Hướng dẫn sử dụng bằng tiếng Việt.
    * `/status`: Xem khoảng cách cảm biến hiện tại và chế độ hoạt động của xe.
    * `/snapshot`: Ra lệnh chụp ảnh và gửi ảnh trực tiếp từ camera của xe về đoạn chat.
    * `/control <F/B/L/R/S>`: Ra lệnh di chuyển thủ công.
    * `/auto <off/cpp/ai>`: Chuyển đổi linh hoạt chế độ tự lái.
* **Lời thoại người trình bày**:
  > "Nhóm cũng đã phát triển một Telegram Bot chạy song song dưới dạng tiến trình ngầm. Tính năng này cho phép bạn giám sát thông tin xe từ bất cứ đâu. Bạn chỉ cần chat với bot là có thể chụp ảnh trực tiếp từ camera xe gửi về điện thoại, hoặc ra lệnh điều khiển khi xe đang hoạt động ngoài tầm mắt."

---

## 🖥️ Slide 13: Bảo mật thông tin & Phân quyền Telegram
* **Nội dung chính**:
  * **Rủi ro**: Ai cũng có thể nhắn tin cho Telegram Bot công khai và chiếm quyền điều khiển xe gây nguy hiểm.
  * **Giải pháp phân quyền 2 tầng**:
    * **Tầng Đọc (Read-only)**: `/status`, `/snapshot`. Nếu chưa thiết lập Chat ID trong `.env`, hệ thống cho phép truy cập công khai. Nếu đã cấu hình Chat ID của chủ sở hữu, chỉ người đó mới xem được.
    * **Tầng Điều khiển (Control)**: `/control`, `/auto`. Luôn kiểm tra đối chiếu `chat.id` và `from_user.id` của người gửi với `TELEGRAM_CHAT_ID` lưu trong `.env`. Từ chối thực thi và cảnh báo nếu không trùng khớp.
* **Lời thoại người trình bày**:
  > "Để tránh trường hợp người lạ xâm nhập và điều khiển xe trái phép qua Telegram, nhóm đã lập trình cơ chế bảo mật phân quyền 2 lớp dựa trên ID người dùng. Mọi lệnh điều khiển di chuyển bắt buộc phải có Chat ID trùng khớp với biến môi trường được cấu hình trước bởi chủ sở hữu."

---

## 🖥️ Slide 14: Đánh giá kết quả thử nghiệm & Hiệu năng
* **Nội dung chính**:
  * **Độ ổn định kết nối nhúng**: Giao tiếp Serial C++ và Rust không xảy ra lỗi nghẽn hay tràn bộ đệm (Buffer overflow) nhờ cơ chế đọc phi tuần tự.
  * **Nhận diện cử chỉ & Giọng nói**: Độ chính xác đạt > 90% trong môi trường đủ sáng và ít tiếng ồn.
  * **Tốc độ phản hồi AI**: Gemini 2.0 Flash trả kết quả trung bình từ 1.1s - 1.6s.
  * **Tính hiệu quả của Safe-Reflex**: Thử nghiệm 20 lần lao xe trực diện vào tường, xe phanh dừng chính xác cả 20 lần trước khi đâm va nhờ cơ chế Rust Safety Core đè lệnh.
* **Lời thoại người trình bày**:
  > "Nhóm đã tiến hành thử nghiệm hệ thống trong nhiều điều kiện thực tế. Kết quả cho thấy độ chính xác của điều khiển bằng cử chỉ và giọng nói đạt hơn 90%. Đặc biệt, cơ chế phản xạ an toàn Safe-Reflex đã vượt qua 100% các bài thử nghiệm va chạm, phanh xe kịp thời trước chướng ngại vật."

---

## 🖥️ Slide 15: Kết luận & Hướng phát triển đề tài
* **Kết luận**:
  * Đồ án đã chế tạo thành công mẫu xe tự hành hoạt động ổn định.
  * Tích hợp thành công Edge AI làm bộ não quyết định hướng lái.
  * Giao diện điều khiển trực quan, mượt mà và đa phương thức (Web, cử chỉ, giọng nói, Telegram).
* **Hướng phát triển**:
  * Tích hợp thêm cánh tay robot kẹp vật thể (Claw) hoạt động tự động dưới sự chỉ đạo của AI Vision.
  * Triển khai chạy các mô hình ngôn ngữ lớn cục bộ (Local SLM như Phi-3, TinyLlama) trực tiếp trên bo mạch Edge (Raspberry Pi 5) để không phụ thuộc vào kết nối mạng internet.
* **Lời kết**: Cảm ơn thầy cô và các bạn đã chú ý lắng nghe!
* **Lời thoại người trình bày**:
  > "Tóm lại, đồ án đã giải quyết được các mục tiêu đặt ra. Trong tương lai, nhóm định hướng sẽ phát triển thêm tính năng của cánh tay kẹp thông minh và tối ưu hóa để AI chạy hoàn toàn offline trên xe. Nhóm chúng em xin chân thành cảm ơn thầy cô đã hướng dẫn và lắng nghe."
