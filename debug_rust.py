import sea_g2p.sea_g2p_rs as sea_g2p_rs
n = sea_g2p_rs.Normalizer()
text = "RAM hệ thống là 128GB DDR5-6400."
print(f"Input: {text}")
print(f"Output: {n.normalize(text)}")

text2 = "1,299"
print(f"Input: {text2}")
print(f"Output: {n.normalize(text2)}")

text3 = "Độ pH của nước là 7."
print(f"Input: {text3}")
print(f"Output: {n.normalize(text3)}")

text4 = "1.5×10^-3"
print(f"Input: {text4}")
print(f"Output: {n.normalize(text4)}")
