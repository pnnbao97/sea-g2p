from sea_g2p import Normalizer, G2P

normalizer = Normalizer(lang="vi")
g2p = G2P(lang="vi")

test_sentences = [

# =========================
# NUMBER / NORMALIZATION
# =========================

"Giá vàng hôm nay là 88,89 USD mỗi ounce.",
"Nhiệt độ ngoài trời là -3.5°C.",
"Anh ấy chạy 10.000m trong 27:45.",
"Khoảng cách từ Trái Đất đến Mặt Trời là 1.496e8 km.",
"Hằng số Avogadro là 6.022e23 mol^-1.",
"Tỉ số USD/EUR đang tăng.",

# =========================
# SCIENTIFIC / MATH
# =========================

"Tốc độ ánh sáng c ≈ 3×10^8 m/s.",
"Phương trình nổi tiếng là E = mc^2.",
"log10(1000) bằng 3.",
"Giới hạn lim(x→0) sin(x)/x = 1.",
"Diện tích hình tròn là πr².",

# =========================
# CURRENCY
# =========================

"Tôi mua chiếc laptop này với giá $1,299.99.",
"Chiếc vé máy bay giá €350,50.",
"Anh ta kiếm được ¥120000 mỗi tháng.",
"Khoản đầu tư ban đầu là £2,500.",
"Cổ phiếu tăng thêm ₫15.500 hôm nay.",

# =========================
# MIXED LANGUAGE
# =========================

"Model GPT-4 có thể generate code Python.",
"GPU RTX 4090 có 24GB VRAM.",
"Dataset có khoảng 1.2M samples.",
"Framework này chạy trên PyTorch 2.1.",
"API trả về JSON trong 120ms.",

# =========================
# URL / INTERNET
# =========================

"Trang chủ là https://openai.com.",
"Repo nằm ở github.com/user/project.",
"Tài liệu đọc tại docs.python.org.",
"Hãy gửi email đến support@example.com.",
"File tải tại ftp://example.org/data.zip.",

# =========================
# ABBREVIATIONS
# =========================

"Ông ấy là PGS.TS ngành AI.",
"Hội nghị tổ chức tại TP.HCM.",
"Công ty đặt trụ sở ở Q.1.",
"Anh ấy tốt nghiệp ĐH Bách Khoa.",
"Chuyến bay khởi hành lúc 07:30 sáng.",

# =========================
# ROMAN NUMERALS
# =========================

"Louis XIV là vua nước Pháp.",
"Thế chiến thứ II kết thúc năm 1945.",
"Chương IX nói về machine learning.",
"Super Bowl LVIII diễn ra năm 2024.",
"Henry VIII nổi tiếng trong lịch sử Anh.",

# =========================
# TECH UNITS
# =========================

"Ổ cứng có dung lượng 2TB.",
"RAM máy tính là 32GB DDR5.",
"Tốc độ mạng đạt 1Gbps.",
"File log có kích thước 512MB.",
"CPU chạy ở xung nhịp 3.6GHz.",

# =========================
# AMBIGUOUS VIETNAMESE
# =========================

"Má mà mạ mã mả.",
"Bác sĩ nói bệnh nhân bị bạc tóc.",
"Con cá rô rô ở ruộng.",
"Anh ta đang bàn về cái bàn.",
"Nó đang chở chỗ chỗ đó.",

# =========================
# PUNCTUATION CHAOS
# =========================

"Ôi!!! Chuyện gì đang xảy ra???",
"Anh nói: \"Tôi sẽ quay lại lúc 5:00!\"",
"Wait... cái gì cơ?",
"Không... không thể nào!",
"Hả?! Sao lại như vậy?!"

]

for text in test_sentences:
    print("=" * 60)
    print("TEXT:", text)

    normalized = normalizer.normalize(text)
    print("NORMALIZED:", normalized)

    phonemes = g2p.convert(normalized)
    print("PHONEMES:", phonemes)