# -*- coding: utf-8 -*-
"""Build the Indonesian pronunciation dictionary from four sources.

Priority, highest first:

  1. **WikiPron gold** — human-transcribed IPA, 18k entries.
  2. **KBBI pelafalan** — the official Indonesian dictionary marks ⟨ê⟩ for
     /ə/ and leaves ⟨e⟩ for /e/, which is exactly the distinction spelling
     hides. Cross-checked against gold: ê->ə 2,423 times, e->e 403, against
     226 + 37 disagreements, so 91.5% agreement. This settles the one
     genuinely unpredictable feature of Indonesian orthography.
  3. **Morphology** — affixed forms derived from a known root, since every
     affix has a fixed pronunciation and only the root's schwa is unknown.
  4. **Rules** — regular orthography plus a schwa default; the fallback for
     anything the three sources above do not cover.

Everything else about Indonesian spelling is regular, so no machine G2P is
used to generate entries: espeak-ng and our own rules both score ~76% on the
gold lexicon, and baking that error rate into a dictionary would make the
data worse than reading the words by rule at runtime.
"""
import csv
import sys
from collections import Counter, defaultdict
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8")
csv.field_size_limit(10 ** 7)
from id_g2p import normalize_gold, g2p_word
from id_morph import analyse
from id_schwa_clf import SchwaClassifier

GOLD = "id_wikipron.tsv"
KBBI = "kbbi.csv"
FREQ = "id_freq.tsv"
OUT = "id_dict.tsv"


def load_gold():
    d = defaultdict(set)
    for line in open(GOLD, encoding="utf-8"):
        if "\t" not in line:
            continue
        w, p = line.rstrip("\n").split("\t")
        n = normalize_gold(p)
        if n:
            d[w.lower()].add(tuple(n))
    # one reading per word: the first in sorted order, deterministic
    return {w: list(sorted(v)[0]) for w, v in d.items()}


def load_kbbi_schwa():
    """word -> set of ⟨e⟩ indices that are /e/ rather than /ə/.

    KBBI writes the schwa as ê, so an unmarked e in a word that HAS a
    pelafalan field is /e/. Words without the field say nothing either way
    and are skipped.
    """
    out = {}
    for row in csv.DictReader(open(KBBI, encoding="utf-8")):
        word = (row.get("kata") or "").strip().lower()
        pel = (row.get("pelafalan") or "").strip().lower()
        if not word or not pel or not word.replace("-", "").isalpha():
            continue
        marks = [c for c in pel if c in "eêé"]
        if len(marks) != word.count("e"):
            continue  # cannot align; ignore rather than guess
        out[word] = {i for i, m in enumerate(marks) if m in "eé"}
    return out


def load_kbbi_words():
    """Every headword of KBBI, pronunciation or not."""
    out = set()
    for row in csv.DictReader(open(KBBI, encoding="utf-8")):
        w = (row.get("kata") or "").strip().lower()
        if w and w.replace("-", "").isalpha():
            out.add(w)
    return out


def main():
    gold = load_gold()
    schwa = load_kbbi_schwa()
    kbbi_words = load_kbbi_words()
    print(f"gold: {len(gold)} | KBBI schwa marks: {len(schwa)}")

    freq = {}
    for line in open(FREQ, encoding="utf-8"):
        w, n = line.rstrip("\n").split("\t")
        freq[w] = int(n)
    total = sum(freq.values())

    entries, source = {}, Counter()

    # 1. gold
    for w, p in gold.items():
        entries[w] = p
        source["gold"] += 1

    # 2a. KBBI words whose pelafalan answers the schwa question
    for w, e_idx in schwa.items():
        if w in entries or "-" in w:
            continue
        p = g2p_word(w, {w: e_idx})
        if p:
            entries[w] = p
            source["kbbi_pelafalan"] += 1

    # 2b. KBBI words with NO ⟨e⟩ at all. Spelling is otherwise regular, so
    # these carry no ambiguity and the rules read them outright — and they
    # are what the morphology step needs, since roots like tiap, capai,
    # jabat and liput are absent from both gold and the pelafalan subset.
    for w in kbbi_words:
        if w in entries or "-" in w or "e" in w:
            continue
        p = g2p_word(w)
        if p:
            entries[w] = p
            source["kbbi_no_e"] += 1

    # 2c. KBBI words that DO contain ⟨e⟩ but carry no pelafalan. KBBI's
    # silence is not evidence of schwa — checked against gold it holds only
    # 58.7% of the time — so the schwa comes from the context classifier
    # instead (80.1% on held-out gold). These are the least certain entries
    # in the dictionary and are the ones worth replacing first if a marked
    # source ever covers them.
    clf = SchwaClassifier(gold)
    for w in kbbi_words:
        if w in entries or "-" in w:
            continue
        p = g2p_word(w, {w: clf.predict_word(w)})
        if p:
            entries[w] = p
            source["kbbi_classifier"] += 1

    # 3. morphology over corpus words, using what we have so far
    lookup = lambda r: entries.get(r)
    for w in sorted(freq, key=lambda x: -freq[x])[:250000]:
        if w in entries or not w.isalpha():
            continue
        p = analyse(w, lookup)
        if p:
            entries[w] = p
            source["morphology"] += 1

    # 4. hyphenated reduplication: each half is looked up on its own, which
    # is also how it is spoken (orang-orang is orang twice, not a new word).
    for w in sorted(freq, key=lambda x: -freq[x])[:250000]:
        if w in entries or "-" not in w:
            continue
        parts = w.split("-")
        got = [entries.get(p) or analyse(p, lookup) for p in parts]
        if all(got):
            entries[w] = [ph for g in got for ph in g]
            source["reduplication"] += 1

    # 5. Compounds, which is how Indonesian toponyms are built. Scanning the
    # uncovered words showed the productive pieces directly: ci- attaches to
    # 176 distinct known words (Sundanese river/place names — Cimahi,
    # Cianjur), -sari to 68, kali- to 51, -rejo to 44, karang- to 38. Rather
    # than list those pieces, split anything into two known words: it covers
    # the same ground and generalises to compounds nobody enumerated.
    #
    # Both halves must be at least three letters, or short function words
    # slice ordinary vocabulary into nonsense.
    for w in sorted(freq, key=lambda x: -freq[x])[:250000]:
        if w in entries or not w.isalpha() or len(w) < 6:
            continue
        for cut in range(3, len(w) - 2):
            a, b = entries.get(w[:cut]), entries.get(w[cut:])
            if a and b:
                entries[w] = a + b
                source["compound"] += 1
                break

    print("sources:", dict(source))
    with open(OUT, "w", encoding="utf-8") as f:
        for w in sorted(entries, key=lambda x: (-freq.get(x, 0), x)):
            f.write(f"{w}\t{' '.join(entries[w])}\n")

    covered = sum(n for w, n in freq.items() if w in entries)
    print(f"\nentries: {len(entries)}")
    print(f"corpus token coverage: {covered/total:.2%}")


if __name__ == "__main__":
    main()
