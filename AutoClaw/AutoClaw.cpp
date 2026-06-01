#include <Servo.h>

// ============================================================
//  AutoClaw Arduino Firmware - Intelligent Scan Obstacle Avoidance
// ============================================================

// ── Pin Configuration (Matches ELEGOO official sample) ──────
const int pinENA = 5;   // Motor Left Enable (PWM)
const int pinIN1 = 7;   // Motor Left Input 1
const int pinIN2 = 8;   // Motor Left Input 2

const int pinENB = 6;   // Motor Right Enable (PWM) - Official Pin
const int pinIN3 = 9;   // Motor Right Input 1
const int pinIN4 = 11;  // Motor Right Input 2 - Official Pin

const int pinTRIG = A5; // HC-SR04 Trigger Pin (Connected to A5 on shield)
const int pinECHO = A4; // HC-SR04 Echo Pin (Connected to A4 on shield)
const int pinSERVO = 3;  // SG90 Servo Control Pin (Connected to Pin 3)

// ── Servo Configuration ─────────────────────────────────────
Servo cameraServo;
const int angleLeft   = 150; // 60 degrees left from center (physically 150 due to inverted mounting)
const int angleCenter = 90;
const int angleRight  = 30;  // 60 degrees right from center (physically 30 due to inverted mounting)

// ── Robot Speed Configuration ───────────────────────────────
const int speedForward  = 230; // Safe high speed (max 255) for forward/backward to avoid brownout
const int speedTurn     = 230; // Safe high speed (max 255) for turning to avoid brownout

// ── State Management ────────────────────────────────────────
enum Mode {
  MODE_MANUAL,
  MODE_AUTO
};

enum AutoState {
  AUTO_FORWARD,
  AUTO_SCAN_STOP,
  AUTO_SCAN_LEFT,
  AUTO_SCAN_RIGHT,
  AUTO_SCAN_DECIDE,
  AUTO_BACKUP,
  AUTO_TURNING_LEFT,
  AUTO_TURNING_RIGHT,
  AUTO_RESUME
};

Mode currentMode = MODE_MANUAL;
AutoState currentAutoState = AUTO_FORWARD;

unsigned long stateStartTime = 0;
unsigned long lastDistanceCheckTime = 0;
unsigned long lastTelemetryTime = 0;
unsigned long lastModeReportTime = 0;

float currentDistance = 999.0;
float distanceLeft = 999.0;
float distanceRight = 999.0;
const float safeDistanceThreshold = 20.0; // cm
int consecutiveObstacleCount = 0;          // Bộ đếm chống nhiễu cảm biến siêu âm

// ── Helper Functions ────────────────────────────────────────

void moveForward(int speed) {
  Serial.println("EXEC:FORWARD");
  analogWrite(pinENA, speed);
  analogWrite(pinENB, speed);
  // Nếu bánh trái chỉ chạy khi IN1=LOW, IN2=HIGH (đã chạy tốt ở lệnh B),
  // ta đổi logic Tiến của bánh trái trùng với logic lùi cũ nhưng đổi phân cực động cơ nếu bị ngược chiều,
  // hoặc thiết lập đúng trạng thái HIGH/LOW để cả 2 bên cùng tiến.
  digitalWrite(pinIN1, HIGH);
  digitalWrite(pinIN2, LOW);
  digitalWrite(pinIN3, LOW);
  digitalWrite(pinIN4, HIGH);
}

void moveBackward(int speed) {
  Serial.println("EXEC:BACKWARD");
  analogWrite(pinENA, speed);
  analogWrite(pinENB, speed);
  digitalWrite(pinIN1, LOW);
  digitalWrite(pinIN2, HIGH);
  digitalWrite(pinIN3, HIGH);
  digitalWrite(pinIN4, LOW);
}

void turnLeft(int speed) {
  Serial.println("EXEC:LEFT");
  analogWrite(pinENA, speed);
  analogWrite(pinENB, speed);
  digitalWrite(pinIN1, LOW);
  digitalWrite(pinIN2, HIGH);
  digitalWrite(pinIN3, LOW);
  digitalWrite(pinIN4, HIGH);
}

void turnRight(int speed) {
  Serial.println("EXEC:RIGHT");
  analogWrite(pinENA, speed);
  analogWrite(pinENB, speed);
  digitalWrite(pinIN1, HIGH);
  digitalWrite(pinIN2, LOW);
  digitalWrite(pinIN3, HIGH);
  digitalWrite(pinIN4, LOW);
}

