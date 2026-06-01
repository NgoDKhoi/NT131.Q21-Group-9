# AutoClaw - AI Project Manifest

This file contains the structured configuration, context, and project information of AutoClaw. It is formatted in JSON to allow an AI model to quickly parse and understand the repository context at the beginning of a new session.

```json
{
  "project_name": "AutoClaw",
  "tagline": "Autonomous Edge AI Robot — Multi-modal control (D-pad, Voice, Hand Gestures, and Autonomous Drive)",
  "context": "Course project for Computer Networks Department, University of Information Technology (UIT) HCMC",
  "hardware": {
    "controllers": {
      "main_server": "Raspberry Pi 4 (8GB RAM)",
      "microcontroller": "Arduino Uno R3"
    },
    "actuators": {
      "motor_driver": "L298N Motor Driver controlling 4 DC motors",
      "servo": "SG90 Servo for panning the ultrasonic sensor / camera"
    },
    "sensors": {
      "ultrasonic": "HC-SR04 for distance measurement",
      "camera": "Raspberry Pi Camera or USB Webcam (configured via CAMERA_INDEX, default is 0; used for manual snapshots and continuous AI Agent Auto Drive loop)"
    },
    "defaults": {
      "SERIAL_PORT": "/dev/ttyACM0",
      "CAMERA_INDEX": 0
    },
    "arduino_pins": {
      "motor_left": {
        "ENA": 5,
        "IN1": 7,
        "IN2": 8
      },
      "motor_right": {
        "ENB": 6,
        "IN3": 9,
        "IN4": 11
      },
      "ultrasonic": {
        "TRIG": 12,
        "ECHO": 3
      },
      "servo": {
        "PIN": 10
      },
      "serial": {
        "baud_rate": 9600,
        "TX": 1,
        "RX": 0
      }
    }
  },
  "software_stack": {
    "languages": ["Python 3.13+", "Rust (2021 edition)", "C++ (Arduino Wiring)"],
    "backend": "Flask 3.0.0",
    "python_dependencies": {
      "Flask": "Web framework",
      "flask-cors": "CORS policy handling",
      "python-dotenv": "Environment variables configuration (.env)",
      "opencv-python-headless": "Image encoding & processing (no GUI overhead)",
      "numpy": "Mock camera frame array generation & image calculations",
      "maturin": "Rust-Python binding compiler",
      "pyngrok": "HTTPS tunneling for Web Speech API compatibility",
      "pyTelegramBotAPI": "Telegram Bot API wrapper for remote monitoring and control"
    },
    "rust_core": {
      "module_name": "autoclaw_core",
      "dependencies": {
        "pyo3": "Python-Rust bridge (v0.21)",
        "serialport": "High-level serial communications (v4.3.0)",
        "reqwest": "HTTP client for API requests",
        "serde": "JSON serialization/deserialization",
        "serde_json": "JSON utility library (v1.0)"
      }
    },
    "frontend": {
      "libraries": [
        "Google Fonts (Orbitron, Share Tech Mono)",
        "MediaPipe Tasks Vision Vision Bundle v0.10.14 (hosted on CDN)"
      ],
      "styles": "Vanilla CSS (Cyberpunk neon retro aesthetic, custom scanlines & CRT glow)"
    }
  },
  "repository_structure": {
    "AutoClaw/app.py": "Flask server backend coordinating camera, local commands, and loading Rust serial engine",
    "AutoClaw/requirements.txt": "Python dependencies listing",
    "AutoClaw/.env": "Local environment variables (SERIAL_PORT, BAUD_RATE, FLASK_PORT, SECRET_KEY, NGROK_AUTHTOKEN, GEMINI_API_KEY, CAMERA_INDEX, TELEGRAM_BOT_TOKEN, TELEGRAM_CHAT_ID)",
    "AutoClaw/AutoClaw.cpp": "Arduino Uno firmware carrying non-blocking obstacle avoidance state machine (millis-based)",
    "AutoClaw/core/Cargo.toml": "Rust build configuration",
    "AutoClaw/core/src/lib.rs": "Rust safety engine containing PyO3 binding, background serial reader thread, and APIs to start/stop the AutoClaw agent",
    "AutoClaw/core/src/gemini.rs": "Vision AI client utilizing Gemini 2.0 Flash API to generate navigation commands",
    "AutoClaw/core/src/agent.rs": "AutoClaw autonomous agent runner running a tokio background loop and querying Gemini Vision AI with combined sonar and camera perception data",
    "AutoClaw/autoclaw_mock.py": "Python mock implementation of AutoClaw for offline and local testing without hardware",
    "AutoClaw/templates/index.html": "Jinja2 dashboard markup containing system status, live camera panel, and D-pad control UI",
    "AutoClaw/static/css/style.css": "Cyberpunk layout styles including reactive glow animations and layout frames",
    "AutoClaw/static/js/api.js": "Command dispatch helper exposing window.cmd() and window.toggleAuto()",
    "AutoClaw/static/js/ui.js": "UI status poller (AbortController backed), keyboard listener, and system logs compiler",
    "AutoClaw/static/js/voice.js": "Web Speech API wrapper for continuous Vietnamese command mapping",
    "AutoClaw/static/js/hand_tracking.js": "MediaPipe Vision-Tasks client tracking hand gestures with custom geometrical filters"
  },
  "control_mappings": {
    "serial_protocol_commands": {
      "F": "Forward (manual movement)",
      "B": "Backward (manual movement)",
      "L": "Left (manual movement)",
      "R": "Right (manual movement)",
      "S": "Stop (clears movement state & re-centers servo)",
      "A": "Auto Mode ON (engages autonomous navigation)",
      "M": "Manual Mode (disengages autonomous navigation & stops motors)",
      "1": "Servo Left (0 degrees)",
      "2": "Servo Right (180 degrees)",
      "3": "Servo Center (90 degrees)"
    },
    "keyboard_shortcuts": {
      "W / ArrowUp": "Move Forward",
      "S / ArrowDown": "Move Backward",
      "A / ArrowLeft": "Turn Left",
      "D / ArrowRight": "Turn Right",
      "X / Space": "Stop Movement",
      "P": "Toggle Auto Drive",
      "V": "Toggle Voice Recognition",
      "G": "Trigger AI Scene Analysis (Gemini Vision)",
      "1, 2, 3": "Pan Servo Left, Right, Center"
    },
    "voice_commands_vietnamese": {
      "nhìn trái / nhìn sang trái": "Servo camera to Left ('1')",
      "nhìn phải / nhìn sang phải": "Servo camera to Right ('2')",
      "nhìn thẳng / nhìn giữa": "Servo camera to Center ('3')",
      "tiến / đi thẳng / tới": "Move Forward ('F')",
      "lùi / xuống": "Move Backward ('B')",
      "quay trái / rẽ trái / sang trái": "Turn Left ('L')",
      "quay phải / rẽ phải / sang phải": "Turn Right ('R')",
      "dừng / đứng lại / stop / dừng lại": "Stop Movement ('S')",
      "tự lái / tự động": "Toggle Auto Drive",
      "ai phân tích / ai xem trước mặt / quét vật cản": "Trigger AI Scene Analysis (calls /ai_analyze endpoint)"
    },
    "hand_gestures_mediapipe": {
      "OK_Sign (👌)": "Move Forward ('F')",
      "Closed_Fist (✊)": "Stop Movement ('S')",
      "Pointing_Up (☝)": "Move Backward ('B')",
      "Victory (✌)": "Trigger AI Scene Analysis (calls /ai_analyze endpoint, custom cooldown 3s)",
      "Thumb_Left (👈)": "Turn Left ('L')",
      "Thumb_Right (👉)": "Turn Right ('R')",
      "ILoveYou (🤟)": "Toggle Auto Drive (custom cooldown 2s)"
    }
  },
  "api_endpoints": {
    "GET /": {
      "description": "Serve the Web Dashboard (index.html)"
    },
    "GET /control/<cmd>": {
      "description": "Send movement/servo commands to the robot",
      "parameters": {
        "cmd": "F, B, L, R, S, 1, 2, 3"
      }
    },
    "GET /auto/<state>": {
      "description": "Set autonomous drive mode state. Chế độ 'ai' kích hoạt AutoClaw Agent chạy bằng Rust (gọi robot.start_agent(api_key)) chạy ngầm vòng lặp Tokio. Trạng thái 'off' hoặc 'cpp' sẽ ngắt agent này bằng robot.stop_agent().",
      "parameters": {
        "state": "off, cpp, ai"
      }
    },
    "GET /status": {
      "description": "Poll current distance telemetry (cm), auto mode status, and running AI agent log text",
      "response_format": {
        "distance": "String (e.g. '25.4' or '---')",
        "auto": "String ('off', 'cpp', 'ai')",
        "ai_log": "String"
      }
    },
    "GET /snapshot": {
      "description": "Capture a single JPEG frame from Pi camera on-demand (falls back to Cyberpunk-styled mock frame if camera initialization failed)"
    },
    "GET /ai_analyze": {
      "description": "Capture a camera frame, encode it, and query Gemini Vision AI to get Vietnamese scene description and suggested driving command",
      "response_format": {
        "status": "String ('success' or 'error')",
        "image": "String (Base64 JPEG Data URL)",
        "description": "String (Short Vietnamese description)",
        "command": "String ('F', 'B', 'L', 'R', 'S')"
      }
    }
  },
  "telegram_bot": {
    "library": "pyTelegramBotAPI (telebot)",
    "status": "Active",
    "threading": "Runs bot.infinity_polling() on a daemon background thread, independent of Flask server.",
    "env_variables": {
      "TELEGRAM_BOT_TOKEN": "Bot token from @BotFather. If missing or placeholder, bot is gracefully skipped.",
      "TELEGRAM_CHAT_ID": "Owner's chat ID for access control. If missing, read commands are public but control commands are disabled."
    },
    "commands": {
      "/start or /help": "Show usage instructions in Vietnamese",
      "/status": "Display current sensor distance and driving mode",
      "/snapshot": "Capture and send JPEG photo from robot camera",
      "/control <cmd>": "Send manual movement command (F, B, L, R, S) via robot.send_command() with Safety Reflex",
      "/auto <state>": "Switch driving mode (off, cpp, ai)"
    },
    "security_model": {
      "read_commands": "Public if no TELEGRAM_CHAT_ID configured, otherwise restricted to matching chat/user ID",
      "control_commands": "Always restricted. Requires TELEGRAM_CHAT_ID to be configured and matching."
    },
    "cleanup": "bot.stop_polling() called in atexit cleanup handler"
  }
}
```
