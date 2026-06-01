// ============================================================
//  HC-SR04 Ultrasonic Sensor Isolated Test
// ============================================================

const int pinTRIG = A5; // Trigger Pin (Nối vào cổng A5 trên board mở rộng)
const int pinECHO = A4; // Echo Pin (Nối vào cổng A4 trên board mở rộng)

void setup() {
  Serial.begin(9600);
  pinMode(pinTRIG, OUTPUT);
  pinMode(pinECHO, INPUT);
  
  Serial.println("--- BẮT ĐẦU TEST CẢM BIẾN SIÊU ÂM HC-SR04 ---");
  Serial.println("Đấu nối dây: TRIG -> Cổng A5 | ECHO -> Cổng A4");
  delay(1000);
}

void loop() {
  // Phát xung Trigger 10 micro giây
  digitalWrite(pinTRIG, LOW);
  delayMicroseconds(2);
  digitalWrite(pinTRIG, HIGH);
  delayMicroseconds(10);
  digitalWrite(pinTRIG, LOW);

  // Đo thời gian xung phản hồi (Timeout 30ms = 30000us)
  long duration = pulseIn(pinECHO, HIGH, 30000);
  
  if (duration == 0) {
    Serial.println("Lỗi: Không nhận được phản hồi từ cảm biến (Timeout)! Kiểm tra nguồn 5V/GND hoặc cổng tín hiệu.");
  } else {
    // Tính khoảng cách ra cm
    float distance = duration * 0.0343 / 2.0;
    
    Serial.print("Thoi gian xung: ");
    Serial.print(duration);
    Serial.print(" us | Khoang cach do duoc: ");
    Serial.print(distance, 1);
    Serial.println(" cm");
  }
  
  delay(200); // Đo mỗi 200ms
}
