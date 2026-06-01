# AutoClaw - AI Architecture Manifest

This file outlines the technical system architecture, data flows, and design decisions of AutoClaw. It is formatted in JSON to give AI models a clean structural map of how components interact.

```json 
{
  "system_architecture": {
    "layers": {
      "layer_1_client_browser": {
        "responsibility": "Expose human-interactive dashboards, capture user inputs, process client-side AI (MediaPipe), handle voice parsing, and display AI analysis and visual feedback.",
        "communication": "HTTP fetch commands and GET /ai_analyze requests to the Flask API server.",
        "scripts_execution_order": [
          "api.js: Defines the global shared window.AppState namespace, window.cmd(), and window.toggleAuto(). Classic script.",
          "ui.js: Manages status poller, keyboard shortcuts (press 'G' to trigger AI analysis), DOM logs, display of AI subtitles, and D-pad glowing guidance system. Classic script.",
          "voice.js: Controls browser-native Web Speech API, processes Vietnamese audio transcripts, mapping triggers ('ai phân tích', 'quét vật cản') to window.triggerAiAnalyze(). Classic IIFE script.",
          "hand_tracking.js: Leverages GPU-delegate MediaPipe Vision Tasks. Maps Victory (✌) gesture to window.triggerAiAnalyze() under a 3-second cooldown."
        ],
        "state_management": {
          "namespace": "window.AppState",
          "rationale": "Allows classical scripts and ES modules to share runtime variables (e.g. isAuto) without standard module imports which are restricted by cross-origin policies on classic scripts."
        }
      },
      "layer_2_backend_flask_rust_core": {
        "responsibility": "Run web endpoints (including /ai_analyze), capture camera snapshots, manage direct hardware serial communication with high safety checks, and host a Telegram Bot on a background daemon thread for remote monitoring/control. Supports a camera device configured via CAMERA_INDEX.",
        "concurrency_model": "Python Flask handles requests dynamically while delegating the serial device exclusively to the compiled Rust library (autoclaw_core) via PyO3. If compiled Rust is not present, Flask falls back to autoclaw_mock.py. When vehicle auto mode is set to 'ai', Flask calls `robot.start_agent(api_key)` to launch a background Tokio loop in Rust (`agent::run_agent_loop`) driving the vehicle using the AutoClaw agent framework and Gemini API.",
        "default_configuration": {
          "serial_port": "/dev/ttyACM0",
          "camera_index": 0,
          "baud_rate": 9600
        },
        "endpoints_logic": {
          "ai_analyze_flow": "Flask captures current video frame via OpenCV, encodes it to base64, gathers latest ultrasonic distance, and passes both to the core's analyze_scene() method. It returns a JSON structure containing image base64, Vietnamese description, and proposed command."
        },
        "rust_thread_safety": {
          "structure": "AutoClaw class",
          "data_sharing": {
            "port_writer": "Arc<Mutex<Box<dyn SerialPort>>> - Locked briefly inside send_command() to safely push raw byte commands.",
            "latest_distance": "Arc<Mutex<f32>> - Updated by background thread, polled by Flask GET /status.",
            "latest_mode": "Arc<Mutex<String>> - Cache variable synchronizing vehicle AUTO/MANUAL state.",
            "latest_ai_log": "Arc<Mutex<String>> - Cache variable holding the real-time decision and status log of the AI agent, exposed via get_ai_log()."
          },
          "background_reader_thread": "Spawns within new(). Runs a continuous loop using BufReader::read_line() to block until newline (\\n) is encountered, avoiding buffer tearing/partial parse bugs.",
          "autoclaw_agent_functions": [
            {
              "name": "control_car",
              "description": "Sends drive command (F, B, L, R, S) down to Arduino with Safety Reflex (overrides F to S if distance < 15cm)"
            },
            {
              "name": "get_distance",
              "description": "Reads the latest cached ultrasonic distance in centimeters"
            },
            {
              "name": "capture_snapshot",
              "description": "Calls local Flask endpoint `/snapshot` (via dynamically read FLASK_PORT env var) to retrieve current camera frame base64 without locking the camera hardware device"
            }
          ]
        }
      },
      "layer_3_firmware_arduino_uno": {
        "responsibility": "Interact directly with sensors and motor drivers. Handle low-level safety reflexes and run a non-blocking autonomous drive loop.",
        "concurrency": "Completely millis() based state machine loop to guarantee high frequency serial polling (~1ms) and responsive safety stops. Avoids delay()."
      }
    }
  },
  "key_pipelines": {
    "gesture_recognition_pipeline": {
      "processing_location": "Client browser using WebGL GPU acceleration.",
      "frame_rate": "25-30 FPS.",
      "algorithms": {
        "priority_1_isOKSign": "Geometric Euclidean check: distance(thumb_tip_4, index_tip_8) < 0.05. Overrides ML predictions.",
        "priority_2_isVictorySign": "Geometric check: Index and Middle finger extended, Ring and Pinky folded, Thumb and Index far apart. Replaces unreliable ML labels.",
        "priority_3_thumb_direction": "Evaluates horizontal displacement of thumb relative to index MCP. Maps to LEFT or RIGHT commands.",
        "priority_4_standard_ml": "Fallback classification labels provided directly by MediaPipe recognizer task (e.g., ILoveYou to toggle auto mode)."
      },
      "anti_spam_cooldowns_ms": {
        "movement_commands": 500,
        "auto_mode_toggle": 2000,
        "camera_snapshot": 3000
      }
    },
    "voice_processing_priority": {
      "api": "Web Speech Recognition API configured for vi-VN.",
      "parsing_rule": "Camera keywords ('nhìn trái/phải/thẳng') are parsed BEFORE movement keywords ('trái/phải/tiến/lùi') to prevent collision errors where 'nhìn trái' is incorrectly matched as a LEFT movement."
    },
    "safety_reflex_pipeline": {
      "path": "Flask route (/control/<cmd>) -> Rust send_command() -> Read cache -> Serial write.",
      "safety_check": "If the command is 'F' (Forward) and latest_distance in Rust cache is < 15.0cm, the command is instantly overridden to 'S' (Stop) before raw bytes are sent to the port. Non-bypassable safety loop."
    },
    "ultrasonic_noise_filtering": {
      "path": "AutoClaw.cpp loop() -> measureDistance() every 50ms.",
      "algorithm": "Consecutive Reading Filter. Autonomous state machine only stops and scans if the distance measures < 20cm (safeDistanceThreshold) for at least 2 consecutive cycles (~100ms), eliminating erratic stops from temporary sonar echo drops."
    },
    "telegram_bot_pipeline": {
      "threading": "Telegram Bot runs bot.infinity_polling() on a Python daemon thread, decoupled from Flask request handling. If TELEGRAM_BOT_TOKEN is not configured, initialization is skipped entirely.",
      "command_routing": {
        "/status": "Reads robot.get_distance() cache and current_auto_mode global variable.",
        "/snapshot": "Calls get_camera_frame() -> cv2.imencode -> io.BytesIO -> bot.send_photo.",
        "/control <cmd>": "Routes through robot.send_command(cmd) which includes Safety Reflex (F -> S if distance < 15cm).",
        "/auto <state>": "Calls the same stop_ai_agent()/start_ai_agent()/robot.send_command() logic as the Flask /auto/<state> endpoint."
      },
      "security_pipeline": {
        "read_commands": "check_read_permission(): If TELEGRAM_CHAT_ID not configured, allow public access. If configured, restrict to matching chat.id or from_user.id.",
        "control_commands": "check_control_permission(): Always requires TELEGRAM_CHAT_ID to be configured AND matching. Prevents unauthorized movement commands."
      },
      "cleanup": "bot.stop_polling() is called in the atexit cleanup() handler before releasing camera and serial resources."
    }
  },
  "design_decisions_rationale": {
    "rust_serial_ownership": {
      "problem": "Python (pyserial) and Rust both trying to access the same port caused race conditions and 'Device Busy' system blockages.",
      "solution": "Rust core owns the Serial connection exclusively. Python accesses it strictly through safe, cross-language PyO3 bindings."
    },
    "http_polling_vs_websockets": {
      "decision": "HTTP polling at 500ms intervals was selected instead of WebSockets.",
      "rationale": "Keeps the application stack simple and lightweight without importing greenlet/eventlet dependencies while satisfying real-time LAN requirements (<50ms latency)."
    },
    "camera_snapshot_on_demand": {
      "problem": "MJPEG stream saturated Pi 4 CPU and bandwidth, causing high latency on other pipelines.",
      "solution": "Snapshot mode captures a single frame on-demand (triggered via Victory ✌ gesture). Saves ~99% camera resource load."
    }
  },
  "known_limitations": {
    "concurrency": "No multi-user authentication. Conflicting controls will occur if multiple users access the dashboard simultaneously.",
    "sensor_blind_spots": "The HC-SR04 ultrasonic sensor is front-facing only. Obstacles coming from the sides cannot be detected by the autonomous logic.",
    "voice_compatibility": "Web Speech API is restricted to Chromium browsers (Chrome, Edge) and requires HTTPS (or insecure origin flags enabled in localhost).",
    "snapshot_latency": "Pi Camera requires warming up (3 frame capture cycles), causing the first snapshot request to return a slightly darker frame.",
    "telegram_bot": "Bot uses long-polling (not webhooks). If running behind a strict firewall blocking outbound HTTPS to api.telegram.org, the bot will fail to connect. Bot thread is daemon — it will be killed when the main Flask process exits."
  },
  "gemini_vision_ai_integration": {
    "module": "core/src/gemini.rs (Rust) / autoclaw_mock.py (Python Mock fallback)",
    "status": "Active (fully integrated and active)",
    "flow": {
      "input": "Base64 encoded JPEG camera frame + current sonar distance (f32)",
      "target_model": "gemini-2.0-flash",
      "mechanism": "Sends image and distance to Gemini API using reqwest blocking client with a 10s timeout, or falls back to mock logic.",
      "system_prompt": "Instructs the model to act as an image-analysis assistant, describing obstacles in Vietnamese (<15 words) and suggesting a navigation decision ('F', 'B', 'L', 'R', 'S').",
      "validation": "Validates that the returned JSON contains 'description' and a valid 'command'. Fallback to 'S' (Stop) on format errors.",
      "trigger_events": {
        "gesture": "Victory (✌) gesture detected via MediaPipe.",
        "keyboard": "Pressing the 'G' key.",
        "voice": "Vietnamese voice commands ('ai phân tích', 'quét vật cản', 'ai xem trước mặt').",
        "loop": "Tokio run_agent_loop running continuously on a Rust background thread (AutoClaw Agent) every 3 seconds when auto mode is 'ai'."
      },
      "feedback_effects": {
        "camera_panel": "Radar-like scanning scanning animation overlay (.camera-wrap.scanning)",
        "dpad_glow": "Visual guide flashes the recommended D-pad button (highlight-suggest) for 5 seconds.",
        "subtitles": "Display of the Vietnamese description inside the AI feedback status box."
      }
    }
  }
}
```
