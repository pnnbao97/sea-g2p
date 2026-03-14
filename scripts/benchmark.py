import time
from sea_g2p import Normalizer, G2P
import random

def benchmark():
    normalizer = Normalizer(lang="vi")
    g2p = G2P(lang="vi")
    
    test_sentences = [
        "Giá SP500 hôm nay là 4.200,5 điểm",
        "Ngày 15/03/2024, thời tiết Hà Nội rất đẹp.",
        "Tôi đang học tại trường Đại học Bách Khoa TP.HCM.",
        "Số điện thoại của tôi là 0912-345-678.",
        "Nhiệt độ ngoài trời là 38.5°C.",
        "Công thức H2O là của nước.",
        "Chào bạn, bạn khỏe không?",
        "Mời bạn uống cốc café 20k.",
        "Tỉ giá USD/VND hiện nay là 24.500.",
        "Ông ấy sinh năm 1990.",
    ] * 1000  # 10,000 sentences
    
    print(f"Benchmarking with {len(test_sentences)} sentences...")
    
    # Benchmark Normalization
    start_time = time.time()
    normalized_texts = [normalizer.normalize(t) for t in test_sentences]
    norm_time = time.time() - start_time
    print(f"Normalization time: {norm_time:.4f}s ({len(test_sentences)/norm_time:.2f} sentences/sec)")
    
    # Benchmark G2P Batch
    start_time = time.time()
    phonemes = g2p.phonemize_batch(normalized_texts)
    g2p_time = time.time() - start_time
    print(f"G2P Batch time: {g2p_time:.4f}s ({len(test_sentences)/g2p_time:.2f} sentences/sec)")
    
    # Total
    print(f"Total time: {norm_time + g2p_time:.4f}s")

if __name__ == "__main__":
    benchmark()
