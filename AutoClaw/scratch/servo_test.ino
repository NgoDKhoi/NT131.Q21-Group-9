// ============================================================
//  SG90 Servo Motor Isolated Test
// ============================================================

#include <Servo.h>

Servo testServo;
const int pinSERVO = 10; // Chân Servo (Cổng số 10 trên Arduino Uno)

void setup() {
  Serial.begin(9600);
  
  // Khởi tạo ban đầu
  testServo.attach(pinSERVO);
  testServo.write(90); // Xoay về giữa (90 độ)
  delay(500);
  
  Serial.println("--- BẮT ĐẦU TEST ĐỘNG CƠ SERVO SG90 ---");
  Serial.println("Đấu nối dây: Dây tín hiệu (Cam/Vàng) -> Chân 10");
  Serial.println("Nhập các phím sau trên Serial Monitor (chọn No line ending hoặc Both NL & CR):");
  Serial.println("  L : Xoay sang Trái (0 độ)");
  Serial.println("  R : Xoay sang Phải (180 độ)");
  Serial.println("  M : Xoay về Giữa (90 độ)");
  Serial.println("----------------------------------------");
}

void loop() {
  if (Serial.available() > 0) {
    char cmd = Serial.read();
    
    // Loại bỏ ký tự xuống dòng nếu có
    if (cmd == '\r' || cmd == '\n') return;

    Serial.print("Nhan duoc lenh: ");
    Serial.println(cmd);

    // Kích hoạt Servo trước khi ghi góc
    testServo.attach(pinSERVO);

    if (cmd == 'L' || cmd == 'l') {
      Serial.println("-> Xoay sang TRAI (0 do)");
      testServo.write(0);
      delay(500); // Đợi servo xoay xong
    } 
    else if (cmd == 'R' || cmd == 'r') {
      Serial.println("-> Xoay sang PHAI (180 do)");
      testServo.write(180);
      delay(500);
    } 
    else if (cmd == 'M' || cmd == 'm') {
      Serial.println("-> Xoay ve GIUA (90 do)");
      testServo.write(90);
      delay(500);
    } 
    else {
      Serial.println("Lenh khong hop le! Chi nhap L, R, hoặc M.");
    }

    // Tắt kích hoạt Servo để giải phóng Timer 1
    testServo.detach();
    Serial.println("-> Da dettach Servo de giai phong Timer.");
    Serial.println("Sẵn sàng nhận lệnh tiếp theo...");
  }
}
