# -*- coding: utf-8 -*-
"""Reader/writer for the SEAP phoneme-dictionary binary, versions 1 and 2.

v1 (32-byte header): magic, version=1, counts and positions for the string
pool plus the two legacy vi/en tables (merged, common). Strings live at
32 + offset.

v2 (48-byte header): identical first 32 bytes except version=2, then
section_count and sections_pos; strings live at 48 + offset. Sections are
extra per-language word->phoneme tables as (kind, count, pos) triples with
8-byte rows, binary-searched exactly like `merged`. Kinds mirror
src/core/dict.rs: 3 = Thai pronunciations, 4 = Thai word frequencies,
5 = Indonesian pronunciations.

All tables are sorted by the word's UTF-8 bytes, which matches Rust's
`&str` ordering in the binary search.
"""
import struct

SECTION_TH = 3
SECTION_TH_FREQ = 4
SECTION_ID = 5

HEADER_V1 = 32
HEADER_V2 = 48


def load_bin(path):
    """Read either version. Returns (merged, common, sections) where
    sections is {kind: {word: phonemes}} (empty for v1)."""
    data = path.read_bytes()
    assert data[0:4] == b"SEAP", "bad magic"
    version = struct.unpack_from("<I", data, 4)[0]
    sc, mc, cc = struct.unpack_from("<III", data, 8)
    sop, mp, cp = struct.unpack_from("<III", data, 20)
    base = HEADER_V2 if version >= 2 else HEADER_V1

    def gs(sid):
        off = struct.unpack_from("<I", data, sop + sid * 4)[0]
        st = base + off
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

    sections = {}
    if version >= 2:
        section_count, sections_pos = struct.unpack_from("<II", data, 32)
        for i in range(section_count):
            kind, count, pos = struct.unpack_from("<III", data, sections_pos + i * 12)
            table = {}
            for j in range(count):
                w_id, p_id = struct.unpack_from("<II", data, pos + j * 8)
                table[gs(w_id)] = gs(p_id)
            sections[kind] = table
    return merged, common, sections


def write_bin_v2(path, merged, common, sections):
    """Write a v2 file. `sections` is {kind: {word: phonemes}}."""
    strings = {}

    def sid(s):
        if s not in strings:
            strings[s] = len(strings)
        return strings[s]

    by_key = lambda kv: kv[0].encode("utf-8")
    merged_rows = [(sid(w), sid(p)) for w, p in sorted(merged.items(), key=by_key)]
    common_rows = [(sid(w), sid(v), sid(e)) for w, (v, e) in sorted(common.items(), key=by_key)]
    section_rows = {
        kind: [(sid(w), sid(p)) for w, p in sorted(table.items(), key=by_key)]
        for kind, table in sorted(sections.items())
    }

    blob = bytearray()
    offsets = []
    for s in strings:  # dict preserves insertion order = id order
        offsets.append(len(blob))
        blob += s.encode("utf-8") + b"\x00"

    sop = HEADER_V2 + len(blob)
    mp = sop + 4 * len(offsets)
    cp = mp + 8 * len(merged_rows)
    sections_pos = cp + 12 * len(common_rows)
    pos = sections_pos + 12 * len(section_rows)
    section_meta = []
    for kind, rows in section_rows.items():
        section_meta.append((kind, len(rows), pos))
        pos += 8 * len(rows)

    out = bytearray()
    out += b"SEAP" + struct.pack("<I", 2)
    out += struct.pack("<III", len(strings), len(merged_rows), len(common_rows))
    out += struct.pack("<III", sop, mp, cp)
    out += struct.pack("<II", len(section_rows), sections_pos)
    out += b"\x00" * 8  # reserved
    assert len(out) == HEADER_V2
    out += blob
    for off in offsets:
        out += struct.pack("<I", off)
    for w, p in merged_rows:
        out += struct.pack("<II", w, p)
    for w, v, e in common_rows:
        out += struct.pack("<III", w, v, e)
    for kind, count, p0 in section_meta:
        out += struct.pack("<III", kind, count, p0)
    for kind, rows in section_rows.items():
        for w, p in rows:
            out += struct.pack("<II", w, p)
    path.write_bytes(bytes(out))
