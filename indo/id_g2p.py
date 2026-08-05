# -*- coding: utf-8 -*-
"""Rule-based Indonesian grapheme-to-phoneme, plus a normalizer for the
WikiPron gold lexicon.

Indonesian orthography is close to phonemic, so rules carry most of the work
and the dictionary only has to settle what spelling genuinely does not say.

# What spelling does not say

  - **⟨e⟩ is two vowels.** It writes /ə/ (7,449 times in the gold lexicon)
    and /e/ (2,018). Nothing in the spelling distinguishes them, so the
    default is schwa and the exceptions are lexical.
  - **Final ⟨k⟩ is /k/ or /ʔ/.** Native words glottalise (banyak -> baɲaʔ),
    loanwords do not (batik -> batik). 887 vs 250 in the lexicon; /k/ is the
    safer default and the rest is lexical.

# What the gold lexicon says that we discard

Tense/lax vowel pairs (i/ɪ, u/ʊ, e/ɛ, o/ɔ) are transcribed inconsistently:
in the unambiguous closed-final-syllable context only 16-27% of /i u o/ are
lax, and 21 words carry two records differing in nothing else (Kertosono is
both kərtosono and kərtɔsɔnɔ). The distinction is not contrastive in
Indonesian, so it is transcriber noise rather than signal, and everything is
folded to the phonemic six-vowel system /a i u e o ə/.

Non-syllabic glides are written j and w, as they are for Vietnamese and Thai,
so the library keeps one inventory.
"""
import re

# ── canonical inventory ─────────────────────────────────────────────────────
VOWELS = ["a", "i", "u", "e", "o", "ə"]
CONSONANTS = [
    "p", "b", "t", "d", "k", "ɡ", "ʔ", "t͡ʃ", "d͡ʒ", "f", "v", "s", "z",
    "ʃ", "x", "h", "m", "n", "ɲ", "ŋ", "l", "r", "w", "j",
]

# gold -> canonical
FOLD = {
    "ɪ": "i", "ʊ": "u", "ɛ": "e", "ɔ": "o",
    "i̯": "j", "u̯": "w", "a̯": "a", "ʊ̯": "w", "ɪ̯": "j", "o̯": "w", "e̯": "j",
    "ə̯": "ə", "ə̂": "ə", "ə̆": "ə", "ä": "a", "ɑ": "a", "ɘ": "ə", "ʌ": "a",
    "aː": "a", "iː": "i", "uː": "u", "eː": "e", "oː": "o",
    "g": "ɡ", "c": "t͡ʃ", "ɾ": "r", "ʋ": "v", "ɣ": "x", "θ": "s", "q": "k",
    "t̪": "t", "n̪": "n", "m̪": "m", "y": "j", "ɨ": "i", "ʏ": "u", "ɤ": "o",
    "é": "e", "ĕ": "ə", "ɛ̀": "e", "ɛ̯": "j", "b̩": "b",
}
_STRIP = re.compile("[̚ʲ]")            # unreleased mark, palatalisation
_JUNK = re.compile("k͡ǀ|ʔ̚|tʷ|hʲ|xʲ")   # clicks and other artefacts


def normalize_gold(pron):
    """WikiPron transcription -> canonical phonemes, or None if unusable."""
    pron = _JUNK.sub("", pron)
    out = []
    for tok in pron.split():
        tok = _STRIP.sub("", tok)
        tok = FOLD.get(tok, tok)
        if not tok:
            continue
        if tok not in VOWELS and tok not in CONSONANTS:
            return None
        out.append(tok)
    return out or None


