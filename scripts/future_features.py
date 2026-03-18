import time
from sea_g2p import Normalizer, G2P

# ==============================================================================
# FULL HARDCORE STRESS TEST SUITE FOR SEA-G2P
# ==============================================================================
ALL_TEST_SAMPLES = {
    "TOAN_KHOA_HOC_PHUC_TAP": [
        "Xét hàm số f(x) = ∇ · E với sai số ε ≤ 10^-9 tại T=25°C.",
        "Phản ứng Cu + 2H2SO4 (đặc) → CuSO4 + SO2 + 2H2O đạt pH = 7.4 ± 0.02.",
        "Vận tốc ánh sáng c ≈ 3×10^8 m/s, khối lượng m = 1.67×10^-27 kg.",
        "Tích phân từ 0 đến π của sin(x)dx xấp xỉ bằng 2.0000001."
    ],
    "TAI_CHINH_LUAT_QUOC_TE": [
        "Lợi nhuận ròng tăng $1,234.56 lên +15.7% YoY (Year-over-Year).",
        "Khoản vay 500.000.000 VNĐ có lãi suất 7,5%/năm tính theo mã ISO-4217.",
        "Nguyên tắc Pacta sunt servanda trong Công ước Vienna 1969 có hiệu lực @2026.",
        "Thuế VAT 10% áp dụng cho đơn hàng nội địa từ ngày 01/01/2026."
    ],
    "CONG_NGHE_KY_THUAT_HE_THONG": [
        "Triển khai Microservices trên Kubernetes (K8s) qua gRPC framework v1.5.0.",
        "Regex `^[a-zA-Z0-9+_.-]+@[a-zA-Z0-9.-]+$` dùng để validate email.",
        "Băng thông 5G đỉnh ~20 Gbps với độ trễ <1ms trên dải tần 3.5GHz.",
        "Sử dụng lệnh `sudo chmod -R 755 /var/www/html/index.php` để phân quyền."
    ],
    "XU_LY_DU_LIEU_LOGS": [
        "Model đạt accuracy 99.87% sau 3×10^5 steps với lr=1e-4.",
        "Server chạy tại 127.0.0.1:8080, RAM 32GB, CPU 3.2GHz.",
        "Dataset có 1,234,567 samples (~1.2M), train/val/test = 80/10/10.",
        "File lưu tại /home/user/data_v3_final.csv lúc 23:59:59."
    ],
    "MEDICAL_BIO": [
        "Bệnh nhân được chỉ định dùng Paracetamol 500mg, 02 viên/ngày (uống lúc 08:00 và 20:00).",
        "Kết quả xét nghiệm máu: chỉ số HbA1c = 6.5% (ngưỡng tiền tiểu đường).",
        "Công thức cấu tạo của Glucose là C6H12O6 với khối lượng mol M = 180.16 g/mol.",
        "Tiêm tĩnh mạng IV (Intravenous) chậm 10ml dung dịch NaCl 0.9%."
    ],
    "PHYSICS_ELECTRICAL": [
        "Trở kháng của mạch Z = R + jX = 50 + j10 Ω tại tần số f = 50Hz.",
        "Hệ số công suất cosφ = 0.85 (lagging) với dòng điện I = 10∠30° A.",
        "Hằng số Planck h ≈ 6.626×10^-34 J·s, hằng số Boltzmann k_B = 1.38×10^-23 J/K.",
        "Bước sóng λ = 550nm tương ứng với tần số ν ≈ 5.45×10^14 Hz."
    ],
    "NAMED_ENTITY_LITERAL": [
        "Giáo sư Stephen Hawking là tác giả cuốn 'A Brief History of Time'.",
        "Chuyến bay VN-123 của Vietnam Airlines hạ cánh lúc 14:15 tại sân bay Nội Bài (HAN).",
        "Biển số xe 51F-123.45 vừa đi qua trạm thu phí lúc 09:30:15.",
        "Mã số thuế của doanh nghiệp là 0123456789-001."
    ],
    "FINAL_BOSS_EDGE_CASES": [
        "Ngày 10/11/12, anh ấy đã mua 10/12 cân táo.",
        "Mã số thuế cá nhân: 8123456789-001 (Vui lòng đọc từng số).",
        "Tọa độ GPS: 10°46'37\"N 106°41'43\"E (TP. Hồ Chí Minh).",
        "Lệnh CLI: `uv run python -m pytest --cov=sea_g2p tests/`.",
        "Toán tử Laplace: Δf = ∇²f = Σ (∂²f/∂xᵢ²).",
        "Slogan: 'VieNeu - AI cho người Việt' (Đọc đúng tên brand)."
    ]
}

def run_stress_test():
    # Khởi tạo engine
    norm_engine = Normalizer()
    g2p_engine = G2P()
    
    print(f"{'#'*30} SEA-G2P ULTIMATE STRESS TEST {'#'*30}")
    start_total = time.time()
    count = 0

    for category, samples in ALL_TEST_SAMPLES.items():
        print(f"\n{'='*25} {category} {'='*25}\n")
        for text in samples:
            count += 1
            # Đo thời gian xử lý từng câu nếu cần
            norm_out = norm_engine.normalize(text)
            g2p_out = g2p_engine.convert(norm_out)
            
            print(f"INPUT: {text}")
            print(f"NORM : {norm_out}")
            print(f"G2P  : {g2p_out}\n")

    end_total = time.time()
    print(f"{'#'*30} TEST SUMMARY {'#'*30}")
    print(f"Tổng số câu test: {count}")
    print(f"Thời gian thực thi: {end_total - start_total:.4f} giây")
    print(f"Tốc độ trung bình: {(end_total - start_total)/count:.4f} s/câu")

if __name__ == "__main__":
    run_stress_test()