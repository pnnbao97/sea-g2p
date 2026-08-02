# -*- coding: utf-8 -*-
"""Sinh src/vi_normalizer/vi_top_syllables.rs: skeleton của các âm tiết Việt
phổ biến nhất (từ vi_50k) để splitter phân định "tin hoc" vs "ti nhoc"."""
import sys
import unicodedata
sys.stdout.reconfigure(encoding="utf-8")
sys.path.insert(0, r"E:\sea-g2p\scripts")
from clean_dict import skeleton, is_vi_syllable

seen = []
seen_set = set()
n_lines = 0
for line in open(r"C:\Users\Admin\AppData\Local\Temp\vi_50k.txt", encoding="utf-8"):
    parts = line.strip().split()
    if len(parts) != 2:
        continue
    n_lines += 1
    if n_lines > 3000:
        break
    w = parts[0].lower()
    sk = skeleton(w)
    if not sk.isascii() or not sk.isalpha() or len(sk) < 2:
        continue
    if not is_vi_syllable(sk):
        continue
    if sk not in seen_set:
        seen_set.add(sk)
        seen.append(sk)

print(f"{len(seen)} skeleton từ top 3000 dòng")

lines = []
lines.append("// File SINH TỰ ĐỘNG bởi scripts/gen_top_syllables (nguồn: tần suất vi_50k).")
lines.append("// Skeleton (bỏ dấu) của các âm tiết Việt phổ biến nhất — splitter dùng để")
lines.append('// phân định cách cắt: "tin|hoc" (cả hai phổ biến) thắng "ti|nhoc".')
lines.append("use once_cell::sync::Lazy;")
lines.append("use std::collections::HashSet;")
lines.append("")
lines.append("pub static VI_TOP_SYLLABLES: Lazy<HashSet<&'static str>> = Lazy::new(|| {")
lines.append("    [")
row = []
for i, s in enumerate(seen):
    row.append(f'"{s}"')
    if len(row) == 10:
        lines.append("        " + ", ".join(row) + ",")
        row = []
if row:
    lines.append("        " + ", ".join(row) + ",")
lines.append("    ].into_iter().collect()")
lines.append("});")
open(r"E:\sea-g2p\src\vi_normalizer\vi_top_syllables.rs", "w", encoding="utf-8", newline="\n").write("\n".join(lines) + "\n")
print("đã ghi src/vi_normalizer/vi_top_syllables.rs")
