# -*- coding: utf-8 -*-
"""Dọn dict sea_g2p.bin: xử lý lớp entry "âm tiết Việt không dấu bị phoneme
tiếng Anh chiếm" ("lieu" -> <en>luː, "thuoc" -> <en>θjuːɑːk...).

Quy tắc (đã thống nhất):
  - Ứng viên: entry merged có phoneme <en>, key là ASCII và là ÂM TIẾT VIỆT
    không dấu hợp lệ (ngữ pháp âm đầu x vần), có ít nhất một "anh em có dấu"
    mang phoneme Việt trong dict.
  - Key là TỪ TIẾNG ANH THẬT (wordlist tần suất + danh sách bổ sung) ->
    chuyển sang bảng COMMON (vi = phoneme anh em có dấu, en = phoneme cũ),
    xóa khỏi merged để ngữ cảnh câu quyết định cách đọc.
  - Không phải từ Anh thật (rác corpus) -> thay phoneme merged bằng phoneme
    Việt (bỏ tag <en>).
  - Chọn anh em có dấu: ưu tiên THANH NGANG, cùng chất nguyên âm với skeleton;
    thứ tự thanh: ngang > sắc > huyền > nặng > hỏi > ngã; cùng thanh thì ưu
    tiên biến thể không đổi chất nguyên âm ("hop" -> "họp" hơn "hợp").

Chạy:  python scripts/clean_dict.py [--apply] [--wordlist FILE ...]
       mặc định dry-run: chỉ in báo cáo ra clean_dict_report.txt
"""
import argparse
import struct
import sys
import unicodedata
from collections import defaultdict
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8")

ROOT = Path(__file__).resolve().parent.parent
BIN = ROOT / "python" / "sea_g2p" / "sea_g2p.bin"
REPORT = ROOT / "scripts" / "clean_dict_report.txt"

# ── Ngữ pháp âm tiết Việt không dấu (đồng bộ technical.rs) ───────────────────
ONSETS = ["ngh", "ch", "gh", "gi", "kh", "ng", "nh", "ph", "qu", "th", "tr",
          "b", "c", "d", "g", "h", "k", "l", "m", "n", "p", "r", "s", "t",
          "v", "x", ""]
RHYMES = set("""a ac ach ai am an ang anh ao ap at au ay
e ec ech em en eng enh eo ep et eu
i ia ich iec iem ien ieng iep iet ieu im in inh ip it iu
o oa oac oach oai oan oang oanh oap oat oay oc oe oen oeo oi om on ong ooc oong op ot
u ua uan uat uay uc ue uech uenh ui um un ung uo uoc uoi uom uon uong uot uou up ut
uu uy uya uych uyen uyet uynh uyt uyu
y yem yen yet yeu""".split())


def is_vi_syllable(s):
    for onset in ONSETS:
        if s.startswith(onset) and s[len(onset):] in RHYMES:
            return True
    return False


TONE_MARKS = {"́": "sac", "̀": "huyen", "̉": "hoi",
              "̃": "nga", "̣": "nang"}
TONE_ORDER = {"ngang": 0, "sac": 1, "huyen": 2, "nang": 3, "hoi": 4, "nga": 5}


def skeleton(word):
    """Bỏ toàn bộ dấu (cả thanh lẫn chất): "thuộc" -> "thuoc", "đá" -> "da"."""
    out = []
    for ch in unicodedata.normalize("NFD", word):
        if unicodedata.combining(ch):
            continue
        out.append("d" if ch in "đĐ" else ch)
    return unicodedata.normalize("NFC", "".join(out)).lower()


def tone_of(word):
    for ch in unicodedata.normalize("NFD", word):
        if ch in TONE_MARKS:
            return TONE_MARKS[ch]
    return "ngang"


def quality_skeleton(word):
    """Bỏ THANH, giữ chất nguyên âm: "họp" -> "hop", "hợp" -> "hơp"."""
    out = [ch for ch in unicodedata.normalize("NFD", word) if ch not in TONE_MARKS]
    return unicodedata.normalize("NFC", "".join(out)).lower()


# ── Đọc bin ──────────────────────────────────────────────────────────────────
def load_bin(path):
    data = path.read_bytes()
    assert data[0:4] == b"SEAP", "bad magic"
    head_4_8 = data[4:8]
    sc, mc, cc = struct.unpack_from("<III", data, 8)
    sop, mp, cp = struct.unpack_from("<III", data, 20)

    def gs(sid):
        off = struct.unpack_from("<I", data, sop + sid * 4)[0]
        st = 32 + off
        en = data.index(b"\x00", st)
        return data[st:en].decode("utf-8")

    merged = {}
    for i in range(mc):
        w_id, p_id = struct.unpack_from("<II", data, mp + i * 8)
        merged[gs(w_id)] = gs(p_id)
    common = {}
    for i in range(cc):
        w_id, v_id, e_id = struct.unpack_from("<III", data, cp + i * 12)
        common[gs(w_id)] = (gs(v_id), gs(e_id))
    return head_4_8, merged, common


