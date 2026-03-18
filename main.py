TEST_SUITE = {
    "ULTRA_MATH": [
        r"Tính ∫_{-∞}^{∞} \frac{1}{\sqrt{2\pi\sigma^2}} e^{-\frac{(x-\mu)^2}{2\sigma^2}} dx.",
        r"Giải phương trình: log₁₀(x^2 + 1) - ln(e^x) = 0.",
        r"Cho dãy aₙ = ∑_{k=1}^{n} \frac{k^2}{2^k}, tìm lim_{n→∞} aₙ.",
        r"Ma trận A ∈ ℝ^{n×n}, với det(A) = 0 ⇒ A không khả nghịch."
    ],

    "FINANCE_MIXED": [
        "Doanh thu đạt $1.23B (+12.5% YoY), EBITDA margin = 35.75%.",
        "Giá BTC tăng từ $29,500.50 lên $61,234.99 trong 6 tháng (~+107.6%).",
        "Lãi suất 7.25%/năm, tính lãi kép sau 3 năm với P = 100,000,000 VND.",
        "Tỷ giá USD/VND = 24,567.89; EUR/USD = 1.085."
    ],

    "PHYSICS_ENGINEERING": [
        "Gia tốc a = 9.81 m/s², quãng đường s = 1/2 a t^2 với t = 5s.",
        "Công suất 3.5kW hoạt động trong 2.25h tiêu thụ bao nhiêu kWh?",
        "Tần số 2.4GHz, bước sóng λ = c/f.",
        "Áp suất 1.2bar trong bình thể tích 3m³."
    ],

    "CHEMISTRY": [
        "Phản ứng: C6H12O6 + 6O2 → 6CO2 + 6H2O.",
        "Dung dịch H2SO4 0.5M có pH ≈ -log₁₀[H+].",
        "NaCl → Na⁺ + Cl⁻.",
        "CH3COOH + NaOH → CH3COONa + H2O."
    ],

    "COMPUTER_SCIENCE": [
        "Call API https://api.example.com/v2/users?id=42&sort=desc.",
        "Regex: ^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\\.com$",
        "Chạy python train.py --lr=1e-4 --batch_size=64 --epochs=10.",
        "JSON: {\"user_id\": 123, \"score\": 99.87, \"active\": true}"
    ],

    "MIXED_HARDCORE": [
        "Model đạt accuracy 99.87% sau 3×10^5 steps với lr=1e-4.",
        "Server chạy tại 127.0.0.1:8080, RAM 32GB, CPU 3.2GHz.",
        "Dataset có 1,234,567 samples (~1.2M), train/val/test = 80/10/10.",
        "File lưu tại /home/user/data_v3_final.csv lúc 23:59:59."
    ]
}

def run_tests(normalize, g2p):
    for category, samples in TEST_SUITE.items():
        print(f"\n{'='*20} {category} {'='*20}\n")

        for text in samples:
            norm = normalize(text)
            phon = g2p(norm)

            print(f"INPUT: {text}")
            print(f"NORM : {norm}")
            print(f"G2P  : {phon}\n")

from sea_g2p import Normalizer, G2P

run_tests(Normalizer().normalize, G2P().convert)