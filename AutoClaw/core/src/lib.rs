// ============================================================
//  AutoClaw Core — Rust Safety Engine (PyO3)
//  
//  Kiến trúc giải quyết 2 vấn đề cũ:
//  1. Serial owned hoàn toàn bởi Rust — Python không mở port
//  2. Background reader thread đọc liên tục, cache vào
//     Arc<Mutex<f32>> — send_command() đọc cache, không
//     đọc thẳng port → tránh đọc sai / deadlock
// ============================================================

use pyo3::prelude::*;
use serialport::SerialPort;
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

mod gemini;
mod agent;

// ── Ngưỡng an toàn ─────────────────────────────────────────
const SAFETY_DISTANCE_CM: f32 = 15.0;

// ── Shared state giữa reader thread và PyO3 methods ────────
// Dùng Arc<Mutex<T>> thay vì channel để Flask có thể poll
// get_distance() bất kỳ lúc nào mà không block
#[pyclass]
pub struct AutoClaw {
    // Writer: Flask thread gọi send_command() → ghi Serial
    port_writer: Arc<Mutex<Box<dyn SerialPort>>>,

    // Cache distance: reader thread ghi, send_command() đọc
    // Khởi tạo 999.0 = "chưa có dữ liệu / an toàn tối đa"
    latest_distance: Arc<Mutex<f32>>,

    // Cache mode: "MANUAL" | "AUTO"
    latest_mode: Arc<Mutex<String>>,

    // Cache AI log: agent ghi, Python đọc
    latest_ai_log: Arc<Mutex<String>>,

    // Cờ dừng và join handle của AutoClaw Rust Agent
    agent_shutdown: Arc<std::sync::atomic::AtomicBool>,
    agent_thread: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
}

