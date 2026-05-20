// ============================================================
//  ZeroClaw Core — Rust Safety Engine (PyO3)
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

// ── Ngưỡng an toàn ─────────────────────────────────────────
const SAFETY_DISTANCE_CM: f32 = 15.0;

// ── Shared state giữa reader thread và PyO3 methods ────────
// Dùng Arc<Mutex<T>> thay vì channel để Flask có thể poll
// get_distance() bất kỳ lúc nào mà không block
#[pyclass]
pub struct ZeroClaw {
    // Writer: Flask thread gọi send_command() → ghi Serial
    port_writer: Arc<Mutex<Box<dyn SerialPort>>>,

    // Cache distance: reader thread ghi, send_command() đọc
    // Khởi tạo 999.0 = "chưa có dữ liệu / an toàn tối đa"
    latest_distance: Arc<Mutex<f32>>,

    // Cache mode: "MANUAL" | "AUTO"
    latest_mode: Arc<Mutex<String>>,
}

#[pymethods]
impl ZeroClaw {
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
                        eprintln!("[ZeroClaw] Serial EOF, reader thread thoát.");
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
                            // Chỉ chấp nhận khoảng cách hợp lệ (2–400 cm)
                            // Lọc bỏ các giá trị rác hoặc out-of-range
                            if (2.0..=400.0).contains(&dist) {
                                *dist_ref.lock().unwrap() = dist;
                            }
                        }
                        // Bỏ qua: "READY", "ZeroClaw READY", bytes rác
                    }
                    Err(e) => {
                        // Timeout (100ms) → bình thường, tiếp tục vòng lặp
                        // Lỗi thật → log và dừng lại
                        if e.kind() != std::io::ErrorKind::TimedOut {
                            eprintln!("[ZeroClaw] Serial read error: {}", e);
                            thread::sleep(Duration::from_millis(50));
                        }
                    }
                }
            }
        });

        Ok(ZeroClaw {
            port_writer: Arc::new(Mutex::new(writer)),
            latest_distance,
            latest_mode,
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
}

/// PyO3 module entry point
#[pymodule]
fn zeroclaw_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<ZeroClaw>()?;
    Ok(())
}