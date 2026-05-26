use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::time::sleep;
use serde_json::json;
use async_trait::async_trait;
use serialport::SerialPort;
use std::io::Write;

use zeroclaw::tools::{Tool, ToolResult};
use zeroclaw::providers::GeminiModelProvider;
use zeroclaw::Agent;

// ── ControlCarTool ───────────────────────────────────────────
pub struct ControlCarTool {
    pub port_writer: Arc<Mutex<Box<dyn SerialPort>>>,
    pub latest_distance: Arc<Mutex<f32>>,
}

#[async_trait]
impl Tool for ControlCarTool {
    fn name(&self) -> &str {
        "control_car"
    }

    fn description(&self) -> &str {
        "Gửi lệnh di chuyển xuống Arduino để điều khiển xe: F (Tiến), B (Lùi), L (Rẽ trái), R (Rẽ phải), S (Dừng)."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "enum": ["F", "B", "L", "R", "S"],
                    "description": "Lệnh di chuyển cần gửi"
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, anyhow::Error> {
        let cmd = args.get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("S");

        // Safety Reflex: Tiến (F) nhưng khoảng cách phía trước quá gần (< 15cm) -> chuyển thành Dừng (S)
        let final_cmd = if cmd == "F" {
            let dist = *self.latest_distance.lock().unwrap();
            if dist < 15.0 {
                println!("🚨 [Rust Agent Safety Reflex] Phát hiện vật cản {:.1}cm < 15cm -> Tự động chuyển F thành S!", dist);
                "S"
            } else {
                cmd
            }
        } else {
            cmd
        };

        let mut port = self.port_writer.lock().unwrap();
        port.write_all(final_cmd.as_bytes())?;
        port.flush()?;

        Ok(ToolResult {
            success: true,
            output: format!("Gửi lệnh thành công: {}", final_cmd),
            error: None,
        })
    }
}

// ── GetDistanceTool ──────────────────────────────────────────
pub struct GetDistanceTool {
    pub latest_distance: Arc<Mutex<f32>>,
}

#[async_trait]
impl Tool for GetDistanceTool {
    fn name(&self) -> &str {
        "get_distance"
    }

    fn description(&self) -> &str {
        "Đọc khoảng cách hiện tại phía trước xe từ cảm biến siêu âm (đơn vị: cm)."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn execute(&self, _args: serde_json::Value) -> Result<ToolResult, anyhow::Error> {
        let dist = *self.latest_distance.lock().unwrap();
        Ok(ToolResult {
            success: true,
            output: format!("{:.1}", dist),
            error: None,
        })
    }
}

// ── CaptureSnapshotTool ──────────────────────────────────────
pub struct CaptureSnapshotTool;

#[async_trait]
impl Tool for CaptureSnapshotTool {
    fn name(&self) -> &str {
        "capture_snapshot"
    }

    fn description(&self) -> &str {
        "Chụp ảnh từ camera phía trước xe và trả về chuỗi ảnh dạng base64 JPEG."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn execute(&self, _args: serde_json::Value) -> Result<ToolResult, anyhow::Error> {
        // Gửi request HTTP cục bộ tới Flask server để lấy ảnh snapshot mới nhất
        let client = reqwest::Client::new();
        match client.get("http://127.0.0.1:5000/snapshot").send().await {
            Ok(resp) => {
                let bytes = resp.bytes().await?;
                let b64_img = base64::engine::general_purpose::STANDARD.encode(&bytes);
                Ok(ToolResult {
                    success: true,
                    output: b64_img,
                    error: None,
                })
            }
            Err(e) => {
                Ok(ToolResult {
                    success: false,
                    output: "".to_string(),
                    error: Some(format!("Không thể chụp ảnh: {:?}", e)),
                })
            }
        }
    }
}

// ── Agent Runner Loop ────────────────────────────────────────
pub async fn run_agent_loop(
    api_key: String,
    port_writer: Arc<Mutex<Box<dyn SerialPort>>>,
    latest_distance: Arc<Mutex<f32>>,
    shutdown_flag: Arc<AtomicBool>,
) -> Result<(), anyhow::Error> {
    // 1. Khởi tạo Gemini Model Provider
    let provider = GeminiModelProvider::new("gemini", Some(&api_key));

    // 2. Khởi tạo các tools
    let control_tool = Arc::new(ControlCarTool {
        port_writer,
        latest_distance: Arc::clone(&latest_distance),
    });
    let distance_tool = Arc::new(GetDistanceTool {
        latest_distance,
    });
    let snapshot_tool = Arc::new(CaptureSnapshotTool);

    // 3. Xây dựng AutoClaw Agent
    let agent = Agent::builder()
        .model_provider(Box::new(provider))
        .tools(vec![
            control_tool as Arc<dyn Tool>,
            distance_tool as Arc<dyn Tool>,
            snapshot_tool as Arc<dyn Tool>,
        ])
        .build()?;

    println!("🤖 [AutoClaw Agent] Agent khởi hành thành công! Bắt đầu vòng lặp tuần tra tự trị.");

    // System instruction để hướng dẫn Agent cách tuần tra và điều khiển
    let system_instruction = "Bạn là bộ não tự trị lái xe AutoClaw. Hãy liên tục gọi các công cụ: \
        1. capture_snapshot để chụp ảnh nhìn đường. \
        2. get_distance để đo khoảng cách chướng ngại vật trước mặt. \
        3. Dựa trên phân tích ảnh và khoảng cách, gọi control_car để lái xe rẽ hướng/di chuyển an toàn tránh vật cản.";

    while !shutdown_flag.load(Ordering::Relaxed) {
        // Gửi lệnh để kích hoạt agent turn
        match agent.turn(system_instruction).await {
            Ok(reply) => {
                println!("🤖 [AutoClaw Agent] Phản hồi: {}", reply);
            }
            Err(e) => {
                eprintln!("❌ [AutoClaw Agent] Lỗi trong lượt chạy: {:?}", e);
            }
        }

        // Delay 3 giây giữa mỗi lượt ra quyết định, kiểm tra shutdown flag mỗi 100ms
        for _ in 0..30 {
            if shutdown_flag.load(Ordering::Relaxed) {
                break;
            }
            sleep(Duration::from_millis(100)).await;
        }
    }

    println!("🤖 [AutoClaw Agent] Agent dừng tuần tra tự trị.");
    Ok(())
}
