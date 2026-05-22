import random
import logging

logger = logging.getLogger(__name__)

class ZeroClaw:
    """Mock implementation of the ZeroClaw Rust Core for UI/Frontend testing."""
    def __init__(self, port, baud):
        logger.info(f"🤖 [MOCK-CORE] Initializing on {port} @ {baud} baud")
        self.distance = 120.0
        self.is_auto = False
        self.servo_pos = 90
        
    def send_command(self, cmd):
        logger.info(f"📡 [MOCK-CORE] Command: {cmd}")
        
        if cmd == 'A':
            self.is_auto = True
        elif cmd == 'M':
            self.is_auto = False
        elif cmd == 'S':
            if self.is_auto:
                self.is_auto = False
                logger.info("🤖 [MOCK-CORE] Auto Drive stopped via emergency stop")
        elif cmd in ('1', '2', '3'):
            positions = {'1': 0, '2': 180, '3': 90}
            self.servo_pos = positions.get(cmd, 90)
            logger.info(f"📸 [MOCK-CORE] Servo camera turned to position {cmd} ({self.servo_pos} deg)")
        elif cmd == 'F':
            if not self.is_auto:
                self.distance = max(5.0, self.distance - 10.0)
        elif cmd == 'B':
            if not self.is_auto:
                self.distance = min(400.0, self.distance + 10.0)
        elif cmd in ('L', 'R'):
            # Manual turning doesn't change distance directly
            pass

    def get_distance(self) -> float:
        if self.is_auto:
            # Simulate auto-navigation distance changes
            if self.distance > 22.0:
                # Moving forward
                self.distance = max(5.0, self.distance - random.uniform(3.0, 8.0))
            else:
                # Blocked! Backing up and turning
                logger.info("🚨 [MOCK-CORE] Obstacle detected! Simulating scanner sweep and redirect...")
                self.distance = random.uniform(80.0, 150.0)  # Reset distance after simulated redirection
        else:
            # Manual mode slight distance noise
            self.distance = max(2.0, min(400.0, self.distance + random.uniform(-0.5, 0.5)))
        return self.distance

    def get_mode(self) -> str:
        return "AUTO" if self.is_auto else "MANUAL"

    def is_auto_mode(self) -> bool:
        return self.is_auto

    def analyze_scene(self, image_base64: str, api_key: str) -> str:
        """Gửi ảnh (mock/real) lên Gemini bằng Python requests (nếu dùng mock core)."""
        import requests
        import json
        
        # prompt & payload
        system_prompt = (
            "Bạn là trợ lý AI phân tích hình ảnh của xe robot ZeroClaw. "
            "Nhìn vào bức ảnh phía trước và đưa ra phân tích theo định dạng JSON sau:\n"
            "{\n"
            '  "description": "Mô tả ngắn gọn vật thể/vật cản chính chắn phía trước bằng tiếng Việt (dưới 15 từ)",\n'
            '  "command": "Lệnh di chuyển đề xuất: \'F\' (Tiến nếu thoáng), \'B\' (Lùi nếu bị chặn sát), \'L\' (Rẽ trái), \'R\' (Rẽ phải), \'S\' (Dừng lại)"\n'
            "}\n"
            "Lưu ý: Chỉ trả về chuỗi JSON hợp lệ, không thêm bất kỳ văn bản giải thích nào khác ngoài JSON. Không bọc trong dấu nháy ```json."
        )
        
        url = f"https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={api_key}"
        headers = {"Content-Type": "application/json"}
        body = {
            "contents": [
                {
                    "parts": [
                        {"text": f"{system_prompt}\n\nThông tin bổ sung: Khoảng cách cảm biến phía trước đo được = {self.distance:.1f} cm."},
                        {
                            "inline_data": {
                                "mime_type": "image/jpeg",
                                "data": image_base64
                            }
                        }
                    ]
                }
            ]
        }
        
        try:
            resp = requests.post(url, headers=headers, json=body, timeout=10)
            resp.raise_for_status()
            resp_json = resp.json()
            
            # Parse text từ candidate
            raw = resp_json["candidates"][0]["content"]["parts"][0]["text"]
            clean_json = raw.strip().replace("```json", "").replace("```", "").strip()
            
            # Validate JSON
            decision = json.loads(clean_json)
            cmd = decision.get("command", "S").upper().strip()
            if cmd not in ("F", "B", "L", "R", "S"):
                cmd = "S"
                
            return json.dumps({
                "description": decision.get("description", "Không xác định"),
                "command": cmd
            }, ensure_ascii=False)
        except Exception as e:
            logger.error(f"[MOCK-CORE] Gemini call failed: {e}. Returning mock analysis.")
            mock_desc = "Cửa sổ/Vật cản mock"
            if self.distance < 15.0:
                mock_desc = "Bức tường quá gần"
                mock_cmd = "B"
            elif self.distance < 30.0:
                mock_desc = "Hộp giấy phía trước"
                mock_cmd = "L"
            else:
                mock_desc = "Đường đi thoáng đãng"
                mock_cmd = "F"
            return json.dumps({
                "description": f"[Mock AI] {mock_desc}",
                "command": mock_cmd
            }, ensure_ascii=False)

    def start_agent(self, api_key: str):
        logger.info("🤖 [MOCK-CORE] Bật ZeroClaw Agent (Mock)")
        self.is_auto = True

    def stop_agent(self):
        logger.info("🤖 [MOCK-CORE] Tắt ZeroClaw Agent (Mock)")
        self.is_auto = False
