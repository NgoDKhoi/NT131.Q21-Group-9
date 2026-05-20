#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
AutoClaw Control Panel — Flask Backend v4.0
============================================
Thay đổi so với v3.0:
- Serial hoàn toàn do Rust Core (zeroclaw_core) quản lý
- Xóa toàn bộ pyserial — không còn tranh cổng Serial
- Safety Reflex (< 15cm → Auto-STOP) chạy trong Rust
- Config đọc từ .env qua python-dotenv
"""

import os
import logging
import atexit
import threading

from flask import Flask, render_template, Response, jsonify
from dotenv import load_dotenv
import cv2

# ── Load .env trước mọi thứ ────────────────────────────────
load_dotenv()

SERIAL_PORT = os.getenv("SERIAL_PORT",  "/dev/ttyACM0")
BAUD_RATE   = int(os.getenv("BAUD_RATE",  "9600"))
FLASK_PORT  = int(os.getenv("FLASK_PORT", "5000"))
SECRET_KEY  = os.getenv("SECRET_KEY",   "zeroclaw_uit_2026")

CAMERA_INDEX  = 0
CAMERA_WIDTH  = 320
CAMERA_HEIGHT = 240
CAMERA_FPS    = 30

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
#  RUST CORE — ZeroClaw (PyO3)
#  Serial được mở 1 lần duy nhất tại đây.
#  Background reader thread spawn bên trong __init__ của Rust.
# ══════════════════════════════════════════════════════════════
robot = None

def init_robot() -> bool:
    global robot
    try:
        from zeroclaw_core import ZeroClaw
        robot = ZeroClaw(SERIAL_PORT, BAUD_RATE)
        logger.info(f"✅ ZeroClaw Core connected: {SERIAL_PORT} @ {BAUD_RATE}")
        return True
    except ImportError:
        logger.error("❌ zeroclaw_core not found. Chạy: cd core && maturin develop")
        return False
    except RuntimeError as e:
        logger.error(f"❌ Serial init failed: {e}")
        logger.info("💡 Kiểm tra: ls /dev/ttyACM*")
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

@app.route("/auto/<state>")
def auto_mode(state: str):
    """Bật/tắt chế độ tự lái."""
    if state not in ("on", "off"):
        return jsonify({"status": "error", "message": "Dùng on hoặc off"}), 400

    if robot is None:
        return jsonify({"status": "error", "message": "Arduino chưa kết nối"}), 503

    try:
        robot.send_command("A" if state == "on" else "M")
        logger.info(f"🤖 Auto mode: {state.upper()}")
        return jsonify({"status": "success", "mode": state})
    except RuntimeError as e:
        return jsonify({"status": "error", "message": str(e)}), 500

@app.route("/status")
def get_status():
    """
    Trả về trạng thái robot cho frontend polling (mỗi 500ms).
    Đọc từ Rust cache — non-blocking.
    """
    if robot is None:
        return jsonify({"distance": "---", "auto": False})

    try:
        dist = robot.get_distance()
        # 999.0 = giá trị khởi tạo "chưa có data"
        dist_str = "---" if dist >= 999.0 else f"{dist:.1f}"
        return jsonify({
            "distance": dist_str,
            "auto": robot.is_auto_mode(),
        })
    except Exception as e:
        logger.error(f"get_status error: {e}")
        return jsonify({"distance": "---", "auto": False})

@app.route("/snapshot")
def snapshot():
    """
    Chụp 1 frame từ Pi Camera và trả về JPEG.
    Gọi bởi hand_tracking.js khi detect Victory gesture (✌).
    """
    if camera is None:
        return jsonify({"error": "Camera chưa khởi tạo"}), 503

    with camera_lock:
        ok, frame = camera.read()

    if not ok:
        logger.warning("⚠ Camera read failed")
        return jsonify({"error": "Capture thất bại"}), 500

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

# ══════════════════════════════════════════════════════════════
#  CLEANUP
# ══════════════════════════════════════════════════════════════

def cleanup():
    logger.info("🧹 Cleanup...")
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