from sea_g2p import Normalizer, G2P

normalizer = Normalizer(lang="vi")
g2p = G2P(lang="vi")
text = "Truyền thông nhà nước Iran cho biết chỉ huy các lực lượng vũ trang đã tuyên thệ trung thành với ông Mojtaba Khamenei. Vệ binh Cách mạng Hồi giáo (IRGC) sẵn sàng tuyệt đối tuân lệnh và hy sinh để thực hiện các mệnh lệnh thiêng liêng của Người giám hộ luật Hồi giáo đương nhiệm, ngài Mojtaba Khamenei, lực lượng này cho hay."

normalized = normalizer.normalize(text)
print(normalized)
phonemes = g2p.convert(normalized)
print(phonemes)