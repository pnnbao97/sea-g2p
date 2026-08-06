# -*- coding: utf-8 -*-
"""Convert sea_g2p.bin to SEAP v2 and load the Thai dictionary section.

Reads the existing bin (either version) plus thai/thai_dict.tsv, writes a
v2 bin with the Thai word->phoneme table as SECTION_TH. Round-trips the
result through seap.load_bin and cross-checks every entry before replacing
the original file; the previous bin is kept as sea_g2p.bin.bak.

Run:  python scripts/build_bin_v2.py
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
sys.stdout.reconfigure(encoding="utf-8")
import seap

ROOT = Path(__file__).resolve().parent.parent
BIN = ROOT / "python" / "sea_g2p" / "sea_g2p.bin"
THAI_TSV = ROOT / "thai" / "thai_dict.tsv"
FREQ_TSV = ROOT / "thai" / "blackboard_freq.tsv.gz"
INDO_TSV = ROOT / "indo" / "id_dict.tsv"


def main():
    merged, common, sections = seap.load_bin(BIN)
    print(f"loaded: merged={len(merged)} common={len(common)} "
          f"sections={{{', '.join(f'{k}:{len(v)}' for k, v in sections.items())}}}")

    thai = {}
    for line in THAI_TSV.open(encoding="utf-8"):
        line = line.rstrip("\n")
        if "\t" not in line:
            continue
        w, p = line.split("\t")
        thai[w] = p
    print(f"thai entries: {len(thai)}")
    sections[seap.SECTION_TH] = thai

    # Word frequencies for the segmenter's unigram cost model, counted from
    # HUMAN word-boundary annotation (Blackboard Treebank, CC BY 3.0) rather
    # than from newmm's segmentation of the corpus.
    #
    # This is the single largest quality change the Thai segmenter has had.
    # Counting with newmm baked newmm's compounds into the cost model, so
    # ความสัมพันธ์ and ทางกฎหมาย each looked like one frequent word and the
    # dynamic program never split them: 99.8% of the boundaries we missed
    # against human annotation were inside compounds whose parts were already
    # in the dictionary. Measured on BEST2009, F1 0.8702 -> 0.9060.
    #
    # A word absent from the annotation gets 1. That is not a gap to be
    # patched — absence is evidence the annotators split the word — and
    # blending the old newmm counts back in measured worse.
    import gzip
    counts = {}
    with gzip.open(FREQ_TSV, "rt", encoding="utf-8") as f:
        for line in f:
            line = line.rstrip("\n")
            if "\t" not in line:
                continue
            w, n = line.split("\t")
            counts[w] = n
    sections[seap.SECTION_TH_FREQ] = {w: counts.get(w, "1") for w in thai}
    print(f"thai frequencies: {len(sections[seap.SECTION_TH_FREQ])}")

    indo = {}
    for line in INDO_TSV.open(encoding="utf-8"):
        line = line.rstrip("\n")
        if "\t" not in line:
            continue
        w, p = line.split("\t")
        indo[w] = p
    sections[seap.SECTION_ID] = indo
    print(f"indonesian entries: {len(indo)}")

    tmp = BIN.with_suffix(".bin.new")
    seap.write_bin_v2(tmp, merged, common, sections)

    m2, c2, s2 = seap.load_bin(tmp)
    assert m2 == merged, "merged round-trip mismatch"
    assert c2 == common, "common round-trip mismatch"
    assert s2[seap.SECTION_TH] == thai, "thai round-trip mismatch"
    assert s2[seap.SECTION_TH_FREQ] == sections[seap.SECTION_TH_FREQ], "freq round-trip mismatch"
    assert s2[seap.SECTION_ID] == indo, "indonesian round-trip mismatch"
    print(f"round-trip OK, size {tmp.stat().st_size/1e6:.1f} MB")

    bak = BIN.with_suffix(".bin.bak")
    if BIN.exists():
        if bak.exists():
            bak.unlink()
        BIN.rename(bak)
    tmp.rename(BIN)
    print(f"wrote {BIN} (previous kept at {bak.name})")


if __name__ == "__main__":
    main()
