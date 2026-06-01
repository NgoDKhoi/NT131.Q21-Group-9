# Hướng dẫn Tìm hiểu về AI Agent & ZeroClaw Framework 🤖
*Dự án xe tự hành thông minh AutoClaw - Khoa Mạng Máy Tính & Truyền Thông (UIT)*

Tài liệu này giải thích chi tiết về mặt lý thuyết và kiến trúc lập trình của **AI Agent tự trị (Autonomous AI Agent)** và framework **ZeroClaw (lõi tự trị AutoClaw)** được ứng dụng trong đồ án.

---

## 📁 Mục lục
1. [Khái niệm AI Agent tự trị là gì?](#khái-niệm)
2. [Sự khác biệt giữa AI truyền thống và AI Agent](#khác-biệt)
3. [Framework ZeroClaw / AutoClaw Agent hoạt động như thế nào?](#kiến-trúc-zeroclaw)
4. [Tác dụng thực tế của AI Agent trong đồ án này](#tác-dụng)

---

<a name="khái-niệm"></a>
## 1. Khái niệm AI Agent tự trị là gì?

**AI Agent (Đại lý trí tuệ nhân tạo tự trị)** là một thực thể phần mềm thông minh có khả năng:
* **Perceive (Cảm nhận):** Thu thập dữ liệu từ môi trường thực tế (trong đồ án này là hình ảnh từ camera và khoảng cách từ cảm biến siêu âm).
* **Reason & Plan (Tư duy & Lập kế hoạch):** Sử dụng Mô hình ngôn ngữ lớn (LLM - cụ thể là Gemini 2.0 Flash API) làm bộ não để phân tích thông tin thu thập được và lập ra chuỗi hành động tối ưu để đạt được mục tiêu.
* **Act (Hành động):** Sử dụng các công cụ (**Tools**) được lập trình sẵn để tác động ngược lại môi trường (điều khiển xe di chuyển, rẽ hướng, dừng khẩn cấp).

Một AI Agent tự trị hoạt động theo vòng lặp đóng **Loop (Cảm nhận $\rightarrow$ Suy nghĩ $\rightarrow$ Quyết định $\rightarrow$ Hành động)** liên tục mà không cần sự can thiệp trực tiếp của con người.

---

<a name="khác-biệt"></a>
## 2. Sự khác biệt giữa AI truyền thống và AI Agent

| Đặc điểm | AI truyền thống (Classical AI/ML) | AI Agent tự trị (Đồ án AutoClaw) |
|---|---|---|
| **Cơ chế ra quyết định** | Phụ thuộc vào các luật cứng viết sẵn (Rule-based, ví dụ `if-else` trên Arduino). | Tự động lập kế hoạch linh hoạt dựa trên ngữ cảnh thực tế (Cognitive Reasoning). |
| **Sử dụng công cụ** | Không có khả năng tự gọi công cụ ngoài. | Có khả năng tự quyết định khi nào cần sử dụng các công cụ (**Tools**) như đo khoảng cách, chụp ảnh, lái xe. |
| **Khả năng thích ứng** | Gặp vật cản lạ không có trong lập trình `if-else` sẽ bị kẹt hoặc đâm vào. | Có khả năng hiểu các vật thể phức tạp (chướng ngại vật mềm, dải phân cách, con người) nhờ Vision LLM và tìm đường tránh tối ưu. |

---

<a name="kiến-trúc-zeroclaw"></a>
## 3. Framework ZeroClaw / AutoClaw Agent hoạt động như thế nào?

**ZeroClaw** (tên gọi của lõi phát triển ban đầu, nay đồng bộ thương hiệu thành **AutoClaw Agent**) là framework được viết bằng ngôn ngữ **Rust**, hoạt động dựa trên mô hình chạy đa nhiệm không đồng bộ (Tokio runtime).

Sơ đồ kiến trúc hoạt động của Agent chạy trong Rust Core:

```
   [ Raspberry Pi Camera ]  <─── (Chụp ảnh) ───┐
                                              │
  [ Cảm biến siêu âm ]     <─── (Đọc khoảng cách) ┼─── [ AutoClaw Rust Agent ] ───> [ Gemini 2.0 API ]
                                              │             (Tokio Loop)
   [ Mạch cầu H L298N ]     <─── (Lệnh Serial) ──┘
```

### Cách thức hoạt động trong mã nguồn:
1. **Spawn Thread:** Khi người dùng bật chế độ tự hành AI (`/auto/ai`), server Flask gọi phương thức `robot.start_agent()` sang Rust. Rust sẽ khởi chạy một luồng độc lập (`std::thread`) chứa vòng lặp Tokio không đồng bộ để tránh block ứng dụng Python chính.
2. **Đăng ký Tools (Công cụ):** Agent trong Rust định nghĩa sẵn các công cụ mà nó có quyền dùng:
   * **`CaptureSnapshotTool`:** Chụp ảnh từ camera xe.
   * **`GetDistanceTool`:** Đọc khoảng cách từ cảm biến siêu âm.
   * **`ControlCarTool`:** Gửi lệnh điều khiển bánh xe (`F`, `B`, `L`, `R`, `S`).
3. **Vòng lặp Quyết định:**
   * Agent tự động kích hoạt `CaptureSnapshotTool` và `GetDistanceTool`.
   * Agent gửi dữ liệu dạng đa phương thức (Multimodal: hình ảnh + số đo khoảng cách) lên API Gemini.
   * Gemini phân tích và trả về cấu trúc lệnh dạng JSON.
   * Agent trích xuất JSON, tự động gọi `ControlCarTool` truyền lệnh điều hướng xuống cho xe chạy thực tế qua Serial.

---

<a name="tác-dụng"></a>
## 4. Tác dụng thực tế của AI Agent trong đồ án này

Việc đưa AI Agent tự trị vào đồ án NT131 mang lại giá trị công nghệ thực tế và học thuật vượt trội so với các đồ án xe tránh vật cản thông thường:

1. **Hiểu ngữ cảnh ngữ nghĩa (Semantic Understanding):** Xe không chỉ biết trước mặt có vật cản khoảng cách bao nhiêu cm, mà còn "nhìn" và "hiểu" vật cản đó là gì (ví dụ: *"Trước mặt là một lon nước ngọt, bên phải có đôi giày nhựa, bên trái trống"*). Từ đó, Gemini đưa ra quyết định di chuyển thông minh hơn.
2. **Tối ưu hóa tài nguyên phần cứng (Resource Optimization):** Nhờ cơ chế Agent thông minh chỉ kích hoạt camera chụp ảnh tĩnh gửi đi phân tích khi cần quyết định hướng lái, xe tiết kiệm được **99%** tài nguyên băng thông mạng và năng lượng xử lý CPU của Raspberry Pi, thay vì phải stream video liên tục.
3. **An toàn nhiều lớp (Multi-layer Safety):** Kết hợp khả năng lập kế hoạch tầm xa của AI Agent ở tầng trên và phản xạ phanh khẩn cấp thời gian thực bằng Rust ở tầng dưới (`Safety Reflex`), đảm bảo robot tự hành an toàn tuyệt đối mà không sợ độ trễ truyền tải Internet.