#[pymethods]
impl AutoClaw {
    /// Khởi tạo: mở Serial, spawn reader thread
    #[new]
    fn new(port_name: &str, baud_rate: u32) -> PyResult<Self> {
        // Mở port chính để ghi lệnh
        let writer = serialport::new(port_name, baud_rate)
            .timeout(Duration::from_millis(100))
            .open()
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    format!("Không thể mở Serial '{}': {}", port_name, e),
                )
            })?;

        // Clone port cho reader thread
        // try_clone() tạo file descriptor riêng biệt trên Linux
        let reader_port = writer.try_clone().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Không thể clone Serial port: {}", e),
            )
        })?;

        // Shared state
        let latest_distance = Arc::new(Mutex::new(999.0f32));
        let latest_mode     = Arc::new(Mutex::new("MANUAL".to_string()));
        let latest_ai_log   = Arc::new(Mutex::new("Đang chờ lệnh (✌ hoặc khẩu lệnh \"AI phân tích\")...".to_string()));

        // Clone Arc để move vào reader thread
        let dist_ref = Arc::clone(&latest_distance);
        let mode_ref = Arc::clone(&latest_mode);

        // ── Background Reader Thread ──────────────────────────
        // Thread này không bao giờ gọi Python → không cần GIL
        // Đọc từng dòng Arduino gửi lên, parse và cập nhật cache
        thread::spawn(move || {
            let mut reader = BufReader::new(reader_port);
            let mut line   = String::new();

            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) => {
                        // EOF — Arduino ngắt kết nối
                        eprintln!("[AutoClaw] Serial EOF, reader thread thoát.");
                        break;
                    }
                    Ok(_) => {
                        let trimmed = line.trim();

                        // Parse mode strings từ Arduino
                        if trimmed == "MODE:AUTO" {
                            *mode_ref.lock().unwrap() = "AUTO".to_string();

                        } else if trimmed == "MODE:MANUAL" {
                            *mode_ref.lock().unwrap() = "MANUAL".to_string();

                        } else if let Ok(dist) = trimmed.parse::<f32>() {
                            // Chỉ chấp nhận khoảng cách hợp lệ (2–400 cm) hoặc giá trị đặc biệt 999.0 (thể hiện không có vật cản)
                            if (2.0..=400.0).contains(&dist) || dist == 999.0 {
                                *dist_ref.lock().unwrap() = dist;
                            }
                        }
                        // Bỏ qua: "READY", "AutoClaw READY", bytes rác
                    }
                    Err(e) => {
                        // Timeout (100ms) → bình thường, tiếp tục vòng lặp
                        // Lỗi thật → log và dừng lại
                        if e.kind() != std::io::ErrorKind::TimedOut {
                            eprintln!("[AutoClaw] Serial read error: {}", e);
                            thread::sleep(Duration::from_millis(50));
                        }
                    }
                }
            }
        });

        let agent_shutdown = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let agent_thread   = Arc::new(Mutex::new(None));

        Ok(AutoClaw {
            port_writer: Arc::new(Mutex::new(writer)),
            latest_distance,
            latest_mode,
            latest_ai_log,
            agent_shutdown,
            agent_thread,
        })
    }

    /// Gửi lệnh xuống Arduino, có Safety Reflex tích hợp.
    ///
    /// Safety Reflex: Nếu lệnh là "F" (Tiến) VÀ khoảng cách
    /// cache < 15cm → tự động đổi thành "S" (Dừng).
    /// Đọc từ CACHE, không đọc port trực tiếp → nhanh, an toàn.
    fn send_command(&self, cmd: &str) -> PyResult<()> {
        // Xác định lệnh thực sự sẽ gửi
        let final_cmd: String = if cmd == "F" {
            let dist = *self.latest_distance.lock().unwrap();
            if dist < SAFETY_DISTANCE_CM {
                println!(
                    "🚨 [Safety Reflex] Vật cản {:.1}cm < {}cm → Auto-STOP!",
                    dist, SAFETY_DISTANCE_CM
                );
                "S".to_string()
            } else {
                cmd.to_string()
            }
        } else {
            cmd.to_string()
        };

        // Ghi Serial — lock writer ngắn nhất có thể
        let mut port = self.port_writer.lock().unwrap();
        port.write_all(final_cmd.as_bytes()).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Ghi Serial lỗi: {}", e),
            )
        })?;

        Ok(())
    }

    /// Trả về khoảng cách mới nhất (cm) từ cache.
    /// Non-blocking — luôn trả về ngay lập tức.
    fn get_distance(&self) -> PyResult<f32> {
        Ok(*self.latest_distance.lock().unwrap())
    }

    /// Trả về mode hiện tại: "AUTO" hoặc "MANUAL"
    fn get_mode(&self) -> PyResult<String> {
        Ok(self.latest_mode.lock().unwrap().clone())
    }

    /// Shortcut để Flask check auto mode nhanh
    fn is_auto_mode(&self) -> PyResult<bool> {
        Ok(self.latest_mode.lock().unwrap().as_str() == "AUTO")
    }

    /// Gửi ảnh camera (base64) lên Gemini AI để phân tích vật cản và lấy gợi ý lái
    fn analyze_scene(&self, image_base64: &str, api_key: &str) -> PyResult<String> {
        let client = gemini::GeminiClient::new(api_key);
        let dist = *self.latest_distance.lock().unwrap();
        match client.analyze_scene(image_base64, dist) {
            Ok(decision) => {
                serde_json::to_string(&decision).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                        format!("Lỗi serialize JSON: {}", e),
                    )
                })
            }
            Err(e) => {
                Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    format!("Lỗi gọi Gemini: {}", e),
                ))
            }
        }
    }

    /// Trả về log phân tích mới nhất của AI agent từ cache.
    fn get_ai_log(&self) -> PyResult<String> {
        Ok(self.latest_ai_log.lock().unwrap().clone())
    }

    /// Bắt đầu Agent tuần tra tự trị bằng Rust sử dụng framework AutoClaw
    fn start_agent(&self, api_key: String) -> PyResult<()> {
        self.stop_agent()?;

        self.agent_shutdown.store(false, std::sync::atomic::Ordering::Relaxed);
        
        let shutdown_flag = Arc::clone(&self.agent_shutdown);
        let port_writer = Arc::clone(&self.port_writer);
        let latest_distance = Arc::clone(&self.latest_distance);
        let latest_ai_log = Arc::clone(&self.latest_ai_log);

        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Err(e) = agent::run_agent_loop(api_key, port_writer, latest_distance, latest_ai_log, shutdown_flag).await {
                    eprintln!("🚨 [AutoClaw Agent] Lỗi trong vòng lặp Agent: {:?}", e);
                }
            });
        });

        *self.agent_thread.lock().unwrap() = Some(handle);
        println!("🤖 [AutoClaw Core] Đã khởi chạy Agent chạy ngầm trong Rust.");
        Ok(())
    }

    /// Dừng Agent tuần tra tự trị
    fn stop_agent(&self) -> PyResult<()> {
        self.agent_shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(handle) = self.agent_thread.lock().unwrap().take() {
            let _ = handle.join();
        }
        self.agent_shutdown.store(false, std::sync::atomic::Ordering::Relaxed);
        println!("🤖 [AutoClaw Core] Đã dừng Agent chạy ngầm trong Rust.");
        Ok(())
    }
}

/// PyO3 module entry point
#[pymodule]
fn autoclaw_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<AutoClaw>()?;
    Ok(())
}