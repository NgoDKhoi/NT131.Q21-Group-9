// ============================================================
//  motor_test.ino — isolated test script for AutoClaw DC Motors
//
//  Nạp code này vào Arduino để kiểm tra hoạt động độc lập 
//  của cụm bánh trái và cụm bánh phải.
// ============================================================

// Định nghĩa chân cắm L298N
const int pinENA = 5;   // Điều tốc Motor Trái (PWM)
const int pinIN1 = 7;   // Hướng 1 Motor Trái
const int pinIN2 = 8;   // Hướng 2 Motor Trái

const int pinENB = 6;   // Điều tốc Motor Phải (PWM)
const int pinIN3 = 9;   // Hướng 1 Motor Phải
const int pinIN4 = 11;  // Hướng 2 Motor Phải

void setup() {
  Serial.begin(9600);
  
  pinMode(pinENA, OUTPUT);
  pinMode(pinIN1, OUTPUT);
  pinMode(pinIN2, OUTPUT);
  
  pinMode(pinENB, OUTPUT);
  pinMode(pinIN3, OUTPUT);
  pinMode(pinIN4, OUTPUT);
  
  Serial.println("=== BAT DAU KIEM TRA DONG CO ===");
  Serial.println("GHI CHU: Hãy kê cao bánh xe lên để kiểm tra an toàn!");
  delay(2000);
}

void stopAll() {
  analogWrite(pinENA, 0);
  analogWrite(pinENB, 0);
  digitalWrite(pinIN1, LOW);
  digitalWrite(pinIN2, LOW);
  digitalWrite(pinIN3, LOW);
  digitalWrite(pinIN4, LOW);
}

void loop() {
  // ──────────────────────────────────────────────────────────
  // TEST 1: CHỈ QUAY CỤM BÁNH BÊN TRÁI (LEFT SIDE WHEELS)
  // ──────────────────────────────────────────────────────────
  Serial.println("\n>>> TEST 1: KICH HOAT CU M BANH TRAI (TIEN) <<<");
  Serial.println("Chân kích hoạt: ENA (chân 5) = PWM 200, IN1 = HIGH, IN2 = LOW");
  
  stopAll();
  analogWrite(pinENA, 200); // Tốc độ trung bình
  digitalWrite(pinIN1, HIGH);
  digitalWrite(pinIN2, LOW);
  
  delay(4000); // Chạy 4 giây
  
  // ──────────────────────────────────────────────────────────
  // TEST 2: CHỈ QUAY CỤM BÁNH BÊN PHẢI (RIGHT SIDE WHEELS)
  // ──────────────────────────────────────────────────────────
  Serial.println("\n>>> TEST 2: KICH HOAT CU M BANH PHAI (TIEN) <<<");
  Serial.println("Chân kích hoạt: ENB (chân 6) = PWM 200, IN3 = LOW, IN4 = HIGH");
  
  stopAll();
  analogWrite(pinENB, 200); // Tốc độ trung bình
  digitalWrite(pinIN3, LOW);
  digitalWrite(pinIN4, HIGH);
  
  delay(4000); // Chạy 4 giây
  
  // Dừng lại nghỉ 2 giây rồi lặp lại
  stopAll();
  Serial.println("\n--- Tạm dừng 2 giây trước khi lặp lại chu kỳ ---");
  delay(2000);
}
