# -*- coding: utf-8 -*-
"""Thai word segmentation: dictionary maximal matching over TCC units.

Algorithm (same family as PyThaiNLP's newmm, reimplemented cleanly so it can
be ported to Rust):

1. TCC (Thai Character Cluster) pass: group the text into clusters that can
   NEVER be split by any correct segmentation — a consonant plus its
   dependent (non-spacing) vowels, tone marks, thanthakhat, plus preposed
   vowels bound to the following consonant.  This guarantees no candidate
   boundary falls inside a cluster.

2. DP over cluster boundaries: at each position take either
     - a dictionary word starting here (trie walk), cost (0 unk, 1 word), or
     - one cluster as an unknown chunk, cost (len chars, 1 word).
   Minimise (unknown_chars, word_count) lexicographically.  Adjacent unknown
   chunks are merged into one token afterwards.

Non-Thai runs (Latin, digits, spaces, punctuation) are natural boundaries and
are emitted as their own tokens.
"""
import re

# ── character classes ───────────────────────────────────────────────────────
PREPOSED = "เแโใไ"          # written before the consonant they belong to
DEPENDENT = set("ะัาำิีึืุู็่้๊๋์ํ๎")  # can never start a token
THAI_RE = re.compile(r"[ก-๎]")


def is_thai(ch):
    return "ก" <= ch <= "๎"


def tcc_split(s):
    """Split a run of Thai text into inseparable clusters.

    A cluster is: [preposed vowel(s)] consonant [dependent marks] with two
    extensions: 'C ั C' (mai han akat implies a closing consonant) and
    'เCีย/เCือ/Cัว' style diphthong spellings are NOT forced together beyond
    the dependent-mark rule — the dictionary DP handles those.
    """
    out = []
    i, n = 0, len(s)
    while i < n:
        j = i
        while j < n and s[j] in PREPOSED:
            j += 1
        if j < n:
            j += 1  # the base consonant (or standalone vowel/sign)
        while j < n and s[j] in DEPENDENT:
            j += 1
        out.append(s[i:j])
        i = j
    return out


class Trie:
    __slots__ = ("children", "leaf")

    def __init__(self):
        self.children = {}
        self.leaf = False

    def insert(self, word):
        node = self
        for ch in word:
            node = node.children.setdefault(ch, Trie())
        node.leaf = True


def build_trie(words):
    t = Trie()
    for w in words:
        t.insert(w)
    return t


def _segment_thai_run(s, trie):
    """DP segmentation of a pure-Thai run; returns list of (token, known)."""
    clusters = tcc_split(s)
    # boundary index b = character offset at each cluster edge
    bounds = [0]
    for c in clusters:
        bounds.append(bounds[-1] + len(c))
    n = len(clusters)
    boundary_set = set(bounds)
    INF = (10 ** 9, 10 ** 9)
    # best[k] = (unk_chars, words) to segment clusters[:k]
    best = [INF] * (n + 1)
    back = [None] * (n + 1)  # (prev_k, token, known)
    best[0] = (0, 0)
    for k in range(n):
        if best[k] == INF:
            continue
        start = bounds[k]
        # 1) dictionary words from here (walk trie over raw chars, only
        #    accept matches that end on a cluster boundary)
        node = trie
        pos = start
        k2 = k
        while pos < len(s) and s[pos] in node.children:
            node = node.children[s[pos]]
            pos += 1
            if node.leaf and pos in boundary_set:
                while bounds[k2] < pos:
                    k2 += 1
                cand = (best[k][0], best[k][1] + 1)
                if cand < best[k2]:
                    best[k2] = cand
                    back[k2] = (k, s[start:pos], True)
        # 2) one cluster as unknown
        c = clusters[k]
        cand = (best[k][0] + len(c), best[k][1] + 1)
        if cand < best[k + 1]:
            best[k + 1] = cand
            back[k + 1] = (k, c, False)
    # reconstruct
    toks = []
    k = n
    while k > 0:
        k, tok, known = back[k]
        toks.append((tok, known))
    toks.reverse()
    # merge adjacent unknown chunks
    merged = []
    for tok, known in toks:
        if not known and merged and not merged[-1][1]:
            merged[-1] = (merged[-1][0] + tok, False)
        else:
            merged.append((tok, known))
    return merged


_RUN_RE = re.compile(r"[ก-๎]+|[^ก-๎]+")


_SARA_AM_TONE = re.compile("ํ([่้๊๋])า")
_TONE_AFTER_AA = re.compile("([าำ])([่้๊๋])")
_TONE_BEFORE_VOWEL = re.compile("([่้๊๋])([ิีึืัุู็])")
_ORPHAN_MARKS = re.compile("(?:^|(?<=[^ก-๎]))[ะัาำิีึืุู็่้๊๋์ํ๎]+")


def normalize_thai(text):
    """Real-world spelling quirks found in wiki, news and web corpora:

    - decomposed sara am: nikhahit ํ U+0E4D + า -> precomposed ำ U+0E33
      (ทํา -> ทำ), including the variant with a tone mark wedged between
      (นํ้า = น ํ ้ า -> น้ำ = น ้ ำ);
    - แ typed as two เ characters (เเละ -> และ);
    - mark-order typos, never valid in Thai orthography so the swaps are
      safe: tone typed after า/ำ (นำ้ -> น้ำ, ยา่ -> ย่า) and tone typed
      before an above/below vowel (ท่ี -> ที่);
    - orphan dependent marks at the start of a Thai run (their base
      consonant was stripped with markup/emoji) are deleted.
    """
    text = _SARA_AM_TONE.sub(lambda m: m.group(1) + "ำ", text)
    text = text.replace("ํา", "ำ")
    text = text.replace("เเ", "แ")
    text = _TONE_AFTER_AA.sub(lambda m: m.group(2) + m.group(1), text)
    text = _TONE_BEFORE_VOWEL.sub(lambda m: m.group(2) + m.group(1), text)
    text = _ORPHAN_MARKS.sub("", text)
    return text


def segment(text, trie):
    """Segment mixed text; non-Thai runs pass through as single tokens."""
    text = normalize_thai(text)
    out = []
    for m in _RUN_RE.finditer(text):
        run = m.group(0)
        if is_thai(run[0]):
            # ๆ and ฯ handled at normalizer level; split them off here
            for part in re.split(r"([ๆฯ])", run):
                if not part:
                    continue
                if part in "ๆฯ":
                    out.append((part, True))
                else:
                    out.extend(_segment_thai_run(part, trie))
        else:
            out.append((run, None))  # non-Thai
    return out


if __name__ == "__main__":
    import sys
    sys.stdout.reconfigure(encoding="utf-8")
    import os
    here = os.path.dirname(os.path.abspath(__file__))
    words = [l.split("\t")[0] for l in open(os.path.join(here, "thai_dict.tsv"), encoding="utf-8")]
    trie = build_trie(words)
    tests = [
        "เขาฉลาดพอที่จะซ่อนสติปัญญา",
        "กรุงเทพมหานครเป็นเมืองหลวงของประเทศไทย",
        "ผมชอบกินก๋วยเตี๋ยวเรือมาก",
        "โควิด-19 ระบาดในปี 2020",
        "เด็กเล่นเกม RoV บนมือถือ",
    ]
    for t in tests:
        toks = segment(t, trie)
        print(t)
        print("  ", " | ".join(f"{tok}{'?' if known is False else ''}" for tok, known in toks))
