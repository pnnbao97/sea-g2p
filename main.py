from sea_g2p import Normalizer, G2P

normalizer = Normalizer(lang="vi")
g2p = G2P(lang="vi")

test_sentences = [

# =========================
# EXTREME NUMBER NORMALIZATION
# =========================

"Giá cổ phiếu tăng từ $0.000045 lên $1,234.5678 trong 3.5×10^6 giao dịch.",
"Dân số thế giới khoảng 7,888,000,000 người (~7.9B).",
"Tốc độ xử lý đạt 1.23e-9 giây mỗi phép tính.",
"Nhiệt độ lõi mặt trời khoảng 1.57×10^7 K.",
"Khoảng cách thiên hà Andromeda là 2.537e6 năm ánh sáng.",

# =========================
# ADVANCED SCIENTIFIC EXPRESSIONS
# =========================

"Phương trình Einstein: E = mc².",
"Tốc độ ánh sáng c ≈ 2.99792458×10^8 m/s.",
"Giới hạn lim(x→∞) (1 + 1/x)^x = e.",
"Đạo hàm của sin(x) là cos(x).",
"Tích phân ∫₀^∞ e^-x dx = 1.",

# =========================
# PROGRAMMING / AI / TECH
# =========================

"Model GPT-4.1-turbo đạt 128k context tokens.",
"Framework chạy trên PyTorch 2.2 + CUDA 12.1.",
"GPU NVIDIA RTX 4090 có 24GB GDDR6X VRAM.",
"Dataset gồm 3.2M samples (~1.8TB audio).",
"Latency trung bình chỉ ~42ms / request qua REST API.",

# =========================
# INTERNET / NETWORK CHAOS
# =========================

"Server chạy tại https://api.example.ai/v1/query.",
"Repo nằm ở https://github.com/user/project-v2.",
"Tài liệu tại https://docs.example.ai/v3.1/guide.",
"Ping đến 8.8.8.8 mất khoảng 23ms.",
"IPv6 address là 2001:0db8:85a3:0000:0000:8a2e:0370:7334.",

# =========================
# EMAIL / SOCIAL
# =========================

"Liên hệ qua email research.ai+test@example-domain.org.",
"Gửi báo cáo đến admin_v2@server.ai.",
"Username của tôi là user_2024_dev.",
"File backup nằm ở /home/user/data_v3.2.tar.gz.",
"Log lỗi ghi tại error_log_2024-10-21.txt.",

# =========================
# FINANCE / CRYPTO
# =========================

"Giá Bitcoin đạt $68,450.25 vào lúc 14:32 UTC.",
"Ethereum gas fee khoảng 32 gwei.",
"Market cap của công ty là $1.25T.",
"Lãi suất tăng từ 3.75% lên 5.25%.",
"Tổng giá trị giao dịch đạt ₫12,500,000,000.",

# =========================
# ROMAN NUMERALS / HISTORY
# =========================

"Louis XIV là vua nước Pháp.",
"Henry VIII có sáu đời vợ.",
"Thế chiến thứ II kết thúc năm 1945.",
"Super Bowl LVIII diễn ra năm 2024.",
"Chương IX nói về deep learning.",

# =========================
# EXTREME TECH SPECS
# =========================

"CPU chạy ở 5.7GHz với 32 cores.",
"RAM hệ thống là 128GB DDR5-6400.",
"SSD đạt tốc độ 7,450MB/s.",
"Network bandwidth đạt 10Gbps.",
"Cluster gồm 256 nodes × 8 GPUs.",

# =========================
# AMBIGUOUS VIETNAMESE
# =========================

"Anh ta đang bàn về cái bàn.",
"Con cá rô rô ở ruộng.",
"Má mà mạ mã mả.",
"Nó chở chỗ chỗ đó.",
"Ông ấy bảo bảo anh ấy bảo tôi.",

# =========================
# PUNCTUATION NIGHTMARE
# =========================

"Ôi!!! Chuyện gì đang xảy ra???!!!",
"Hả?! Sao lại như vậy?!",
"Không... không thể nào!!!",
"Wait... cái gì cơ?!",
"Anh nói: \"Tôi sẽ quay lại lúc 05:00!!!\""

]

for text in test_sentences:
    print("=" * 60)
    print("TEXT:", text)

    normalized = normalizer.normalize(text)
    print("NORMALIZED:", normalized)

    phonemes = g2p.convert(normalized)
    print("PHONEMES:", phonemes)