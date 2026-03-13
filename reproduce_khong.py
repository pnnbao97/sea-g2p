import sea_g2p_rs
normalizer = sea_g2p_rs.Normalizer()
text = "128GB DDR5"
print(f"Input: {text}")
print(f"Output: '{normalizer.normalize(text)}'")
