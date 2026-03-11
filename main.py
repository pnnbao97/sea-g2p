from sea_g2p import Normalizer, G2P

normalizer = Normalizer(lang="vi")
g2p = G2P(lang="vi")

test_sentences = [
    "Giá SP500 hôm nay là 4.200,5 điểm."
]

for text in test_sentences:
    print("=" * 60)
    print("TEXT:", text)

    normalized = normalizer.normalize(text)
    print("NORMALIZED:", normalized)

    phonemes = g2p.convert(normalized)
    print("PHONEMES:", phonemes)