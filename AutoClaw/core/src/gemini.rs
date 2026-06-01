// ============================================================
//  src/gemini.rs — Gemini 2.0 Flash Vision Client
//
//  Gửi ảnh camera (base64) + prompt → nhận JSON mô tả vật cản
//  bằng tiếng Việt và gợi ý hướng rẽ.
//  Dùng reqwest::blocking vì agent chạy trong thread riêng.
// ============================================================

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const GEMINI_URL: &str =
    "https://generativelanguage.googleapis.com/v1beta/models/gemini-flash-latest:generateContent";

const SYSTEM_PROMPT: &str = "\
Bạn là trợ lý AI phân tích hình ảnh của xe robot AutoClaw. \
Nhìn vào bức ảnh phía trước và đưa ra phân tích theo định dạng JSON sau:
{
  \"description\": \"Mô tả ngắn gọn vật thể/vật cản chính chắn phía trước bằng tiếng Việt (dưới 15 từ)\",
  \"command\": \"Lệnh di chuyển đề xuất: 'F' (Tiến nếu thoáng), 'B' (Lùi nếu bị chặn sát), 'L' (Rẽ trái), 'R' (Rẽ phải), 'S' (Dừng lại)\",
  \"reason\": \"Lý do ngắn gọn bằng tiếng Việt tại sao bạn đề xuất lệnh lái này (dưới 15 từ)\"
}
Lưu ý: Chỉ trả về chuỗi JSON hợp lệ, không thêm bất kỳ văn bản giải thích nào khác ngoài JSON. Không bọc trong dấu nháy ```json.";

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

// ── Output Decision struct ────────────────────────────────────

#[derive(Serialize, Deserialize, Debug)]
pub struct GeminiDecision {
    pub description: String,
    pub command: String,
    pub reason: String,
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

    /// Gửi ảnh (base64 JPEG) + khoảng cách siêu âm → nhận mô tả & gợi ý (JSON)
    pub fn analyze_scene(
        &self,
        image_base64: &str,
        distance_cm: f32,
    ) -> Result<GeminiDecision, String> {
        let prompt = format!(
            "{}\n\nThông tin bổ sung: Khoảng cách cảm biến phía trước đo được = {:.1} cm.",
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

        let clean_json = raw.trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let decision: GeminiDecision = serde_json::from_str(clean_json)
            .map_err(|e| format!("Serde parse error: {} | Raw output: '{}'", e, raw))?;

        // Validate command
        let valid = ["F", "B", "L", "R", "S"];
        let cmd_upper = decision.command.trim().to_uppercase();
        if valid.contains(&cmd_upper.as_str()) {
            Ok(GeminiDecision {
                description: decision.description,
                command: cmd_upper,
                reason: decision.reason,
            })
        } else {
            Ok(GeminiDecision {
                description: decision.description,
                command: "S".to_string(), // Fallback to Stop if AI commands weirdly
                reason: decision.reason,
            })
        }
    }
}