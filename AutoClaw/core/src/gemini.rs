// ============================================================
//  src/gemini.rs — Gemini 2.0 Flash Vision Client
//
//  Gửi ảnh camera (base64) + prompt → nhận 1 ký tự lệnh
//  Dùng reqwest::blocking vì agent chạy trong thread riêng.
// ============================================================

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const GEMINI_MODEL: &str = "gemini-2.0-flash";
const GEMINI_URL: &str =
    "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent";

const SYSTEM_PROMPT: &str = "\
Bạn là hệ thống lái tự động của xe robot ZeroClaw. \
Nhìn vào bức ảnh phía trước và đưa ra quyết định:

- Đường thoáng, không có vật cản gần: trả về 'F' (Tiến)
- Có vật cản to / tường ngay trước mặt: trả về 'L' hoặc 'R' (Rẽ)
- Quá sát vật cản / bị kẹt: trả về 'B' (Lùi)
- Không chắc chắn / nguy hiểm: trả về 'S' (Dừng)

CHỈ trả về ĐÚNG MỘT KÝ TỰ IN HOA (F, B, L, R, S). Không giải thích.";

// ── Gemini request structs ────────────────────────────────────

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Part {
    Text { text: String },
    Image { inline_data: InlineData },
}

#[derive(Serialize)]
struct InlineData {
    mime_type: String,
    data: String, // base64
}

// ── Gemini response structs ───────────────────────────────────

#[derive(Deserialize, Debug)]
pub struct GeminiResponse {
    pub candidates: Option<Vec<Candidate>>,
    pub error: Option<GeminiError>,
}

#[derive(Deserialize, Debug)]
pub struct Candidate {
    pub content: ResponseContent,
}

#[derive(Deserialize, Debug)]
pub struct ResponseContent {
    pub parts: Vec<ResponsePart>,
}

#[derive(Deserialize, Debug)]
pub struct ResponsePart {
    pub text: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GeminiError {
    pub message: String,
    pub code: Option<u32>,
}

// ── GeminiClient ─────────────────────────────────────────────

pub struct GeminiClient {
    client: Client,
    api_key: String,
}

impl GeminiClient {
    /// Khởi tạo client với timeout 10s
    pub fn new(api_key: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build reqwest client");

        GeminiClient {
            client,
            api_key: api_key.into(),
        }
    }

    /// Gửi ảnh (base64 JPEG) + khoảng cách → nhận 1 ký tự lệnh
    ///
    /// Returns: Ok("F") | Ok("B") | Ok("L") | Ok("R") | Ok("S")
    ///          Err(String) nếu API lỗi / timeout
    pub fn ask_for_command(
        &self,
        image_base64: &str,
        distance_cm: f32,
    ) -> Result<String, String> {
        let prompt = format!(
            "{}\n\nThông tin bổ sung: Khoảng cách cảm biến phía trước = {:.1} cm.",
            SYSTEM_PROMPT, distance_cm
        );

        let body = GeminiRequest {
            contents: vec![Content {
                parts: vec![
                    Part::Text {
                        text: prompt,
                    },
                    Part::Image {
                        inline_data: InlineData {
                            mime_type: "image/jpeg".to_string(),
                            data: image_base64.to_string(),
                        },
                    },
                ],
            }],
        };

        let url = format!("{}?key={}", GEMINI_URL, self.api_key);

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        let gemini_resp: GeminiResponse = resp
            .json()
            .map_err(|e| format!("JSON parse failed: {}", e))?;

        // ── API-level error ──────────────────────────────────
        if let Some(err) = gemini_resp.error {
            return Err(format!(
                "Gemini API error {}: {}",
                err.code.unwrap_or(0),
                err.message
            ));
        }

        // ── Parse response text ──────────────────────────────
        let raw = gemini_resp
            .candidates
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.content.parts.into_iter().next())
            .and_then(|p| p.text)
            .unwrap_or_default();

        let cmd = raw.trim().to_uppercase();

        // Validate — chỉ chấp nhận F B L R S
        let valid = ["F", "B", "L", "R", "S"];
        if let Some(&v) = valid.iter().find(|&&x| cmd.starts_with(x)) {
            Ok(v.to_string())
        } else {
            Err(format!("Gemini returned unexpected: '{}'", raw.trim()))
        }
    }
}