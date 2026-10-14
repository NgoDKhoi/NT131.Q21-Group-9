// ============================================================
//  src/agent.rs — AutoClaw Autonomous Agent Runner
//
//  Rewritten without zeroclaw dependency.
//  Uses reqwest async + tokio loop + gemini.rs directly.
//  Giữ nguyên toàn bộ logic: ControlCarTool, GetDistanceTool,
//  CaptureSnapshotTool, Safety Reflex, shutdown flag.
// ============================================================

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::time::sleep;
use serialport::SerialPort;
use std::io::Write;

use crate::gemini::GeminiClient;

// ── ControlCarTool ───────────────────────────────────────────
async fn control_car(
    cmd: &str,
    port_writer: &Arc<Mutex<Box<dyn SerialPort>>>,
    latest_distance: &Arc<Mutex<f32>>,
) -> String {
    // Safety Reflex: F + khoảng cách < 15cm → chuyển thành S
    let final_cmd = if cmd == "F" {
        let dist = *latest_distance.lock().unwrap();
        if dist < 15.0 {
            println!(
                "🚨 [Rust Agent Safety Reflex] Phát hiện vật cản {:.1}cm < 15cm -> Tự động chuyển F thành S!",
                dist
            );
            "S"
        } else {
            cmd
        }
    } else {
        cmd
    };

    let mut port = port_writer.lock().unwrap();
    match port.write_all(final_cmd.as_bytes()) {
        Ok(_) => {
            let _ = port.flush();
            format!("Gửi lệnh thành công: {}", final_cmd)
        }
        Err(e) => format!("Lỗi gửi lệnh: {}", e),
    }
}

// ── GetDistanceTool ──────────────────────────────────────────
fn get_distance(latest_distance: &Arc<Mutex<f32>>) -> f32 {
    *latest_distance.lock().unwrap()
}

// ── CaptureSnapshotTool ──────────────────────────────────────
async fn capture_snapshot() -> Result<String, String> {
    let port = std::env::var("FLASK_PORT").unwrap_or_else(|_| "5000".to_string());
    let url = format!("http://127.0.0.1:{}/snapshot", port);
    let client = reqwest::Client::new();
    match client
        .get(&url)
        .send()
        .await
    {
        Ok(resp) => {
            let bytes = resp.bytes().await.map_err(|e| format!("Đọc bytes lỗi: {}", e))?;
            use base64::Engine;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
            Ok(b64)
        }
        Err(e) => Err(format!("Không thể chụp ảnh: {}", e)),
    }
}

// ── Agent Runner Loop ────────────────────────────────────────
pub async fn run_agent_loop(
    api_key: String,
    port_writer: Arc<Mutex<Box<dyn SerialPort>>>,
    latest_distance: Arc<Mutex<f32>>,
    latest_ai_log: Arc<Mutex<String>>,
    shutdown_flag: Arc<AtomicBool>,
) -> Result<(), anyhow::Error> {
    println!("🤖 [AutoClaw Agent] Agent khởi hành thành công! Bắt đầu vòng lặp tuần tra tự trị.");
    *latest_ai_log.lock().unwrap() = "Agent khởi hành thành công! Bắt đầu tuần tra...".to_string();

    while !shutdown_flag.load(Ordering::Relaxed) {
        // 1. Đo khoảng cách từ cache
        let dist = get_distance(&latest_distance);
        println!("🔍 [AutoClaw Agent] Khoảng cách hiện tại: {:.1} cm", dist);
        *latest_ai_log.lock().unwrap() = format!("Đang quét vật cản... Khoảng cách: {:.1} cm", dist);

        // 2. Chụp ảnh từ Flask /snapshot
        let image_b64 = match capture_snapshot().await {
            Ok(b64) => b64,
            Err(e) => {
                let err_msg = format!("Lỗi camera: {}", e);
                eprintln!("❌ [AutoClaw Agent] {}", err_msg);
                *latest_ai_log.lock().unwrap() = err_msg;
                // Không có ảnh → dừng xe an toàn
                control_car("S", &port_writer, &latest_distance).await;
                // Chờ rồi thử lại
                for _ in 0..30 {
                    if shutdown_flag.load(Ordering::Relaxed) { break; }
                    sleep(Duration::from_millis(100)).await;
                }
                continue;
            }
        };

        // 3. Gọi Gemini Vision để phân tích ảnh + khoảng cách
        // gemini.rs dùng reqwest::blocking nên spawn_blocking để không block Tokio
        let api_key_clone = api_key.clone();
        let dist_clone = dist;
        *latest_ai_log.lock().unwrap() = format!("AI đang phân tích ảnh... Khoảng cách: {:.1} cm", dist);

        let decision = tokio::task::spawn_blocking(move || {
            let client = GeminiClient::new(api_key_clone);
            client.analyze_scene(&image_b64, dist_clone)
        })
        .await;

        match decision {
            Ok(Ok(d)) => {
                println!(
                    "🤖 [AutoClaw Agent] Phân tích: \"{}\" → Lệnh đề xuất: {} (Lý do: {})",
                    d.description, d.command, d.reason
                );
                *latest_ai_log.lock().unwrap() = format!("{} (Gợi ý: {}). Lý do: {}", d.description, d.command, d.reason);
                // 4. Thực thi lệnh (qua Safety Reflex)
                let result = control_car(&d.command, &port_writer, &latest_distance).await;
                println!("🤖 [AutoClaw Agent] {}", result);
            }
            Ok(Err(e)) => {
                let err_msg = format!("Lỗi Gemini: {}", e);
                eprintln!("❌ [AutoClaw Agent] {}", err_msg);
                *latest_ai_log.lock().unwrap() = err_msg;
                // Fallback an toàn
                control_car("S", &port_writer, &latest_distance).await;
            }
            Err(e) => {
                let err_msg = format!("Lỗi hệ thống: {}", e);
                eprintln!("❌ [AutoClaw Agent] {}", err_msg);
                *latest_ai_log.lock().unwrap() = err_msg;
                control_car("S", &port_writer, &latest_distance).await;
            }
        }

        // 5. Delay 3 giây giữa mỗi lượt, kiểm tra shutdown mỗi 100ms
        for _ in 0..30 {
            if shutdown_flag.load(Ordering::Relaxed) {
                break;
            }
            sleep(Duration::from_millis(100)).await;
        }
    }

    println!("🤖 [AutoClaw Agent] Agent dừng tuần tra tự trị.");
    *latest_ai_log.lock().unwrap() = "AutoClaw Agent đã dừng.".to_string();
    Ok(())
}