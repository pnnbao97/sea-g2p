# -*- coding: utf-8 -*-
"""Sinh src/g2p/en_top_words.rs: top từ tiếng Anh phổ biến (google-10000)
để segment_oov ưu tiên biên cắt tạo TỪ THẬT ("fine|tune" thắng "fin|etune")."""
import sys
sys.stdout.reconfigure(encoding="utf-8")

# Từ/brand phổ biến ở VN mà google-10000 thiếu.
EXTRA_WORDS = [
    "ielts", "toeic", "toefl", "grab", "zalo", "momo", "vnpay", "shopee",
    "lazada", "tiktok", "youtube", "facebook", "instagram", "telegram",
    "gmail", "outlook", "fintech", "startup", "livestream", "podcast",
    "blockchain", "chatbot", "fanpage", "voucher", "combo", "workshop",
    "homestay", "resort", "spa", "gym", "yoga", "vlog", "blog",
]

words = []
seen = set()
for line in open(r"C:\Users\Admin\AppData\Local\Temp\google-10000-english.txt", encoding="utf-8", errors="ignore"):
    w = line.strip().lower()
    if w and w.isalpha() and len(w) >= 2 and w not in seen:
        seen.add(w)
        words.append(w)
for w in EXTRA_WORDS:
    if w not in seen:
        seen.add(w)
        words.append(w)

print(f"{len(words)} từ")

lines = []
lines.append("// File SINH TỰ ĐỘNG bởi scripts/gen_en_top_words (nguồn: google-10000-english).")
lines.append("// Top từ tiếng Anh phổ biến — segment_oov ưu tiên biên cắt tạo từ thật")
lines.append('// ("fine|tune" thắng "fin|etune", "family|app" thắng "famil|yapp").')
lines.append("use once_cell::sync::Lazy;")
lines.append("use std::collections::HashSet;")
lines.append("")
lines.append("pub static EN_TOP_WORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {")
lines.append("    [")
row = []
for w in words:
    row.append(f'"{w}"')
    if len(row) == 8:
        lines.append("        " + ", ".join(row) + ",")
        row = []
if row:
    lines.append("        " + ", ".join(row) + ",")
lines.append("    ].into_iter().collect()")
lines.append("});")
open(r"E:\sea-g2p\src\g2p\en_top_words.rs", "w", encoding="utf-8", newline="\n").write("\n".join(lines) + "\n")
print("đã ghi src/g2p/en_top_words.rs")