void stopMotors() {
  Serial.println("EXEC:STOP");
  analogWrite(pinENA, 0);
  analogWrite(pinENB, 0);
  digitalWrite(pinIN1, LOW);
  digitalWrite(pinIN2, LOW);
  digitalWrite(pinIN3, LOW);
  digitalWrite(pinIN4, LOW);
}

// Safe Servo Control to avoid continuous Timer 1 interrupts interfering with pulseIn
void safeServoWrite(int angle) {
  cameraServo.attach(pinSERVO);
  cameraServo.write(angle);
  delay(350); // Allow sufficient time (350ms) for SG90 servo to complete its physical rotation (max 180 deg)
  cameraServo.detach(); // Detach to release Timer 1 so it doesn't block ultrasonic pulseIn readings
}

float measureDistance() {
  digitalWrite(pinTRIG, LOW);
  delayMicroseconds(2);
  digitalWrite(pinTRIG, HIGH);
  delayMicroseconds(10);
  digitalWrite(pinTRIG, LOW);

  // Measure echo pulse with a 30ms timeout (~5m max range) to avoid early timeout under electrical noise
  long duration = pulseIn(pinECHO, HIGH, 30000);
  if (duration == 0) {
    return 999.0;
  }
  float dist = duration * 0.0343 / 2.0;
  
  // Filter unrealistic readings
  if (dist < 2.0 || dist > 400.0) {
    return 999.0;
  }
  return dist;
}

void sendModeReport() {
  if (currentMode == MODE_AUTO) {
    Serial.println("MODE:AUTO");
  } else {
    Serial.println("MODE:MANUAL");
  }
}

// ── Core Logic ──────────────────────────────────────────────

void handleSerialCommands() {
  if (Serial.available() > 0) {
    char cmd = Serial.read();
    
    switch (cmd) {
      // Manual movements (only processed in manual mode)
      case 'F':
        if (currentMode == MODE_MANUAL) moveForward(speedForward);
        break;
      case 'B':
        if (currentMode == MODE_MANUAL) moveBackward(speedForward);
        break;
      case 'L':
        if (currentMode == MODE_MANUAL) turnLeft(speedTurn);
        break;
      case 'R':
        if (currentMode == MODE_MANUAL) turnRight(speedTurn);
        break;
      case 'S':
        stopMotors();
        safeServoWrite(angleCenter); // Center camera for safety / manual use
        // If in auto mode, pressing STOP exits auto mode for safety
        if (currentMode == MODE_AUTO) {
          currentMode = MODE_MANUAL;
          sendModeReport();
        }
        break;

      // Mode switching
      case 'A': // Auto Mode ON
        if (currentMode != MODE_AUTO) {
          currentMode = MODE_AUTO;
          currentAutoState = AUTO_FORWARD;
          sendModeReport();
        }
        break;
      case 'M': // Manual Mode (Auto OFF)
        if (currentMode != MODE_MANUAL) {
          currentMode = MODE_MANUAL;
          stopMotors();
          safeServoWrite(angleCenter); // Re-center camera servo
          sendModeReport();
        }
        break;

      // Servo movements (accessible in any mode)
      case '1':
        safeServoWrite(angleLeft);
        break;
      case '2':
        safeServoWrite(angleRight);
        break;
      case '3':
        safeServoWrite(angleCenter);
        break;
        
      default:
        break;
    }
  }
}

