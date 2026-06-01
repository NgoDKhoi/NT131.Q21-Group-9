import os
import requests
from dotenv import load_dotenv

# Load .env
load_dotenv(dotenv_path="../.env")
api_key = os.getenv("GEMINI_API_KEY")

if not api_key:
    print("❌ Lỗi: Chưa cấu hình GEMINI_API_KEY trong file .env")
    exit(1)

print(f"🔑 Đang kiểm tra API Key: {api_key[:10]}...{api_key[-5:]}")

# Thử truy vấn danh sách model khả dụng qua API v1
url = f"https://generativelanguage.googleapis.com/v1/models?key={api_key}"

try:
    resp = requests.get(url, timeout=10)
    if resp.status_code == 200:
        models = resp.json().get("models", [])
        print("\n✅ Danh sách các mô hình mà API Key của bạn ĐƯỢC PHÉP gọi:")
        for m in models:
            name = m.get("name", "")
            methods = m.get("supportedGenerationMethods", [])
            if "generateContent" in methods:
                print(f" - {name}")
    else:
        print(f"❌ Lỗi truy vấn API v1 (HTTP {resp.status_code}): {resp.text}")
        
    # Thử thêm qua v1beta để đối chiếu
    url_beta = f"https://generativelanguage.googleapis.com/v1beta/models?key={api_key}"
    resp_beta = requests.get(url_beta, timeout=10)
    if resp_beta.status_code == 200:
        print("\n✅ Danh sách qua API v1beta:")
        models_beta = resp_beta.json().get("models", [])
        for m in models_beta:
            name = m.get("name", "")
            methods = m.get("supportedGenerationMethods", [])
            if "generateContent" in methods:
                print(f" - {name}")
    else:
         print(f"❌ Lỗi truy vấn API v1beta (HTTP {resp_beta.status_code}): {resp_beta.text}")
         
except Exception as e:
    print(f"❌ Lỗi kết nối: {e}")
