#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
AutoClaw Control Panel — Flask Backend v4.0
============================================
Thay đổi so với v3.0:
- Serial hoàn toàn do Rust Core (autoclaw_core) quản lý
- Xóa toàn bộ pyserial — không còn tranh cổng Serial
- Safety Reflex (< 15cm → Auto-STOP) chạy trong Rust
- Config đọc từ .env qua python-dotenv
"""

import os
import logging
import atexit
import threading
import base64
import json
import io

from flask import Flask, render_template, Response, jsonify
from dotenv import load_dotenv
import cv2
import telebot

# ── Load .env trước mọi thứ ────────────────────────────────
load_dotenv()

SERIAL_PORT = os.getenv("SERIAL_PORT",  "/dev/ttyACM0")
BAUD_RATE   = int(os.getenv("BAUD_RATE",  "9600"))
FLASK_PORT  = int(os.getenv("FLASK_PORT", "5000"))
SECRET_KEY  = os.getenv("SECRET_KEY",   "autoclaw_uit_2026")

CAMERA_INDEX  = int(os.getenv("CAMERA_INDEX", "0"))
NGROK_AUTHTOKEN = os.getenv("NGROK_AUTHTOKEN")
CAMERA_WIDTH  = 320
CAMERA_HEIGHT = 240
CAMERA_FPS    = 30

TELEGRAM_BOT_TOKEN = os.getenv("TELEGRAM_BOT_TOKEN")
TELEGRAM_CHAT_ID   = os.getenv("TELEGRAM_CHAT_ID")

# ── Flask ───────────────────────────────────────────────────
app = Flask(__name__)
app.config["SECRET_KEY"] = SECRET_KEY

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
    datefmt="%H:%M:%S",
)
logger = logging.getLogger(__name__)

# ══════════════════════════════════════════════════════════════
#  RUST CORE — AutoClaw (PyO3)
#  Serial được mở 1 lần duy nhất tại đây.
#  Background reader thread spawn bên trong __init__ của Rust.
# ══════════════════════════════════════════════════════════════
robot = None

def init_robot() -> bool:
    global robot
    try:
        # Thử load lõi thực tế biên dịch bằng Rust
        from autoclaw_core import AutoClaw
        robot = AutoClaw(SERIAL_PORT, BAUD_RATE)
        logger.info(f"✅ AutoClaw Core (Rust) connected: {SERIAL_PORT} @ {BAUD_RATE}")
        return True
    except ImportError:
        logger.warning("💡 autoclaw_core (Rust) không tìm thấy. Chuyển sang chế độ giả lập (Mock)...")
        try:
            from autoclaw_mock import AutoClawMock
            robot = AutoClawMock(SERIAL_PORT, BAUD_RATE)
            logger.info("🤖 AutoClaw Mock (Simulation) connected")
            return True
        except ImportError as e:
            logger.error(f"❌ Không thể load cả Lõi Rust và Mock: {e}")
            return False
    except RuntimeError as e:
        logger.error(f"❌ Lõi nhúng báo lỗi khởi tạo: {e}")
        logger.warning("💡 Chuyển sang chế độ giả lập (Mock) để chạy giao diện...")
        try:
            from autoclaw_mock import AutoClawMock
            robot = AutoClawMock(SERIAL_PORT, BAUD_RATE)
            logger.info("🤖 AutoClaw Mock (Simulation) connected")
            return True
        except ImportError as e2:
            logger.error(f"❌ Không thể load Mock fallback: {e2}")
            return False

# ══════════════════════════════════════════════════════════════
#  CAMERA — Snapshot on demand (thay thế MJPEG stream liên tục)
#  Trigger: Victory gesture (✌) từ MediaPipe trên browser
#  Lý do: MJPEG ngốn bandwidth Pi → chỉ capture khi cần
# ══════════════════════════════════════════════════════════════
camera      = None
camera_lock = threading.Lock()   # VideoCapture không thread-safe

def init_camera() -> bool:
    global camera
    try:
        camera = cv2.VideoCapture(CAMERA_INDEX)
        if not camera.isOpened():
            logger.error("❌ Camera không tìm thấy")
            return False
        camera.set(cv2.CAP_PROP_FRAME_WIDTH,  CAMERA_WIDTH)
        camera.set(cv2.CAP_PROP_FRAME_HEIGHT, CAMERA_HEIGHT)
        camera.set(cv2.CAP_PROP_FPS,          CAMERA_FPS)
        # Warm-up: đọc vài frame đầu để sensor ổn định
        for _ in range(3):
            camera.read()
        logger.info(f"✅ Camera: {CAMERA_WIDTH}x{CAMERA_HEIGHT} @ {CAMERA_FPS}fps")
        return True
    except Exception as e:
        logger.error(f"❌ Camera error: {e}")
        return False

# ══════════════════════════════════════════════════════════════
#  NGROK TUNNEL (pyngrok)
# ══════════════════════════════════════════════════════════════
def init_ngrok():
    if not NGROK_AUTHTOKEN or NGROK_AUTHTOKEN == "YOUR_NGROK_AUTHTOKEN_HERE":
        logger.info("💡 Ngrok: NGROK_AUTHTOKEN chưa được cấu hình hoặc sử dụng mặc định. Bỏ qua tự động mở tunnel.")
        return

    try:
        from pyngrok import ngrok
        ngrok.set_auth_token(NGROK_AUTHTOKEN)
        # Tự động đóng các tunnel cũ nếu có
        tunnels = ngrok.get_tunnels()
        for t in tunnels:
            ngrok.disconnect(t.public_url)
            
        public_url = ngrok.connect(FLASK_PORT)
        logger.info("=" * 55)
        logger.info(f"🚀 Ngrok Tunnel Active: {public_url}")
        logger.info(f"📱 Mở link HTTPS này trên điện thoại để cấp quyền Micro + Camera:")
        logger.info(f"   {public_url}")
        logger.info("=" * 55)
    except ImportError:
        logger.warning("⚠ pyngrok chưa được cài đặt. Không thể tự khởi động Ngrok. Chạy: pip install pyngrok")
    except Exception as e:
        logger.error(f"❌ Lỗi khởi động Ngrok: {e}")

# ══════════════════════════════════════════════════════════════
#  FLASK ROUTES
# ══════════════════════════════════════════════════════════════

@app.route("/")
def index():
    return render_template("index.html")

@app.route("/control/<cmd>")
def control(cmd: str):
    """
    Nhận lệnh từ frontend, gọi Rust Core.
    Safety Reflex tự động xử lý bên trong send_command().
    """
    VALID = {"F", "B", "L", "R", "S", "1", "2", "3"}
    if cmd not in VALID:
        return jsonify({"status": "error", "message": f"Lệnh không hợp lệ: {cmd}"}), 400

    if robot is None:
        return jsonify({"status": "error", "message": "Arduino chưa kết nối"}), 503

    try:
        robot.send_command(cmd)
        logger.info(f"📡 CMD: {cmd}")
        return jsonify({"status": "success", "command": cmd})
    except RuntimeError as e:
        logger.error(f"Serial write error: {e}")
        return jsonify({"status": "error", "message": str(e)}), 500

# ── AI Agent Loop (AutoClaw) ─────────────────────────────────
current_auto_mode = "off"
ai_agent_thread = None
current_ai_log = "Đang chờ lệnh (✌ hoặc khẩu lệnh \"AI phân tích\")..."

def get_camera_frame():
    frame = None
    if camera is not None:
        with camera_lock:
            ok, read_frame = camera.read()
            if ok:
                frame = read_frame
            else:
                logger.warning("⚠ Camera read failed, using mock frame instead")
                
    if frame is None:
        import numpy as np
        import time
        frame = np.zeros((240, 320, 3), dtype=np.uint8)
        cv2.putText(frame, "MOCK PI CAMERA", (60, 90), cv2.FONT_HERSHEY_SIMPLEX, 0.7, (136, 255, 0), 2)
        cv2.putText(frame, "STATUS: LIVE (MOCK)", (50, 130), cv2.FONT_HERSHEY_SIMPLEX, 0.5, (0, 207, 255), 1)
        ts_str = time.strftime("%H:%M:%S")
        cv2.putText(frame, f"TIME: {ts_str}", (50, 160), cv2.FONT_HERSHEY_SIMPLEX, 0.5, (255, 207, 0), 1)
        cv2.drawMarker(frame, (160, 120), (0, 0, 255), cv2.MARKER_CROSS, 20, 2)
    return frame

def get_camera_frame_base64() -> str:
    frame = get_camera_frame()
    _, buf = cv2.imencode(".jpg", frame, [cv2.IMWRITE_JPEG_QUALITY, 90])
    return base64.b64encode(buf.tobytes()).decode("utf-8")

def start_ai_agent() -> bool:
    global current_ai_log
    api_key = os.getenv("GEMINI_API_KEY")
    if not api_key:
        logger.error("❌ GEMINI_API_KEY chưa được cấu hình trong file .env")
        current_ai_log = "Lỗi: Chưa cấu hình GEMINI_API_KEY trong file .env"
        return False
        
    if robot is None:
        logger.error("❌ Robot/Serial chưa được kết nối")
        current_ai_log = "Lỗi: Robot/Serial chưa được kết nối"
        return False
        
    try:
        robot.start_agent(api_key)
        current_ai_log = "AutoClaw Rust Agent đang hoạt động tuần tra tự trị..."
        return True
    except Exception as e:
        logger.error(f"❌ Không thể khởi động AutoClaw Agent: {e}")
        current_ai_log = f"Lỗi khởi động Agent: {e}"
        return False

def stop_ai_agent():
    global current_ai_log
    if robot is not None:
        try:
            robot.stop_agent()
            current_ai_log = "AutoClaw Rust Agent đã dừng."
        except Exception as e:
            logger.error(f"❌ Không thể dừng AutoClaw Agent: {e}")

@app.route("/auto/<state>")
def auto_mode(state: str):
    """Thiết lập chế độ tự lái (off, cpp, ai)."""
    global current_auto_mode
    if state not in ("off", "cpp", "ai"):
        return jsonify({"status": "error", "message": "Mode không hợp lệ. Phải là off, cpp, hoặc ai"}), 400

    if robot is None:
        return jsonify({"status": "error", "message": "Arduino chưa kết nối"}), 503

    try:
        if state == "off":
            stop_ai_agent()
            robot.send_command("M")
            current_auto_mode = "off"
            logger.info("🔴 Auto mode: OFF")
        elif state == "cpp":
            stop_ai_agent()
            robot.send_command("A")
            current_auto_mode = "cpp"
            logger.info("⚙ Auto mode: NATIVE (CPP)")
        elif state == "ai":
            robot.send_command("M")  # Arduino ở chế độ Manual để AI điều khiển trực tiếp
            start_ai_agent()
            current_auto_mode = "ai"
            logger.info("🤖 Auto mode: AI AGENT (RUST/Python)")

        return jsonify({"status": "success", "mode": state})
    except RuntimeError as e:
        return jsonify({"status": "error", "message": str(e)}), 500

@app.route("/status")
def get_status():
    """
    Trả về trạng thái robot cho frontend polling (mỗi 500ms).
    """
    global current_auto_mode
    if robot is None:
        return jsonify({"distance": "---", "auto": "off", "ai_log": "Robot chưa kết nối"})

    try:
        dist = robot.get_distance()
        dist_str = "---" if dist >= 999.0 else f"{dist:.1f}"
        
        # Đồng bộ trạng thái từ Arduino nếu Arduino đổi mode tự phát (ví dụ: bấm nút bấm vật lý)
        is_arduino_auto = robot.is_auto_mode()
        if is_arduino_auto:
            if current_auto_mode != "cpp":
                current_auto_mode = "cpp"
                stop_ai_agent()
        else:
            if current_auto_mode == "cpp":
                current_auto_mode = "off"
                
        return jsonify({
            "distance": dist_str,
            "auto": current_auto_mode,
            "ai_log": current_ai_log,
        })
    except Exception as e:
        logger.error(f"get_status error: {e}")
        return jsonify({"distance": "---", "auto": current_auto_mode, "ai_log": current_ai_log})

@app.route("/snapshot")
def snapshot():
    """
    Chụp 1 frame từ Pi Camera và trả về JPEG.
    """
    frame = get_camera_frame()
    _, buf = cv2.imencode(".jpg", frame, [cv2.IMWRITE_JPEG_QUALITY, 90])
    logger.info("📸 Snapshot captured")
    return Response(
        buf.tobytes(),
        mimetype="image/jpeg",
        headers={
            "Cache-Control": "no-store",
            "X-Timestamp": str(int(__import__("time").time())),
        },
    )

@app.route("/ai_analyze")
def ai_analyze():
    """
    Chụp ảnh từ camera, mã hóa Base64 và gửi lên để phân tích qua Gemini API.
    """
    api_key = os.getenv("GEMINI_API_KEY")
    if not api_key:
        logger.error("❌ GEMINI_API_KEY chưa được cấu hình trong file .env")
        return jsonify({
            "status": "error",
            "message": "Chưa cấu hình GEMINI_API_KEY trong file .env"
        }), 400

    if robot is None:
        return jsonify({"status": "error", "message": "Arduino/Robot chưa kết nối"}), 503

    image_base64 = get_camera_frame_base64()

    try:
        decision_str = robot.analyze_scene(image_base64, api_key)
        decision = json.loads(decision_str)
        logger.info(f"🤖 AI Decision: {decision}")
        return jsonify({
            "status": "success",
            "image": f"data:image/jpeg;base64,{image_base64}",
            "description": decision.get("description", "Không có mô tả"),
            "command": decision.get("command", "S")
        })
    except Exception as e:
        logger.error(f"❌ Lỗi phân tích AI: {e}")
        return jsonify({"status": "error", "message": str(e)}), 500

# ══════════════════════════════════════════════════════════════
#  TELEGRAM BOT INTEGRATION
# ══════════════════════════════════════════════════════════════
bot = None
bot_thread = None

def check_control_permission(message):
    if not TELEGRAM_CHAT_ID or TELEGRAM_CHAT_ID == "YOUR_TELEGRAM_CHAT_ID_HERE":
        return False, "Chức năng điều khiển đã bị vô hiệu hóa vì lý do bảo mật (chưa cấu hình TELEGRAM_CHAT_ID)."
    
    if str(message.chat.id) == TELEGRAM_CHAT_ID or str(message.from_user.id) == TELEGRAM_CHAT_ID:
        return True, ""
    
    return False, "Bạn không có quyền điều khiển xe robot này!"

def check_read_permission(message):
    if not TELEGRAM_CHAT_ID or TELEGRAM_CHAT_ID == "YOUR_TELEGRAM_CHAT_ID_HERE":
        return True, "" # Public access
    
    if str(message.chat.id) == TELEGRAM_CHAT_ID or str(message.from_user.id) == TELEGRAM_CHAT_ID:
        return True, ""
    
    return False, "Bạn không có quyền truy cập bot này!"

def init_telegram_bot():
    global bot, bot_thread
    if not TELEGRAM_BOT_TOKEN or TELEGRAM_BOT_TOKEN == "YOUR_TELEGRAM_BOT_TOKEN_HERE":
        logger.info("💡 Telegram Bot: TELEGRAM_BOT_TOKEN chưa được cấu hình. Bỏ qua.")
        return

    try:
        bot = telebot.TeleBot(TELEGRAM_BOT_TOKEN, parse_mode="Markdown")
        logger.info("🤖 Khởi tạo Telegram Bot...")

        @bot.message_handler(commands=["start", "help"])
        def handle_start_help(message):
            allowed, err = check_read_permission(message)
            if not allowed:
                bot.reply_to(message, err)
                return
            help_text = (
                "🤖 *AutoClaw Telegram Bot* 🤖\n"
                "Hệ thống điều khiển & giám sát xe robot từ xa.\n\n"
                "*Các lệnh truy cập công khai:*\n"
                "/start hoặc /help - Hiển thị hướng dẫn này\n"
                "/status - Xem trạng thái cảm biến và chế độ tự lái\n"
                "/snapshot - Chụp ảnh thời gian thực từ camera xe\n\n"
                "*Các lệnh điều khiển (yêu cầu cấu hình Chat ID):*\n"
                "/control <Lệnh> - Di chuyển xe thủ công (F, B, L, R, S)\n"
                "/auto <Chế độ> - Thiết lập chế độ tự động (off, cpp, ai)"
            )
            bot.reply_to(message, help_text)

        @bot.message_handler(commands=["status"])
        def handle_status(message):
            allowed, err = check_read_permission(message)
            if not allowed:
                bot.reply_to(message, err)
                return
            
            if robot is None:
                bot.reply_to(message, "Robot/Serial chưa kết nối.")
                return
            
            try:
                dist = robot.get_distance()
                dist_str = "---" if dist >= 999.0 else f"{dist:.1f} cm"
                mode = current_auto_mode.upper()
                msg = (
                    f"📊 *AutoClaw Status:*\n"
                    f"• Khoảng cách: `{dist_str}`\n"
                    f"• Chế độ lái: `{mode}`"
                )
                bot.reply_to(message, msg)
            except Exception as e:
                bot.reply_to(message, f"Lỗi lấy trạng thái: {e}")

        @bot.message_handler(commands=["snapshot"])
        def handle_snapshot(message):
            allowed, err = check_read_permission(message)
            if not allowed:
                bot.reply_to(message, err)
                return
            
            bot.send_chat_action(message.chat.id, "upload_photo")
            frame = get_camera_frame()
            ok, buf = cv2.imencode(".jpg", frame, [cv2.IMWRITE_JPEG_QUALITY, 90])
            if ok:
                bio = io.BytesIO(buf.tobytes())
                bio.name = 'snapshot.jpg'
                bot.send_photo(message.chat.id, bio, caption="📸 Ảnh chụp thực tế từ AutoClaw")
            else:
                bot.reply_to(message, "Lỗi chụp/mã hóa hình ảnh.")

        @bot.message_handler(commands=["control"])
        def handle_control(message):
            parts = message.text.split()
            if len(parts) < 2:
                bot.reply_to(message, "Vui lòng nhập lệnh. Ví dụ: `/control F` (F, B, L, R, S)")
                return
            
            cmd = parts[1].upper()
            VALID = {"F", "B", "L", "R", "S"}
            if cmd not in VALID:
                bot.reply_to(message, f"Lệnh không hợp lệ: `{cmd}`. Chỉ chấp nhận: F, B, L, R, S")
                return

            allowed, err = check_control_permission(message)
            if not allowed:
                bot.reply_to(message, err)
                return

            if robot is None:
                bot.reply_to(message, "Lỗi: Robot/Serial chưa kết nối.")
                return

            try:
                robot.send_command(cmd)
                bot.reply_to(message, f"📡 Đã gửi lệnh di chuyển: `{cmd}`")
            except Exception as e:
                bot.reply_to(message, f"Lỗi gửi lệnh: {e}")

        @bot.message_handler(commands=["auto"])
        def handle_auto(message):
            parts = message.text.split()
            if len(parts) < 2:
                bot.reply_to(message, "Vui lòng nhập chế độ. Ví dụ: `/auto cpp` (off, cpp, ai)")
                return
            
            state = parts[1].lower()
            if state not in ("off", "cpp", "ai"):
                bot.reply_to(message, "Chế độ không hợp lệ. Chỉ chấp nhận: off, cpp, ai")
                return

            allowed, err = check_control_permission(message)
            if not allowed:
                bot.reply_to(message, err)
                return

            if robot is None:
                bot.reply_to(message, "Lỗi: Robot/Serial chưa kết nối.")
                return

            global current_auto_mode
            try:
                if state == "off":
                    stop_ai_agent()
                    robot.send_command("M")
                    current_auto_mode = "off"
                elif state == "cpp":
                    stop_ai_agent()
                    robot.send_command("A")
                    current_auto_mode = "cpp"
                elif state == "ai":
                    robot.send_command("M")
                    start_ai_agent()
                    current_auto_mode = "ai"
                bot.reply_to(message, f"🤖 Đã chuyển chế độ lái sang: `{state.upper()}`")
            except Exception as e:
                bot.reply_to(message, f"Lỗi chuyển chế độ: {e}")

        bot_thread = threading.Thread(
            target=bot.infinity_polling,
            daemon=True
        )
        bot_thread.start()
        logger.info("🤖 Telegram Bot đang chạy ngầm (infinity_polling)...")

    except Exception as e:
        logger.error(f"❌ Lỗi khởi động Telegram Bot: {e}")

# ══════════════════════════════════════════════════════════════
#  CLEANUP
# ══════════════════════════════════════════════════════════════

def cleanup():
    logger.info("🧹 Cleanup...")
    stop_ai_agent()
    
    global bot
    if bot is not None:
        try:
            bot.stop_polling()
            logger.info("🤖 Telegram Bot stopped polling.")
        except Exception as e:
            logger.error(f"Lỗi dừng Telegram Bot: {e}")
            
    if robot:
        try:
            robot.send_command("S")
        except Exception:
            pass
    if camera:
        camera.release()
    logger.info("✅ Done.")

atexit.register(cleanup)

# ══════════════════════════════════════════════════════════════
#  MAIN
# ══════════════════════════════════════════════════════════════

if __name__ == "__main__":
    logger.info("=" * 55)
    logger.info("🚀 AutoClaw Control Panel v4.0")
    logger.info("=" * 55)

    init_robot()
    init_camera()
    init_ngrok()
    init_telegram_bot()

    logger.info(f"🌐 http://0.0.0.0:{FLASK_PORT}")
    logger.info("=" * 55)

    # ⚠️ debug=False BẮT BUỘC
    # debug=True restart process → Rust mở Serial 2 lần → crash
    app.run(
        host="0.0.0.0",
        port=FLASK_PORT,
        debug=False,
        threaded=True,
    )