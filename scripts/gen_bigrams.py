# -*- coding: utf-8 -*-
"""Sinh bảng bigram skeleton từ các mục 2 âm tiết trong Viet74K
("tin học" -> "tin hoc") để splitter nhận diện từ ghép."""
import sys
sys.stdout.reconfigure(encoding="utf-8")
sys.path.insert(0, r"E:\sea-g2p\scripts")
from clean_dict import skeleton, is_vi_syllable

bigrams = set()
for line in open(r"C:\Users\Admin\AppData\Local\Temp\Viet74K.txt", encoding="utf-8", errors="ignore"):
    toks = line.strip().lower().replace("-", " ").split()
    if len(toks) != 2:
        continue
    a, b = skeleton(toks[0]), skeleton(toks[1])
    if not (a.isascii() and b.isascii() and a.isalpha() and b.isalpha()):
        continue
    if not (is_vi_syllable(a) and is_vi_syllable(b)):
        continue
    bigrams.add(f"{a} {b}")

print(f"{len(bigrams)} bigram")

lines = []
lines.append("// File SINH TỰ ĐỘNG bởi scripts/gen_top_syllables (nguồn: Viet74K).")
lines.append('// Bigram skeleton của từ ghép 2 âm tiết ("tin hoc", "khi tuong") — splitter')
lines.append('// cộng điểm khi cách cắt tạo đúng từ ghép, phân định "tin|hoc" vs "ti|nhoc".')
lines.append("use once_cell::sync::Lazy;")
lines.append("use std::collections::HashSet;")
lines.append("")
lines.append("pub static VI_BIGRAMS: Lazy<HashSet<&'static str>> = Lazy::new(|| {")
lines.append("    [")
row = []
for s in sorted(bigrams):
    row.append(f'"{s}"')
    if len(row) == 6:
        lines.append("        " + ", ".join(row) + ",")
        row = []
if row:
    lines.append("        " + ", ".join(row) + ",")
lines.append("    ].into_iter().collect()")
lines.append("});")
open(r"E:\sea-g2p\src\vi_normalizer\vi_bigrams.rs", "w", encoding="utf-8", newline="\n").write("\n".join(lines) + "\n")
print("đã ghi src/vi_normalizer/vi_bigrams.rs")
