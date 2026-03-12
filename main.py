from sea_g2p import Normalizer, G2P

normalizer = Normalizer(lang="vi")
g2p = G2P(lang="vi")

test_sentences = [
# =========================
# EXTREME SCIENTIFIC NOTATION
# =========================

"Hằng số Planck là 6.62607015×10^-34 J·s.",
"Khối lượng electron là 9.109×10^-31 kg.",
"Nồng độ ion là 3.5×10^-6 mol/L.",
"Entropy ΔS = k ln W.",
"Giới hạn lim(x→0) sin(x)/x = 1.",
"Phương trình Schrödinger: iħ∂ψ/∂t = Hψ.",
"Biến thiên δE ≈ 1.6×10^-19 J.",
"Vector ⟨x,y,z⟩ có độ dài √(x²+y²+z²)."
# =========================
# INTERNET CHAOS
# =========================

"Website là https://openai.com.",
"API endpoint: https://api.example.com/v1/chat/completions.",
"Email liên hệ: support@example.ai.",
"Repo nằm tại github.com/user/repo.",
"Ping 8.8.8.8 với TTL=64.",
"SSH vào 192.168.1.10:22.",
"MAC address là AA:BB:CC:DD:EE:FF.",
"URL có query: https://site.com?q=sea-g2p&lang=vi."
# =========================
# PROGRAMMING CHAOS
# =========================

"Python list comprehension: [x*x for x in range(10)].",
"Regex pattern là ^[a-zA-Z0-9_]+$.",
"Biểu thức lambda: lambda x: x**2.",
"Câu lệnh SQL: SELECT * FROM users WHERE id=1;",
"JSON ví dụ: {\"name\":\"GPT\",\"year\":2025}.",
"Commit hash: 9f86d081884c7d659a2feaa0.",
"Semantic version: v2.10.3-alpha.1."
# =========================
# FINANCIAL CHAOS
# =========================

"Market cap của Apple là $2.87T.",
"Startup gọi vốn $12.5M.",
"Giá Bitcoin là $63,420.50.",
"Lãi suất tăng thêm +0.75%.",
"Lợi suất trái phiếu đạt 4.25%/năm.",
"Tỷ giá USD/VND là 25,430.",
"EPS quý này đạt $3.45."
# =========================
# UNICODE NIGHTMARE
# =========================

"Tiếng Việt có dấu: Hoà, Hòa, Hòa.",
"Ký tự đặc biệt: zero-width​space.",
"Emoji trong câu: 🤖 GPT-5 rất mạnh.",
"Chuỗi RTL: العربية English tiếng Việt.",
"Math Unicode: ∑_{i=1}^{n} i = n(n+1)/2.",
"Subscript H₂O và superscript x².",
"Chuỗi kết hợp: ấ.",
# =========================
# ABSOLUTE KILLER
# =========================

"Phiên bản model là gpt-4.1-mini-2025-03-11.",
"File backup: backup_2025-03-11_14-32-08.tar.gz.",
"GPU RTX4090 chạy CUDA 12.3.",
"Model đạt 92.45% accuracy trên dataset v1.2.0.",
"CPU Intel Core i9-14900KS chạy ở 6.2GHz.",
"Commit tag release/v2.4.1-hotfix.",
"Container image: ghcr.io/org/model:1.0.0.",
"Log lỗi: ERROR[2025-03-11T14:22:03Z]."
]

for text in test_sentences:
    print("=" * 60)
    print("TEXT:", text)

    normalized = normalizer.normalize(text)
    print("NORMALIZED:", normalized)

    phonemes = g2p.convert(normalized)
    print("PHONEMES:", phonemes)