def write_bin(path, head_4_8, merged, common):
    """Ghi lại bin đúng format PhonemeDict::new đọc (sort theo byte UTF-8)."""
    strings = {}

    def sid(s):
        if s not in strings:
            strings[s] = len(strings)
        return strings[s]

    merged_rows = [(sid(w), sid(p)) for w, p in
                   sorted(merged.items(), key=lambda kv: kv[0].encode("utf-8"))]
    common_rows = [(sid(w), sid(v), sid(e)) for w, (v, e) in
                   sorted(common.items(), key=lambda kv: kv[0].encode("utf-8"))]

    blob = bytearray()
    offsets = []
    for s in strings:  # dict giữ thứ tự chèn = thứ tự id
        offsets.append(len(blob))
        blob += s.encode("utf-8") + b"\x00"

    string_data_pos = 32
    sop = string_data_pos + len(blob)
    mp = sop + 4 * len(offsets)
    cp = mp + 8 * len(merged_rows)

    out = bytearray()
    out += b"SEAP" + head_4_8
    out += struct.pack("<III", len(strings), len(merged_rows), len(common_rows))
    out += struct.pack("<III", sop, mp, cp)
    assert len(out) == 32
    out += blob
    for off in offsets:
        out += struct.pack("<I", off)
    for w, p in merged_rows:
        out += struct.pack("<II", w, p)
    for w, v, e in common_rows:
        out += struct.pack("<III", w, v, e)
    path.write_bytes(out)