# ── rules ───────────────────────────────────────────────────────────────────
# Longest first: "ng" must beat "n", "sy" must beat "s".
DIGRAPHS = [
    ("ngg", "ŋɡ"), ("ng", "ŋ"), ("ny", "ɲ"), ("sy", "ʃ"), ("kh", "x"),
    # Pre-1972 spellings. The reform changed dj->j, tj->c, j->y, nj->ny,
    # sj->sy, ch->kh and oe->u, but personal and place names kept the old
    # forms: Soeharto, Djakarta, Gadjah Mada, Achmad, Tjipto. Names are 77%
    # of what the dictionary does not cover, so reading them by the modern
    # rules alone turns "gadjah" into ɡad-d͡ʒah and "achmad" into at͡ʃ-hmad.
    # nj is DELIBERATELY absent. In modern spelling it is nearly always a
    # morpheme boundary — menjadi is meN- + jadi, /mən.d͡ʒa.di/, not
    # /mə.ɲa.di/ — and 219 gold words have it against a handful of old-
    # spelling names. Adding it cost 0.89 points of gold accuracy.
    ("dj", "d͡ʒ"), ("tj", "t͡ʃ"), ("sj", "ʃ"), ("ch", "x"), ("oe", "u"),
]
# ⟨ai⟩ and ⟨au⟩ are diphthongs when nothing but consonants follow: the gold
# lexicon writes the glide 157 times against 39 hiatus readings for final
# "ai", and 72 against 3 for "au". Before a vowel they stay in hiatus
# (mai-n is two syllables), which is what the lookahead checks.
DIPHTHONGS = [("ai", "aj"), ("au", "aw"), ("oi", "oj")]
VOWEL_LETTERS = "aiueo"
SINGLE = {
    "a": "a", "i": "i", "u": "u", "o": "o", "e": "ə",
    "b": "b", "c": "t͡ʃ", "d": "d", "f": "f", "g": "ɡ", "h": "h",
    "j": "d͡ʒ", "k": "k", "l": "l", "m": "m", "n": "n", "p": "p",
    "q": "k", "r": "r", "s": "s", "t": "t", "v": "v", "w": "w",
    "x": "ks", "y": "j", "z": "z",
}


def g2p_word(word, schwa_exceptions=None):
    """Read an Indonesian word by rule.

    `schwa_exceptions` maps a lowercase word to the indices of its ⟨e⟩
    letters that are /e/ rather than /ə/; without it every ⟨e⟩ is schwa.
    """
    w = word.lower()
    e_is_e = (schwa_exceptions or {}).get(w, set())
    out = []
    i = 0
    e_index = 0
    while i < len(w):
        step = _match_diphthong(w, i) or _match_digraph(w, i)
        if step is not None:
            phones, length = step
            out.extend(phones)
            i += length
            continue
        c = w[i]
        if c == "e":
            out.append("e" if e_index in e_is_e else "ə")
            e_index += 1
        elif c in SINGLE:
            out.extend(_split(SINGLE[c]))
        i += 1
    return out


def _match_diphthong(w, i):
    """ai/au are diphthongs only at the END of a word.

    Counted over the gold lexicon: word-final "ai" takes the glide 154 times
    against 39 in hiatus, and "au" 72 against 3 — but before a consonant the
    counts invert, 124 hiatus against 53, because those are two syllables
    (ma-in, ka-in, la-in).
    """
    for src, dst in DIPHTHONGS:
        if w.startswith(src, i) and i + len(src) == len(w):
            return _split(dst), len(src)
    return None


def _match_digraph(w, i):
    for src, dst in DIGRAPHS:
        if w.startswith(src, i):
            return _split(dst), len(src)
    return None


def _split(s):
    """Split a multi-character mapping such as "ŋɡ" or "ks" into phonemes."""
    if s in CONSONANTS or s in VOWELS:
        return [s]
    out, i = [], 0
    while i < len(s):
        for n in (3, 2, 1):
            piece = s[i:i + n]
            if piece in CONSONANTS or piece in VOWELS:
                out.append(piece)
                i += n
                break
        else:
            i += 1
    return out


if __name__ == "__main__":
    import sys
    sys.stdout.reconfigure(encoding="utf-8")
    for w in ["nyanyi", "syarat", "khusus", "bangun", "banyak", "cerdas",
              "menyembunyikan", "orang", "sungguh", "ekonomi"]:
        print(f"{w:16} -> {' '.join(g2p_word(w))}")
