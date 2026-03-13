import sea_g2p_rs
normalizer = sea_g2p_rs.Normalizer()
text = "RAM hệ thống là 128GB DDR5-6400."
print(f"Input: {text}")
print(f"Output: {normalizer.normalize(text)}")
text2 = "1,299"
print(f"Input: {text2}")
print(f"Output: {normalizer.normalize(text2)}")
