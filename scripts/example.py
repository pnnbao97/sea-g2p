from sea_g2p import Normalizer, G2P

normalizer = Normalizer(lang="vi")
g2p = G2P(lang="vi")

text = "Giá SP500 hôm nay là 4.200,5 điểm."
normalized = normalizer.normalize(text)
print(normalized)
phonemes = g2p.convert(normalized)
print(phonemes)