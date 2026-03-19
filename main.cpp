#include <Servo.h>

int ENA = 5;  int ENB = 6;
int IN1 = 7;  int IN2 = 8;
int IN3 = 9;  int IN4 = 11;

Servo myServo;
const int servoPin = 3;
const int trigPin = A5;
const int echoPin = A4;

int speedForward = 180;  
int speedTurn = 255;     
int khoangCachAnToan = 25; 

void setup() {
  Serial.begin(9600);
  
  pinMode(ENA, OUTPUT); pinMode(ENB, OUTPUT);
  pinMode(IN1, OUTPUT); pinMode(IN2, OUTPUT);
  pinMode(IN3, OUTPUT); pinMode(IN4, OUTPUT);
  
  myServo.attach(servoPin);
  pinMode(trigPin, OUTPUT);
  pinMode(echoPin, INPUT);

  myServo.write(90); 
  delay(2000); 
}

int getDistance() {
  digitalWrite(trigPin, LOW);
  delayMicroseconds(2);
  digitalWrite(trigPin, HIGH);
  delayMicroseconds(10);
  digitalWrite(trigPin, LOW);
  
  long duration = pulseIn(echoPin, HIGH, 30000); 
  int dist = duration * 0.034 / 2;
  
  // NẾU BỊ LỖI (0cm) HOẶC XA QUÁ
  if (dist == 0) {
    dist = 999; 
  }
  return dist;
}

void tienLen() {
  analogWrite(ENA, speedForward); analogWrite(ENB, speedForward);
  digitalWrite(IN1, HIGH); digitalWrite(IN2, LOW);
  digitalWrite(IN3, LOW);  digitalWrite(IN4, HIGH);
}

void luiLai() {
  analogWrite(ENA, speedForward); analogWrite(ENB, speedForward);
  digitalWrite(IN1, LOW);  digitalWrite(IN2, HIGH);
  digitalWrite(IN3, HIGH); digitalWrite(IN4, LOW);
}

void dungLai() {
  analogWrite(ENA, 0); analogWrite(ENB, 0);
}

void reTrai() {
  analogWrite(ENA, speedTurn); analogWrite(ENB, speedTurn);
  digitalWrite(IN1, LOW);  digitalWrite(IN2, HIGH); 
  digitalWrite(IN3, LOW);  digitalWrite(IN4, HIGH); 
}

void rePhai() {
  analogWrite(ENA, speedTurn); analogWrite(ENB, speedTurn);
  digitalWrite(IN1, HIGH); digitalWrite(IN2, LOW);  
  digitalWrite(IN3, HIGH); digitalWrite(IN4, LOW);  
}

void loop() {
  int distanceFront = getDistance();
  
  Serial.print("Mat dang nhin thay: ");
  Serial.print(distanceFront);
  Serial.println(" cm");

  if (distanceFront > khoangCachAnToan) {
    tienLen(); 
  } 
  else {
    dungLai();
    Serial.println(">>> PHAT HIEN VAT CAN! DUNG LAI!");
    delay(300); 

    myServo.write(0);
    delay(500);
    int distanceRight = getDistance();

    myServo.write(180);
    delay(700);
    int distanceLeft = getDistance();

    myServo.write(90);
    delay(300);

    if (distanceLeft > distanceRight && distanceLeft > khoangCachAnToan) {
      reTrai(); delay(400); dungLai();
    } 
    else if (distanceRight >= distanceLeft && distanceRight > khoangCachAnToan) {
      rePhai(); delay(400); dungLai();
    } 
    else {
      luiLai(); delay(500);
      rePhai(); delay(600); dungLai();
    }
  }
}