from sea_g2p_rs import Normalizer as RustNormalizer

rn = RustNormalizer()
text = "RAM hệ thống là 128GB DDR5-6400."
print(f"Input: {text}")
print(f"Output: {rn.normalize(text)}")
