## Xây dựng hệ thống xe tự lái và sofware điều khiển
## 1. System Architecture
-  Dùng mạch ESP32, yêu cầu điều khiển qua Website (Internet) -> mô hình IoT qua Cloud là phù hợp nhất.

- Hệ thống chia làm 3 tầng:

    + Tầng Thiết bị (Device Layer - The Car):
        - Bộ não: ESP32 (Khuyên dùng làm vi xử lý chính vì có WiFi/Bluetooth tích hợp và mạnh hơn Arduino).
        - Vận động: L298N điều khiển 2 (hoặc 4) động cơ DC.
        - Giác quan: HC-SR04 (mắt) + SG90 Servo (cổ xoay để nhìn trái/phải).
    + Tầng Mạng & Server (Network & Cloud Layer):
        - Giao thức: Sử dụng Firebase Realtime Database (Google). Đây là lựa chọn tốt nhất cho sinh viên vì tốc độ phản hồi nhanh (Realtime), dễ cấu hình hơn AWS, và miễn phí.
        - Dữ liệu: Lưu trạng thái điều khiển (Ví dụ: control_signal: "UP", mode: "AUTO").
    + Tầng Người dùng (User Layer - Web App):
        - Giao diện HTML/CSS/JS đơn giản kết nối tới Firebase để gửi lệnh.

## 2. Data Flow
- Điều khiển thủ công
    + User ra lệnh trên websie (nếu chưa có website thì dùng bộ điều khiển trước)
    + `Website` gửi signal cho `Firebase`
    + `ESP32` đang lắng nghe `Firebase` và nhận được signal
    + `ESP32` kích hoạt chân GPIO nối với L298N và xe chạy
- Tự động lái
    + User bật chế độ `Auto move` trên website
    + ESP32 nhận lệnh và đổi chế độ, bỏ qua các lệnh điều khiển bằng tay
    + Vòng lặp:
        - Đọc cảm biến siêu âm (HC-SR04)
        - Gọi d là khoảng cách tới vật phẩm:
            + Nếu d > 10cm: tiếp tục đi thẳng
            + Nếu d < 10cm: dừng và xoay servo kiểm tra trái, phải (nếu trái phải đều có vật thể thì tiến hành `back`)  

## 3. Roadmap
### Giai đoạn 1: Xây dựng phần cứng và xe hoạt động được
- Mục tiêu: Xe có thể chạy 
- Công việc:
    + Lắp ráp khung xe, gắn động cơ DC
    + Kết nối L298N với ESP32 (Cấp nguồn riêng cho L298N bằng pin, nối chung dây GND của pin và ESP32)
    + Viết code test cho xe chạy thử
    + (Thêm phần kết nối với xe qua bluetooth thử)

### Giai đoạn 2: Tích hợp hệ thống cảm biến và tính năng tự lái
- Mục tiêu: Xe cảm nhận được vật cản và né 
- Công việc:
    + Gắn HC-SR04 lên Servo SG90
    + Kết nối với ESP32
    + Viết hàm tính khoảng cách tới vật thể
    + Viết thuật toán né vật cản

### Giai đoạn 3: Kết nối mạng và code backend
- Mục tiêu: ESP32 kết nối Internet và nhận lệnh
- Công việc:
    + Tạo project trên Firebase Console. Chọn "Realtime Database".
    + Cài thư viện Firebase ESP32 Client (hoặc tương tự) vào Arduino IDE.
    + Viết code ESP32 kết nối WiFi.
    + Viết code test: ESP32 đọc giá trị từ Firebase và in ra Serial Monitor.

### Giai đoạn 4: Xây dựng website điều khiển 
- Mục tiêu: Giao diện than thiện với người dùng, hỗ trợ điều khiển thay vì thao tác trực tiếp với database.
- Công việc:
    + Viết file index.html (có thêm css thì càng tốt): Tạo các nút bấm (lên, xuống, trái, phải, Auto/Manual)
    + Viết script.js: Sử dụng Firebase SDK cho Web (JavaScript).Logic: Khi bấm nút "Tiến" $\rightarrow$ set(ref(db, 'command'), 'FORWARD'). Khi thả nút $\rightarrow$ set(ref(db, 'command'), 'STOP').
    + Deploy lên Firebase Hosting (miễn phí và chung hệ sinh thái) hoặc chạy local.