# -*- coding: utf-8 -*-
"""Canonical Thai phoneme representation and parsers for the two sources.

Canonical syllable string: onset + vowel + coda + tone, concatenated, e.g.
    "kʰrap̚˦˥"  (ครับ),  "sa˨˩",  "diː˧" (ดี)
A word is a space-separated list of canonical syllables.

Inventory (documented in thai_phonology.md):
  onsets  : p pʰ b t tʰ d k kʰ ʔ tɕ tɕʰ f s h m n ŋ r l w j  (+ C+r/l/w clusters)
  vowels  : a aː i iː ɯ ɯː u uː e eː ɛ ɛː o oː ɔ ɔː ɤ ɤː ia ɯa ua
  codas   : p̚ t̚ k̚ ʔ m n ŋ j w  (+ loanword codas f s l), or none
  tones   : ˧ (mid) ˨˩ (low) ˥˩ (falling) ˦˥ (high) ˩˩˦ (rising)
"""
import re

TONES = ["˩˩˦", "˨˩", "˥˩", "˦˥", "˧"]  # longest first for parsing

# tltk tone digit -> canonical tone
TLTK_TONE = {"1": "˧", "2": "˨˩", "3": "˥˩", "4": "˦˥", "5": "˩˩˦"}

VOWELS = ["ia", "ɯa", "ua", "aː", "iː", "ɯː", "uː", "eː", "ɛː", "oː", "ɔː",
          "ɤː", "a", "i", "ɯ", "u", "e", "ɛ", "o", "ɔ", "ɤ"]
# f s l tɕʰ appear as codas only in loanwords (กราฟ, กรีซ, บิล, พีช); an
# optional extra "s" covers cluster codas of loanwords (มินสก์ -> ns).
CODAS = ["p̚", "t̚", "k̚", "ʔ", "m", "n", "ŋ", "j", "w", "f", "s", "l", "tɕʰ"]
ONSET_C = ["pʰ", "tʰ", "kʰ", "tɕʰ", "tɕ", "p", "b", "t", "d", "k", "ʔ",
           "f", "s", "h", "m", "n", "ŋ", "r", "l", "w", "j"]

_V = "|".join(VOWELS)
_C = "|".join(CODAS)
_O = "|".join(sorted(ONSET_C, key=len, reverse=True))
SYL_RE = re.compile(rf"^(?P<o1>{_O})(?P<o2>r|l|w)?(?P<v>{_V})(?P<c>{_C})?(?P<c2>s)?$")


class ParseError(ValueError):
    pass


class NoTone(ParseError):
    pass


def check_syl(s):
    """Validate a canonical syllable string; return (onset, vowel, coda, tone)."""
    for t in TONES:
        if s.endswith(t):
            body, tone = s[: -len(t)], t
            break
    else:
        raise NoTone(f"no tone: {s!r}")
    m = SYL_RE.match(body)
    if not m:
        raise ParseError(f"bad body: {body!r} in {s!r}")
    onset = m.group("o1") + (m.group("o2") or "")
    coda = (m.group("c") or "") + (m.group("c2") or "")
    return onset, m.group("v"), coda, tone


# ── gold (wikipron) parser ──────────────────────────────────────────────────
# Single-token normalisation applied after stripping the syllabicity mark ̯ .
GOLD_TOKEN_MAP = {
    "t͡ɕ": "tɕ", "t͡ɕʰ": "tɕʰ",
    # rare noisy leftovers normalised to the nearest canonical phone
    "c": "tɕ", "cʰ": "tɕʰ", "ɗ": "d", "ɓ": "b", "ɨ": "ɯ",
    "æ": "ɛ", "æː": "ɛː", "ə": "ɤ",
}
_IS_TONE = re.compile(r"^[˥˦˧˨˩]+$")


def parse_gold(pron):
    """'s a ˨˩ . w a t̚ ˨˩ . d iː ˧' -> ['sa˨˩', 'wat̚˨˩', 'diː˧'].

    Raises NoTone for entries lacking tone marks (unusable) and ParseError
    for anything outside the canonical inventory.
    """
    pron = pron.replace("̯", "")  # U+032F non-syllabic mark: ia̯ -> ia
    syls = []
    for chunk in pron.split(" . "):
        toks = [GOLD_TOKEN_MAP.get(t, t) for t in chunk.split() if t and t != "."]
        if not toks:
            continue
        if not _IS_TONE.match(toks[-1]):
            raise NoTone(f"no tone in gold chunk: {chunk!r}")
        tone = toks.pop()
        # "rɯ"/"lɯ" (ฤ ฦ) are consonant+vowel fused; split so that a following
        # "a" merges into the diphthong: "rɯ a" (เรือ) -> r + ɯa.
        expanded = []
        for t in toks:
            m = re.match(r"^(r|l)(ɯː?)$", t)
            if m:
                expanded.extend([m.group(1), m.group(2)])
            else:
                expanded.append(t)
        if len(expanded) >= 2 and expanded[-1] == "a" and expanded[-2] in ("ɯ", "i", "u"):
            expanded[-2:] = [expanded[-2] + "a"]
        body = "".join(expanded)
        syl = body + tone
        check_syl(syl)
        syls.append(syl)
    return syls


# ── tltk parser ─────────────────────────────────────────────────────────────
TLTK_MAP = [
    ("cʰ", "tɕʰ"), ("c", "tɕ"), ("ᴐ", "ɔ"),
    ("iːa", "ia"), ("ɯːa", "ɯa"), ("uːa", "ua"),
    ("ə", "ɤ"), ("əː", "ɤː"),
]


def parse_tltk(pron):
    """'sa2.wat2.diː1.kʰrap4 <s/>' -> ['sa˨˩', 'wat̚˨˩', 'diː˧', 'kʰrap̚˦˥']."""
    pron = pron.replace("<s/>", " ").replace("<u/>", " ").strip()
    syls = []
    for word in pron.split():
        for chunk in word.split("."):
            if not chunk:
                continue
            m = re.match(r"^(.*?)([1-5])$", chunk)
            if not m:
                raise ParseError(f"no tone digit: {chunk!r}")
            body, d = m.group(1), m.group(2)
            for a, b in TLTK_MAP:
                body = body.replace(a, b)
            # mark native final stops unreleased (only when a vowel precedes)
            if len(body) > 1 and body[-1] in "ptk" and body[-2] not in "ʰ":
                pre = body[:-1]
                if any(v in pre for v in VOWELS):
                    body = pre + body[-1] + "̚"
            syl = body + TLTK_TONE[d]
            check_syl(syl)
            syls.append(syl)
    return syls


if __name__ == "__main__":
    import sys
    sys.stdout.reconfigure(encoding="utf-8")
    cases_gold = [
        "s a ˨˩ . w a t̚ ˨˩ . d iː ˧",
        "kʰ aw ˩˩˦",
        "t͡ɕʰ a ˨˩ . l aː t̚ ˨˩",
        "n aː m ˦˥ .",
        "k r ua̯j ˧",
        "k lɯ a̯ k̚ ˨˩",
        "k r aː f ˦˥",
        "k r a ˨˩ . t͡ɕ ia̯w ˦˥",
        "rɯ a̯ ˧",
        "rɯː ˧",
    ]
    for c in cases_gold:
        print(c, "->", parse_gold(c))
    for c in ["sa2.wat2.diː1.kʰrap4 <s/>", "kʰaw5 <s/>", "pʰrᴐːm4 <s/>", "kiːat2 <s/>"]:
        print(c, "->", parse_tltk(c))