# ── Chính ────────────────────────────────────────────────────────────────────
def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--apply", action="store_true", help="ghi bin mới (mặc định dry-run)")
    ap.add_argument("--wordlist", action="append", default=[],
                    help="file wordlist tiếng Anh (mỗi dòng một từ), có thể lặp")
    ap.add_argument("--viwordlist", action="append", default=[],
                    help="file wordlist tiếng Việt CÓ DẤU (Viet74K...) — sibling phải là từ thật")
    ap.add_argument("--vifreq", default=None,
                    help="file tần suất tiếng Việt (mỗi dòng 'từ tần_suất') để chọn sibling khi không có thanh ngang")
    args = ap.parse_args()

    head, merged, common = load_bin(BIN)
    print(f"dict: {len(merged)} merged, {len(common)} common")

    # Wordlist tiếng Anh: từ file + danh sách bổ sung thủ công (từ phổ biến
    # chắc chắn là tiếng Anh mà wordlist tần suất có thể thiếu).
    en_words = set()
    for wl in args.wordlist:
        for line in open(wl, encoding="utf-8", errors="ignore"):
            w = line.strip().strip("\t").lower()
            if w:
                en_words.add(w)
    en_words |= {"lieu", "hop", "tap", "ten", "tin", "thong", "cam", "van",
                 "man", "ban", "bang", "long", "sang", "sung", "hang", "hung",
                 "tang", "con", "son", "than", "thin", "tan", "am", "an",
                 "may", "me", "no", "so", "to", "do", "in", "on", "be", "he",
                 "day", "bay", "lay", "say", "gay", "im", "sum", "ton", "mom",
                 "men", "loan", "dam", "den", "chi", "chan", "bin", "sin",
                 "non", "nam", "lam", "canh", "com"}
    print(f"wordlist EN: {len(en_words)} từ")

    # Âm tiết Việt THẬT (từ wordlist có dấu): dict chứa cả lưới thanh điệu sinh
    # máy ("boát/boạt/boắt"...) nên phải lọc sibling qua từ điển từ thật.
    vi_real = set()
    for wl in args.viwordlist:
        for line in open(wl, encoding="utf-8", errors="ignore"):
            for tok in line.strip().lower().replace("-", " ").split():
                vi_real.add(tok)
    print(f"wordlist VI: {len(vi_real)} âm tiết/từ")

    # Nhóm anh em có dấu: skeleton -> [(word, phone)] — chỉ entry VI trong merged
    # VÀ là từ Việt thật.
    siblings = defaultdict(list)
    for w, p in merged.items():
        if p.startswith("<en>"):
            continue
        if w != skeleton(w) and (not vi_real or w in vi_real):
            siblings[skeleton(w)].append((w, p))

    # Tần suất tiếng Việt (chọn "một" thay vì "mót" cho skeleton "mot").
    vi_freq = {}
    if args.vifreq:
        for line in open(args.vifreq, encoding="utf-8", errors="ignore"):
            parts = line.strip().split()
            if len(parts) == 2 and parts[1].isdigit():
                vi_freq[parts[0].lower()] = int(parts[1])
        print(f"tần suất VI: {len(vi_freq)} từ")

    def pick_vi_phone(skel):
        """Ưu tiên THANH NGANG nếu là từ thật; không có ngang -> tần suất cao
        nhất; hòa -> giữ chất nguyên âm -> thứ tự thanh."""
        cands = siblings.get(skel, [])
        if not cands:
            return None, None
        def rank(item):
            w, _ = item
            is_ngang = 0 if tone_of(w) == "ngang" else 1
            quality_match = 0 if quality_skeleton(w) == skel else 1
            return (is_ngang, -vi_freq.get(w, 0), quality_match,
                    TONE_ORDER[tone_of(w)], w)
        w, p = sorted(cands, key=rank)[0]
        return w, p

    # Từ giữ nguyên cách đọc Anh bất kể: viết tắt kỹ thuật phổ biến + loanword
    # tiếng Anh đang dùng sống trong khẩu ngữ Việt ("đang chat", "con bot",
    # "32 bit", "nhạc rap", "rep tin nhắn"...) — chuyển sang common sẽ bị ngữ
    # cảnh Việt kéo về cách đọc Việt sai.
    exclude = {"min", "sec", "doc", "lib", "bin", "tam",
               "chat", "hot", "top", "bot", "bit", "hit", "cut",
               "rap", "rep", "rich", "map", "mac", "dec"}

    # Ép chọn sibling cụ thể khi cần (mặc định trống — quy tắc thanh ngang lo).
    sibling_override = {}

    changes = []  # (action, word, old_phone, new_phone, sibling)
    for w in sorted(merged):
        p = merged[w]
        if w in exclude:
            continue
        if not p.startswith("<en>"):
            continue
        if not w.isascii() or len(w) < 2 or not w.isalpha():
            continue
        if not is_vi_syllable(w):
            continue
        if w in common:
            continue
        if w in sibling_override:
            # Tra thẳng merged (tên riêng như "nguyễn" có thể không nằm trong
            # wordlist từ thường).
            target = sibling_override[w]
            tp = merged.get(target)
            sib_word, sib_phone = (target, tp) if tp and not tp.startswith("<en>") else (None, None)
        else:
            sib_word, sib_phone = pick_vi_phone(w)
        if sib_phone is None:
            continue
        if w in en_words:
            changes.append(("common", w, p, sib_phone, sib_word))
        else:
            changes.append(("vi", w, p, sib_phone, sib_word))

    # Phase BỔ SUNG: skeleton hợp lệ có sibling thật nhưng CHƯA có entry nào
    # trong dict ("thuat", "viec", "truoc"...) -> thêm entry merged đọc Việt,
    # khỏi rơi vào OOV đoán kiểu Anh.
    for skel in sorted(siblings):
        if not skel.isascii() or len(skel) < 2 or not skel.isalpha():
            continue
        if not is_vi_syllable(skel):
            continue
        if skel in merged or skel in common or skel in exclude:
            continue
        sib_word, sib_phone = pick_vi_phone(skel)
        if sib_phone is None:
            continue
        changes.append(("add", skel, "(chưa có)", sib_phone, sib_word))

    n_common = sum(1 for c in changes if c[0] == "common")
    n_vi = sum(1 for c in changes if c[0] == "vi")
    n_add = sum(1 for c in changes if c[0] == "add")
    print(f"ứng viên: {len(changes)} (chuyển common: {n_common}, ép VI: {n_vi}, thêm mới: {n_add})")

    with open(REPORT, "w", encoding="utf-8") as f:
        f.write(f"# clean_dict report — {len(changes)} thay đổi "
                f"(common: {n_common}, vi: {n_vi}, add: {n_add})\n\n")
        for action, w, old, new, sib in changes:
            f.write(f"{action:6} {w:14} {old:28} -> {new:20} (theo '{sib}')\n")
    print(f"báo cáo: {REPORT}")

    if not args.apply:
        print("dry-run — chưa ghi bin. Chạy lại với --apply để áp dụng.")
        return

    for action, w, old, new, _ in changes:
        if action == "vi" or action == "add":
            merged[w] = new
        else:
            en_plain = old.replace("<en>", "")
            common[w] = (new, en_plain)
            del merged[w]

    backup = BIN.with_suffix(".bin.bak")
    if not backup.exists():
        backup.write_bytes(BIN.read_bytes())
        print(f"backup: {backup}")
    write_bin(BIN, head, merged, common)
    print(f"đã ghi {BIN}: {len(merged)} merged, {len(common)} common")


if __name__ == "__main__":
    main()
