from sea_g2p import Normalizer, G2P

normalizer = Normalizer(lang="vi")
g2p = G2P(lang="vi")

# Automatic parallel processing when list is passed
texts = ["Giá cổ phiếu tăng từ $0.000045 lên $1,234.5678 trong 3.5×10^6 giao dịch.", "Hãy gửi email đến support@example.com."]
normalized = normalizer.normalize(texts)
print(normalized)
phonemes = g2p.convert(normalized)
print(phonemes)