void updateAutoDrive() {
  if (currentMode != MODE_AUTO) return;

  unsigned long now = millis();
  
  switch (currentAutoState) {
    case AUTO_FORWARD:
      moveForward(speedForward); // Drive at full speedForward (200) to ensure sufficient torque on battery power
      if (currentDistance < safeDistanceThreshold) {
        consecutiveObstacleCount++;
        if (consecutiveObstacleCount >= 2) { // Cần ít nhất 2 chu kỳ đo liên tục (100ms) để xác nhận
          stopMotors();
          currentAutoState = AUTO_SCAN_STOP;
          stateStartTime = now;
          consecutiveObstacleCount = 0;
        }
      } else {
        consecutiveObstacleCount = 0; // Reset nếu khoảng cách an toàn trở lại
      }
      break;

    case AUTO_SCAN_STOP:
      if (now - stateStartTime >= 200) { // Stop for 200ms to stabilize
        safeServoWrite(angleLeft);    // Look Left
        currentAutoState = AUTO_SCAN_LEFT;
        stateStartTime = now;
      }
      break;

    case AUTO_SCAN_LEFT:
      if (now - stateStartTime >= 500) { // Wait 500ms for servo to reach 0 degrees and reading to settle
        distanceLeft = currentDistance;  // Record left distance
        safeServoWrite(angleRight);   // Look Right
        currentAutoState = AUTO_SCAN_RIGHT;
        stateStartTime = now;
      }
      break;

    case AUTO_SCAN_RIGHT:
      if (now - stateStartTime >= 700) { // Wait 700ms for sweep to 150 degrees (right side) and settle
        distanceRight = currentDistance; // Record right distance
        safeServoWrite(angleCenter);  // Return to center
        currentAutoState = AUTO_SCAN_DECIDE;
        stateStartTime = now;
      }
      break;

    case AUTO_SCAN_DECIDE:
      if (now - stateStartTime >= 300) { // Wait 300ms for servo to re-center
        // Compare distances and choose direction
        if (distanceLeft <= safeDistanceThreshold && distanceRight <= safeDistanceThreshold) {
          // Both sides blocked, back up and then turn right
          moveBackward(speedForward);
          currentAutoState = AUTO_BACKUP;
          stateStartTime = now;
        } else if (distanceLeft > distanceRight) {
          // Left side is clearer
          turnLeft(speedTurn);
          currentAutoState = AUTO_TURNING_LEFT;
          stateStartTime = now;
        } else {
          // Right side is clearer or equal
          turnRight(speedTurn);
          currentAutoState = AUTO_TURNING_RIGHT;
          stateStartTime = now;
        }
      }
      break;

    case AUTO_BACKUP:
      if (now - stateStartTime >= 500) { // Back up for 500ms
        turnRight(speedTurn);            // Turn right after backing up
        currentAutoState = AUTO_TURNING_RIGHT;
        stateStartTime = now;
      }
      break;

    case AUTO_TURNING_LEFT:
      if (now - stateStartTime >= 600) { // Turn left for 600ms
        stopMotors();
        currentAutoState = AUTO_RESUME;
        stateStartTime = now;
      }
      break;

    case AUTO_TURNING_RIGHT:
      if (now - stateStartTime >= 600) { // Turn right for 600ms
        stopMotors();
        currentAutoState = AUTO_RESUME;
        stateStartTime = now;
      }
      break;

    case AUTO_RESUME:
      if (now - stateStartTime >= 100) { // Brief pause before resuming forward path
        currentAutoState = AUTO_FORWARD;
      }
      break;
  }
}

// ── Setup & Main Loop ────────────────────────────────────────

void setup() {
  Serial.begin(9600);

  // Initialize motor pins
  pinMode(pinENA, OUTPUT);
  pinMode(pinIN1, OUTPUT);
  pinMode(pinIN2, OUTPUT);
  pinMode(pinENB, OUTPUT);
  pinMode(pinIN3, OUTPUT);
  pinMode(pinIN4, OUTPUT);

  // Initialize ultrasonic sensor pins
  pinMode(pinTRIG, OUTPUT);
  pinMode(pinECHO, INPUT);

  // Initialize camera servo SG90
  safeServoWrite(angleCenter); // safeServoWrite internally handles attach, write, delay, and detach safely

  // Stop motors initially
  stopMotors();

  // Send startup signal
  delay(100);
  Serial.println("READY");
}

void loop() {
  unsigned long now = millis();

  // 1. Process incoming commands instantly (low latency)
  handleSerialCommands();

  // 2. Measure distance non-blockingly every 50ms to avoid sensor cross-talk
  if (now - lastDistanceCheckTime >= 50) {
    currentDistance = measureDistance();
    lastDistanceCheckTime = now;
  }

  // 3. Send distance telemetry every 200ms
  if (now - lastTelemetryTime >= 200) {
    Serial.println(currentDistance, 1);
    lastTelemetryTime = now;
  }

  // 4. Periodically report mode state every 1000ms to keep Pi synchronized
  if (now - lastModeReportTime >= 1000) {
    sendModeReport();
    lastModeReportTime = now;
  }

  // 5. Update autonomous navigation state machine if active
  updateAutoDrive();
}
